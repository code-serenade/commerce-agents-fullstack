pub(super) mod config;
pub(super) mod rules;

use config::WorkerAutosuggestPolicy;
pub(super) use config::load_autosuggest_policy;
use rules::{configured_rule_requests, contains_any, render_template_value};

use super::{
    inference::{autosuggest_match_text, search},
    policy::worker_policies,
    types::ParsedSkillRequest,
};
use crate::agent::worker::Task;

pub(super) fn autosuggest_skill_requests(
    role_runtime: &str,
    task: &Task,
    existing_requests: &[ParsedSkillRequest],
) -> Vec<ParsedSkillRequest> {
    let policy = &worker_policies().autosuggest;
    let match_text = autosuggest_match_text(task);
    let render_text = match_text.clone();
    let Some(route) = policy
        .role_routes
        .iter()
        .find(|route| route.role == role_runtime)
    else {
        return Vec::new();
    };

    let mut requests = Vec::new();
    if route.learning_first
        && let Some(request) = learning_request(policy, &match_text, task)
    {
        requests.push(request);
        if route.mode != "requests" {
            return dedupe_new_requests(requests, existing_requests);
        }
    }

    match route.handler.as_str() {
        "chat" => {
            let request_count_before_chat = requests.len();
            extend_optional(&mut requests, chat_request(policy, &match_text, task));
            if requests.len() == request_count_before_chat || route.mode == "requests" {
                requests.extend(configured_rule_requests(
                    policy,
                    role_runtime,
                    &match_text,
                    &render_text,
                    task,
                    &route.mode,
                ));
            }
        }
        "search" => requests.extend(configured_rule_requests(
            policy,
            role_runtime,
            &match_text,
            &render_text,
            task,
            &route.mode,
        )),
        _ => requests.extend(configured_rule_requests(
            policy,
            role_runtime,
            &match_text,
            &render_text,
            task,
            &route.mode,
        )),
    }

    if route.handler != "search" && requests.is_empty() {
        let fallback_mode = if route.mode == "requests" {
            "requests"
        } else {
            "single"
        };
        requests.extend(configured_rule_requests(
            policy,
            "search",
            &match_text,
            &render_text,
            task,
            fallback_mode,
        ));
    }

    normalize_search_strength(&mut requests, &match_text, task);
    dedupe_new_requests(requests, existing_requests)
}

fn chat_request(
    policy: &WorkerAutosuggestPolicy,
    text: &str,
    task: &Task,
) -> Option<ParsedSkillRequest> {
    if contains_any(text, &policy.chat.save_memory_keywords) {
        return Some(ParsedSkillRequest {
            skill_id: policy.chat.save_memory_skill_id.clone(),
            input: render_template_value(&policy.chat.save_memory_input, text, task),
            reason: Some("autosuggest:chat.save_memory".to_string()),
        });
    }
    if contains_any(text, &policy.chat.status_query_keywords) {
        return Some(ParsedSkillRequest {
            skill_id: policy.chat.status_query_skill_id.clone(),
            input: render_template_value(&policy.chat.status_query_input, text, task),
            reason: Some("autosuggest:chat.status_query".to_string()),
        });
    }
    learning_request(policy, text, task)
}

fn learning_request(
    policy: &WorkerAutosuggestPolicy,
    text: &str,
    task: &Task,
) -> Option<ParsedSkillRequest> {
    let has_explicit_keyword = contains_any(text, &policy.learning.explicit_keywords);
    let has_question_with_explanation = contains_any(text, &policy.learning.question_keywords)
        && contains_any(text, &policy.learning.explanation_keywords);
    if !has_explicit_keyword && !has_question_with_explanation {
        return None;
    }

    Some(ParsedSkillRequest {
        skill_id: policy.learning.skill_id.clone(),
        input: render_learning_input(policy, text, task),
        reason: Some("autosuggest:learning".to_string()),
    })
}

fn render_learning_input(
    policy: &WorkerAutosuggestPolicy,
    text: &str,
    task: &Task,
) -> serde_json::Value {
    let mut input = render_template_value(&policy.learning.input, text, task);
    if let Some(object) = input.as_object_mut() {
        object.insert(
            "include_self_check".to_string(),
            serde_json::json!(!contains_any(
                text,
                &policy.learning.self_check_disabled_keywords
            )),
        );
    }
    input
}

fn extend_optional<T>(items: &mut Vec<T>, item: Option<T>) {
    if let Some(item) = item {
        items.push(item);
    }
}

fn dedupe_new_requests(
    requests: Vec<ParsedSkillRequest>,
    existing_requests: &[ParsedSkillRequest],
) -> Vec<ParsedSkillRequest> {
    let mut seen = std::collections::HashSet::new();
    for request in existing_requests {
        seen.insert(request.skill_id.clone());
    }

    let mut unique = Vec::new();
    for request in requests {
        if seen.insert(request.skill_id.clone()) {
            unique.push(request);
        }
    }
    unique
}

fn normalize_search_strength(requests: &mut [ParsedSkillRequest], text: &str, task: &Task) {
    let Some(max_results) = search::max_results_for_task(text, task) else {
        return;
    };

    for request in requests {
        if !request.skill_id.starts_with("search_") {
            continue;
        }
        let Some(input) = request.input.as_object_mut() else {
            continue;
        };
        if input
            .get("max_results")
            .is_none_or(serde_json::Value::is_null)
        {
            input.insert("max_results".to_string(), serde_json::json!(max_results));
        }
    }
}

#[cfg(test)]
mod tests {
    use super::autosuggest_skill_requests;
    use crate::{
        agent::worker::Task,
        core::protocol::{
            AgentId, MessageContext, TaskGraphId, TaskId, TaskNodeId, WorkerAssignment, WorkerId,
        },
    };

    fn task(description: &str) -> Task {
        Task {
            id: "task-1".to_string(),
            context: MessageContext::default(),
            description: description.to_string(),
            requester: AgentId("commander".to_string()),
            assignment: None,
        }
    }

    fn task_with_response_mode(description: &str, response_mode: &str) -> Task {
        let mut task = task(description);
        task.context.metadata.insert(
            "strategy.response_mode".to_string(),
            response_mode.to_string(),
        );
        task
    }

    fn assignment_task(root_task: &str, objective: &str) -> Task {
        Task {
            id: "task-1".to_string(),
            context: MessageContext::default(),
            description: format!("{root_task}\n\n当前岗位任务：\n{objective}"),
            requester: AgentId("commander".to_string()),
            assignment: Some(WorkerAssignment {
                graph_id: TaskGraphId("graph-1".to_string()),
                node_id: TaskNodeId("data".to_string()),
                task_id: TaskId("task-1".to_string()),
                worker_id: WorkerId("worker.data".to_string()),
                role: "data".to_string(),
                attempt: 1,
                root_task: root_task.to_string(),
                objective: objective.to_string(),
                llm_usage: None,
                skill_autosuggest: None,
                timeout_sec: None,
                inputs: Vec::new(),
                rework_instruction: None,
            }),
        }
    }

    fn skill_ids(role: &str, description: &str) -> Vec<String> {
        autosuggest_skill_requests(role, &task(description), &[])
            .into_iter()
            .map(|request| request.skill_id)
            .collect()
    }

    #[test]
    fn search_role_competitor_price_prefers_competitor_search() {
        let skills = skill_ids("search", "请搜索并对比校园冷萃咖啡竞品定价");

        assert!(skills.contains(&"search_competitor".to_string()));
        assert!(!skills.contains(&"search_platform_policy".to_string()));
    }

    #[test]
    fn search_role_platform_policy_prefers_policy_search() {
        let skills = skill_ids(
            "search",
            "请联网搜索抖音电商价格合规、低价引流、虚假宣传的官方公开规则和来源链接",
        );

        assert!(skills.contains(&"search_platform_policy".to_string()));
        assert!(!skills.contains(&"search_competitor".to_string()));
    }

    #[test]
    fn no_search_request_disables_search_autosuggest_even_with_market_keywords() {
        let prompt = "我想卖一款学生宿舍用桌面收纳盒，售价39元，只有普通白底图，没有销量数据。请先判断能不能小测，再给主图、标题、验证数据三个方向，不要联网，不要搜索，不要编造销量。";

        for role in ["search", "data", "ops", "creative", "design"] {
            let skills = skill_ids(role, prompt);
            assert!(
                skills
                    .iter()
                    .all(|skill_id| !skill_id.starts_with("search_")),
                "role {role} should not trigger search skills for no-search request: {skills:?}"
            );
        }
    }

    #[test]
    fn business_roles_do_not_fallback_search_for_attachment_only_profit() {
        let skills = skill_ids("accounting", "只看附件里的 Excel 表，帮我算一下毛利和利润");

        assert!(
            skills.contains(&"material_table_profiler".to_string()),
            "attachment-only profit table task should trigger table profiling first: {skills:?}"
        );
        assert!(
            skills
                .iter()
                .all(|skill_id| !skill_id.starts_with("search_")),
            "attachment-only profit task should not trigger search fallback: {skills:?}"
        );
    }

    #[test]
    fn accounting_budget_boundary_uses_finance_skills_before_material_profiler() {
        let skills = skill_ids(
            "accounting",
            "作为财务分析师，评估预算、毛利、CPA/ROI \
             止损口径和放量风险；资料不足时给可补数的成本结构表和公式，不能给固定止损阈值：\
             基于选中商品和活动，输出一份执行方案，包含卖点、预算边界、客服承接和复盘指标。",
        );

        assert!(
            skills.contains(&"accounting_roi_calc".to_string()),
            "ROI wording should trigger accounting ROI skill: {skills:?}"
        );
        assert!(
            skills.contains(&"accounting_budget_plan".to_string()),
            "budget boundary wording should trigger accounting budget skill: {skills:?}"
        );
        assert!(
            !skills.contains(&"material_table_profiler".to_string()),
            "cost structure wording alone must not mask finance skills as a table task: {skills:?}"
        );
    }

    #[test]
    fn material_keywords_trigger_file_understanding_without_search() {
        let skills = skill_ids("ops", "请读取上传的 PDF 和 Word，结合附件资料做运营诊断");

        assert!(skills.contains(&"material_file_understanding".to_string()));
        assert!(
            skills
                .iter()
                .all(|skill_id| !skill_id.starts_with("search_")),
            "local material understanding should not trigger web search: {skills:?}"
        );
    }

    #[test]
    fn chat_role_can_fall_through_to_configured_material_rules() {
        let skills = skill_ids("chat", "请读取上传的 Word 文件并总结关键证据");

        assert_eq!(skills, vec!["material_file_understanding".to_string()]);
    }

    #[test]
    fn table_keywords_trigger_table_profiler_before_generic_material() {
        let skills = skill_ids("data", "请剖析上传的 Excel 表格字段和指标口径");

        assert_eq!(skills, vec!["material_table_profiler".to_string()]);
    }

    #[test]
    fn requested_answer_table_does_not_trigger_material_table_profiler() {
        let skills = skill_ids(
            "data",
            "现在切换成手动多智能体：数据、创意、运营一起给一个学生书包开学季小促销方案，\
             要求有表格、有标题示例、有风险边界。",
        );

        assert!(
            !skills.contains(&"material_table_profiler".to_string()),
            "asking the answer to include a table must not be treated as uploaded table input: \
             {skills:?}"
        );
    }

    #[test]
    fn graph_objective_table_words_do_not_trigger_material_table_profiler() {
        let task = assignment_task(
            "不上传任何表格，只要求你把学生书包的分析输出成表格：列出指标、怎么看、可能问题、\
             下一步动作。",
            "把用户问题转成可复盘数据口径。没有用户上传的表格、Excel、CSV \
             或附件时，不要说无法分析，不要等待附件，先按用户文字和已选商品给可执行口径。",
        );
        let skills = autosuggest_skill_requests("data", &task, &[])
            .into_iter()
            .map(|request| request.skill_id)
            .collect::<Vec<_>>();

        assert!(
            !skills.contains(&"material_table_profiler".to_string()),
            "internal graph objective table wording must not trigger table profiling: {skills:?}"
        );
    }

    #[test]
    fn graph_root_real_table_input_still_triggers_material_table_profiler() {
        let task = assignment_task(
            "请剖析上传的 Excel 表格字段和指标口径",
            "没有用户上传的表格、Excel、CSV 或附件时，不要等待附件。",
        );
        let requests = autosuggest_skill_requests("data", &task, &[]);
        let table_request = requests
            .iter()
            .find(|request| request.skill_id == "material_table_profiler")
            .expect("real uploaded table request should still trigger table profiling");

        assert_eq!(
            table_request.input.get("user_question"),
            Some(&serde_json::json!("请剖析上传的 Excel 表格字段和指标口径"))
        );
        assert!(
            !table_request.input.to_string().contains("当前岗位任务"),
            "skill input should not include internal graph objective: {:?}",
            table_request.input
        );
    }

    #[test]
    fn image_keywords_trigger_image_diagnosis_before_generic_material() {
        let skills = skill_ids("chat", "请诊断上传的主图图片，看看卖点呈现和文案可读性");

        assert_eq!(skills, vec!["material_image_diagnosis".to_string()]);
    }

    #[test]
    fn evidence_keywords_trigger_evidence_extractor_before_generic_material() {
        let skills = skill_ids("chat", "请根据上传资料提取证据链，并标注可引用依据");

        assert_eq!(skills, vec!["material_evidence_extractor".to_string()]);
    }

    #[test]
    fn multi_file_keywords_trigger_compare_before_generic_material() {
        let skills = skill_ids("chat", "请对比多份资料的共同点、差异点和冲突点");

        assert_eq!(skills, vec!["material_multi_file_compare".to_string()]);
    }

    #[test]
    fn product_opportunity_keywords_trigger_score_before_assortment() {
        let skills = skill_ids(
            "ops",
            "帮我判断低糖冷萃咖啡这个商品值不值得做，给机会评分和小预算验证建议",
        );

        assert_eq!(skills, vec!["product_opportunity_score".to_string()]);
    }

    #[test]
    fn product_market_fit_keywords_trigger_pmf_diagnosis() {
        let skills = skill_ids(
            "ops",
            "帮我诊断这个商品适合什么人群和渠道，看看商品市场匹配 PMF 是否成立",
        );

        assert_eq!(skills, vec!["product_market_fit_diagnosis".to_string()]);
    }

    #[test]
    fn experiment_keywords_trigger_mvp_plan() {
        let skills = skill_ids(
            "ops",
            "给低糖冷萃咖啡做一个14天小预算MVP实验计划，包含实验分组、成功线和止损线",
        );

        assert_eq!(skills, vec!["experiment_mvp_plan".to_string()]);
    }

    #[test]
    fn experiment_review_keywords_trigger_review_report() {
        let skills = skill_ids(
            "ops",
            "低糖冷萃咖啡MVP实验结果出来了，帮我做实验复盘，判断哪组效果好、是否放量还是停止",
        );

        assert_eq!(skills, vec!["experiment_review_report".to_string()]);
    }

    #[test]
    fn creative_role_does_not_search_for_short_title_rewrite() {
        let skills = skill_ids("creative", "只改这个商品标题，短一点，不要解释");

        assert!(
            skills
                .iter()
                .all(|skill_id| !skill_id.starts_with("search_")),
            "short copy rewrite should not trigger search fallback: {skills:?}"
        );
    }

    #[test]
    fn inferred_search_strength_is_added_to_business_search_requests() {
        let requests = autosuggest_skill_requests(
            "accounting",
            &task("请深度搜索行业财务报告和毛利率公开资料，再给我财务口径"),
            &[],
        );
        let search_request = requests
            .iter()
            .find(|request| request.skill_id == "search_market_info")
            .expect("accounting finance search should be requested");

        assert_eq!(
            search_request.input.get("max_results"),
            Some(&serde_json::json!(10))
        );
    }

    #[test]
    fn configured_light_fallback_search_strength_is_not_overwritten() {
        let requests =
            autosuggest_skill_requests("data", &task("帮我了解咖啡这个商品的市场背景"), &[]);
        let search_request = requests
            .iter()
            .find(|request| request.skill_id == "search_market_info")
            .expect("data market fallback search should be requested");

        assert_eq!(
            search_request.input.get("max_results"),
            Some(&serde_json::json!(3))
        );
    }

    #[test]
    fn quick_mode_defaults_search_strength_to_light_when_search_is_triggered() {
        let requests = autosuggest_skill_requests(
            "search",
            &task_with_response_mode("请搜索校园冷萃咖啡消费趋势", "quick"),
            &[],
        );
        let search_request = requests
            .iter()
            .find(|request| request.skill_id == "search_trends")
            .expect("trend search should be requested");

        assert_eq!(
            search_request.input.get("max_results"),
            Some(&serde_json::json!(3))
        );
    }

    #[test]
    fn execution_mode_defaults_search_strength_to_standard_when_search_is_triggered() {
        let requests = autosuggest_skill_requests(
            "search",
            &task_with_response_mode("请搜索校园冷萃咖啡消费趋势", "execution"),
            &[],
        );
        let search_request = requests
            .iter()
            .find(|request| request.skill_id == "search_trends")
            .expect("trend search should be requested");

        assert_eq!(
            search_request.input.get("max_results"),
            Some(&serde_json::json!(6))
        );
    }

    #[test]
    fn explicit_search_count_overrides_response_mode_default_strength() {
        let requests = autosuggest_skill_requests(
            "search",
            &task_with_response_mode("请搜索5条校园冷萃咖啡消费趋势", "quick"),
            &[],
        );
        let search_request = requests
            .iter()
            .find(|request| request.skill_id == "search_trends")
            .expect("trend search should be requested");

        assert_eq!(
            search_request.input.get("max_results"),
            Some(&serde_json::json!(5))
        );
    }
}
