use tracing::debug;

use super::{
    builder::build_configured_task_graph,
    config::{
        TaskGraphRule, TaskGraphRuleMatch, TaskGraphRuleSet, configured_task_graph_rules_path,
        load_task_graph_rules,
    },
};
use crate::{
    agent::commander::task_graph::util::{contains_keyword, contains_string_any},
    core::protocol::{TaskGraph, TaskId},
    roles::RoleRoute,
};

pub(super) fn plan_configured_task_graph(
    task_id: &TaskId,
    task: &str,
    route: Option<&RoleRoute>,
) -> Option<TaskGraph> {
    let path = configured_task_graph_rules_path()?;

    let rules = match load_task_graph_rules(&path) {
        Ok(rules) => rules,
        Err(error) => {
            crate::log_error_msg!(&format!(
                "task graph rules ignored: path={}, error={error}",
                path.display()
            ));
            return None;
        }
    };
    let graph = plan_task_graph_from_rules(task_id, task, route, &rules)?;
    Some(graph)
}

pub(super) fn plan_configured_non_chat_task_graph(
    task_id: &TaskId,
    task: &str,
    route: Option<&RoleRoute>,
) -> Option<TaskGraph> {
    let path = configured_task_graph_rules_path()?;

    let rules = match load_task_graph_rules(&path) {
        Ok(rules) => rules,
        Err(error) => {
            crate::log_error_msg!(&format!(
                "task graph rules ignored: path={}, error={error}",
                path.display()
            ));
            return None;
        }
    };
    let graph = plan_task_graph_from_rules_matching(task_id, task, route, &rules, |graph| {
        let matches = graph_has_non_chat_role(graph);
        if !matches {
            debug!(
                graph_id = %graph.graph_id.0,
                roles = ?graph.nodes.iter().map(|node| node.role.as_str()).collect::<Vec<_>>(),
                "configured task graph skipped because it has no non-chat roles"
            );
        }
        matches
    })?;
    Some(graph)
}

fn plan_task_graph_from_rules(
    task_id: &TaskId,
    task: &str,
    route: Option<&RoleRoute>,
    rules: &TaskGraphRuleSet,
) -> Option<TaskGraph> {
    plan_task_graph_from_rules_matching(task_id, task, route, rules, |_| true)
}

fn plan_task_graph_from_rules_matching(
    task_id: &TaskId,
    task: &str,
    route: Option<&RoleRoute>,
    rules: &TaskGraphRuleSet,
    graph_matches: impl Fn(&TaskGraph) -> bool,
) -> Option<TaskGraph> {
    let rule = rules
        .rules
        .iter()
        .filter_map(|rule| {
            if !rule.matches(task, route, rules) {
                return None;
            }
            let graph = build_configured_task_graph(task_id, task, route, rules, rule);
            (!graph.nodes.is_empty() && graph_matches(&graph)).then_some((rule, graph))
        })
        .max_by(|left, right| {
            left.0
                .priority
                .cmp(&right.0.priority)
                .then_with(|| right.0.id.cmp(&left.0.id))
        })?;
    Some(rule.1)
}

impl TaskGraphRule {
    pub(super) fn matches(
        &self,
        task: &str,
        route: Option<&RoleRoute>,
        rules: &TaskGraphRuleSet,
    ) -> bool {
        if self.id == "direct_chat_short_answer" && route_has_non_chat_roles(route) {
            return false;
        }
        self.legacy_match().matches(task, route, rules)
            && self.r#match.matches(task, route, rules)
            && (!self.r#match.explicit_team_request
                || contains_string_any(task, &explicit_team_keywords(rules)))
            && (!self.dynamic_roles.from_explicit_request.enabled
                || contains_string_any(
                    task,
                    &self.dynamic_roles.from_explicit_request.trigger_keywords,
                )
                || self
                    .dynamic_roles
                    .from_explicit_request
                    .trigger_keywords
                    .is_empty())
    }

    fn legacy_match(&self) -> TaskGraphRuleMatch {
        TaskGraphRuleMatch {
            any_keywords: self.match_any.clone(),
            all_keywords: self.match_all.clone(),
            all_keyword_groups: self.match_all_any.clone(),
            allow_without_route: true,
            ..TaskGraphRuleMatch::default()
        }
    }
}

impl TaskGraphRuleMatch {
    pub(super) fn matches(
        &self,
        task: &str,
        route: Option<&RoleRoute>,
        rules: &TaskGraphRuleSet,
    ) -> bool {
        let task_chars = task.trim().chars().count();
        if let Some(min_chars) = self.min_chars
            && task_chars < min_chars
        {
            return false;
        }
        if let Some(max_chars) = self.max_chars
            && task_chars > max_chars
        {
            return false;
        }
        if !self
            .all_keywords
            .iter()
            .all(|keyword| contains_keyword(task, keyword))
        {
            return false;
        }
        if !self.any_keywords.is_empty()
            && !self
                .any_keywords
                .iter()
                .any(|keyword| contains_keyword(task, keyword))
        {
            return false;
        }
        if !self
            .all_keyword_groups
            .iter()
            .all(|group| group.iter().any(|keyword| contains_keyword(task, keyword)))
        {
            return false;
        }
        if self
            .none_keywords
            .iter()
            .any(|keyword| contains_keyword(task, keyword))
        {
            return false;
        }
        if let Some(min_aliases) = self.min_explicit_role_aliases {
            let explicit_role_count = rules
                .role_aliases
                .values()
                .filter(|aliases| aliases.iter().any(|alias| contains_keyword(task, alias)))
                .count();
            if explicit_role_count < min_aliases {
                return false;
            }
        }
        self.route_matches(route)
    }

    fn route_matches(&self, route: Option<&RoleRoute>) -> bool {
        let needs_route = !self.route_primary_any.is_empty()
            || !self.route_support_any.is_empty()
            || self.route_score_min.is_some()
            || self.route_min_non_chat_roles.is_some();
        if !needs_route {
            return true;
        }
        let Some(route) = route else {
            return self.allow_without_route;
        };
        if !self.route_primary_any.is_empty()
            && !self
                .route_primary_any
                .iter()
                .any(|role| role == &route.primary_runtime_role)
        {
            return false;
        }
        if !self.route_support_any.is_empty()
            && !route.support_runtime_roles.iter().any(|role| {
                self.route_support_any
                    .iter()
                    .any(|expected| expected == role)
            })
        {
            return false;
        }
        if let Some(min_score) = self.route_score_min
            && (route.scores.is_empty() || route.scores.iter().any(|score| score.score < min_score))
        {
            return false;
        }
        if let Some(min_roles) = self.route_min_non_chat_roles {
            let count = std::iter::once(route.primary_runtime_role.as_str())
                .chain(route.support_runtime_roles.iter().map(String::as_str))
                .filter(|role| *role != "chat")
                .count();
            if count < min_roles {
                return false;
            }
        }
        true
    }
}

fn route_has_non_chat_roles(route: Option<&RoleRoute>) -> bool {
    let Some(route) = route else {
        return false;
    };
    route.primary_runtime_role != "chat"
        || route
            .support_runtime_roles
            .iter()
            .any(|role| role != "chat")
}

fn graph_has_non_chat_role(graph: &TaskGraph) -> bool {
    graph.nodes.iter().any(|node| node.role != "chat")
}

pub(super) fn explicit_team_keywords(rules: &TaskGraphRuleSet) -> Vec<String> {
    let mut keywords = rules.defaults.explicit_team_keywords.clone();
    keywords.extend(rules.rules.iter().flat_map(|rule| {
        rule.dynamic_roles
            .from_explicit_request
            .trigger_keywords
            .clone()
    }));
    keywords
}

#[cfg(test)]
mod tests {
    use super::{load_task_graph_rules, plan_task_graph_from_rules};
    use crate::{
        core::protocol::TaskId,
        roles::{RoleId, RoleRoute, RoleScore},
    };

    fn configured_rules_path() -> String {
        format!(
            "{}/../../config/json/agent/task-graph/index.json",
            env!("CARGO_MANIFEST_DIR")
        )
    }

    fn manual_route(primary_runtime_role: &str, support_runtime_roles: &[&str]) -> RoleRoute {
        let mut scores = Vec::with_capacity(1 + support_runtime_roles.len());
        scores.push(manual_role_score(primary_runtime_role, 100.0));
        scores.extend(
            support_runtime_roles
                .iter()
                .map(|role| manual_role_score(role, 90.0)),
        );

        RoleRoute {
            primary_role: RoleId(format!("role.{primary_runtime_role}")),
            primary_runtime_role: primary_runtime_role.to_string(),
            support_roles: support_runtime_roles
                .iter()
                .map(|role| RoleId(format!("role.{role}")))
                .collect(),
            support_runtime_roles: support_runtime_roles
                .iter()
                .map(|role| role.to_string())
                .collect(),
            scores,
        }
    }

    fn manual_role_score(runtime_role: &str, score: f32) -> RoleScore {
        RoleScore {
            role_id: RoleId(format!("role.{runtime_role}")),
            runtime_role: runtime_role.to_string(),
            score,
            keyword_hits: 0,
        }
    }

    fn assert_readable_answer_layout_objective(objective: &str) {
        assert!(objective.contains("中文小标题"), "{objective}");
        assert!(objective.contains("要点"), "{objective}");
        assert!(objective.contains("不要只输出普通段落"), "{objective}");
        assert!(!objective.contains("# 多智能体协作结论"), "{objective}");
        assert!(
            objective.contains("Markdown 管道表格") || objective.contains("Markdown 表格"),
            "{objective}"
        );
    }

    #[test]
    fn image_content_question_routes_to_single_chat_graph() {
        let rules =
            load_task_graph_rules(configured_rules_path()).expect("task graph rules should load");
        let graph = plan_task_graph_from_rules(
            &TaskId("task-image-question".to_string()),
            "图片内容是什么",
            None,
            &rules,
        )
        .expect("image question should match a graph");

        assert_eq!(graph.nodes.len(), 1);
        assert_eq!(graph.nodes[0].id.0, "chat");
        assert_eq!(graph.nodes[0].role, "chat");
        assert_eq!(graph.nodes[0].title, "图片直答");
    }

    #[test]
    fn generic_content_word_does_not_route_to_content_creation_team() {
        let rules =
            load_task_graph_rules(configured_rules_path()).expect("task graph rules should load");
        let graph = plan_task_graph_from_rules(
            &TaskId("task-content-word".to_string()),
            "这个资料内容是什么",
            None,
            &rules,
        );

        assert!(
            graph
                .as_ref()
                .is_none_or(|graph| graph.nodes.iter().all(|node| node.role == "chat")),
            "generic content question must not start a non-chat team graph"
        );
    }

    #[test]
    fn selected_context_analysis_routes_to_direct_chat_answer() {
        let rules =
            load_task_graph_rules(configured_rules_path()).expect("task graph rules should load");
        let route = manual_route("data", &["ops"]);
        let graph = plan_task_graph_from_rules(
            &TaskId("task-selected-context-analysis".to_string()),
            "请重新分析这个商品和这些资料，先告诉我最关键的判断，再说下一步怎么验证，不要只说缺数据。",
            Some(&route),
            &rules,
        )
        .expect("selected product and material analysis should match direct graph");

        assert_eq!(graph.nodes.len(), 1);
        assert_eq!(graph.nodes[0].id.0, "ops");
        assert_eq!(graph.nodes[0].role, "ops");
        assert_eq!(graph.nodes[0].title, "商品资料直答分析");
        assert_eq!(graph.nodes[0].llm_usage.as_deref(), Some("fast"));
        assert_eq!(graph.nodes[0].skill_autosuggest, Some(false));
    }

    #[test]
    fn selected_context_analysis_allows_one_sentence_key_judgment_request() {
        let rules =
            load_task_graph_rules(configured_rules_path()).expect("task graph rules should load");
        let route = manual_route("data", &["ops"]);
        let graph = plan_task_graph_from_rules(
            &TaskId("task-selected-context-one-sentence".to_string()),
            "请再分析一次这个商品和这些资料，先用一句话说最关键判断，再给3个验证动作，不要只说缺数据。",
            Some(&route),
            &rules,
        )
        .expect("one-sentence key judgment request should still match direct graph");

        assert_eq!(graph.nodes.len(), 1);
        assert_eq!(graph.nodes[0].id.0, "ops");
        assert_eq!(graph.nodes[0].role, "ops");
        assert_eq!(graph.nodes[0].title, "商品资料直答分析");
    }

    #[test]
    fn selected_context_analysis_matches_material_and_poster_wording() {
        let rules =
            load_task_graph_rules(configured_rules_path()).expect("task graph rules should load");
        let route = manual_route("creative", &["ops"]);
        let graph = plan_task_graph_from_rules(
            &TaskId("task-selected-context-poster".to_string()),
            "这张海报和这个素材怎么看，先给关键判断，再给3个验证动作。",
            Some(&route),
            &rules,
        )
        .expect("poster and material wording should match direct selected-context graph");

        assert_eq!(graph.nodes.len(), 1);
        assert_eq!(graph.nodes[0].id.0, "ops");
        assert_eq!(graph.nodes[0].role, "ops");
        assert_eq!(graph.nodes[0].title, "商品资料直答分析");
    }

    #[test]
    fn selected_context_analysis_is_available_to_non_chat_graph_entrypoint() {
        let rules =
            load_task_graph_rules(configured_rules_path()).expect("task graph rules should load");
        let route = manual_route("data", &["ops"]);
        let graph = super::plan_task_graph_from_rules_matching(
            &TaskId("task-selected-context-non-chat-entry".to_string()),
            "请再分析一次这个商品和这些资料，先用一句话说最关键判断，再给3个验证动作，不要只说缺数据。",
            Some(&route),
            &rules,
            super::graph_has_non_chat_role,
        )
        .expect("direct selected-context graph should not be filtered by non-chat entrypoint");

        assert_eq!(graph.nodes.len(), 1);
        assert_eq!(graph.nodes[0].id.0, "ops");
        assert_eq!(graph.nodes[0].title, "商品资料直答分析");
    }

    #[test]
    fn commerce_team_route_dynamic_roles_do_not_default_depend_on_data() {
        let rules =
            load_task_graph_rules(configured_rules_path()).expect("task graph rules should load");
        let route = RoleRoute {
            primary_role: RoleId("role.data".to_string()),
            primary_runtime_role: "data".to_string(),
            support_roles: vec![RoleId("role.ops".to_string())],
            support_runtime_roles: vec!["ops".to_string()],
            scores: vec![
                manual_role_score("data", 88.0),
                manual_role_score("ops", 70.0),
            ],
        };
        let graph = plan_task_graph_from_rules(
            &TaskId("task-commerce-team".to_string()),
            "分析这个咖啡商品的转化问题，给我一个运营方案。",
            Some(&route),
            &rules,
        )
        .expect("commerce team graph should match");

        let ops = graph
            .nodes
            .iter()
            .find(|node| node.id.0 == "ops")
            .expect("route support ops node should be planned");

        assert!(
            ops.input_refs.is_empty(),
            "route support ops should be able to run even if data is slow: {:?}",
            ops.input_refs
        );
        assert_eq!(ops.llm_usage.as_deref(), Some("fast"));
        assert_eq!(ops.skill_autosuggest, Some(false));
    }

    #[test]
    fn explicit_source_search_routes_to_web_search_graph() {
        let rules =
            load_task_graph_rules(configured_rules_path()).expect("task graph rules should load");
        let graph = plan_task_graph_from_rules(
            &TaskId("task-source-search".to_string()),
            "请联网搜索并给出来源链接：最近30天抖音电商关于价格合规、低价引流、\
             虚假宣传的官方公开规则或公告；只输出来源标题、链接、发布日期和适用风险点。",
            None,
            &rules,
        )
        .expect("explicit source search should match a graph");

        assert_eq!(graph.nodes.len(), 1);
        assert_eq!(graph.nodes[0].id.0, "search_evidence");
        assert_eq!(graph.nodes[0].role, "search");
        assert_eq!(graph.nodes[0].title, "实时来源搜索");
    }

    #[test]
    fn search_plus_business_advice_routes_to_search_then_final_synthesis() {
        let rules =
            load_task_graph_rules(configured_rules_path()).expect("task graph rules should load");
        let graph = plan_task_graph_from_rules(
            &TaskId("task-search-synthesis".to_string()),
            "请搜索并对比校园冷萃咖啡竞品定价，然后给我一个可执行的定价建议。",
            None,
            &rules,
        )
        .expect("search plus advice should match a graph");

        let roles = graph
            .nodes
            .iter()
            .map(|node| node.role.as_str())
            .collect::<Vec<_>>();
        assert_eq!(roles, vec!["search", "ops"]);
        assert_eq!(graph.nodes[0].id.0, "search_evidence");
        assert_eq!(graph.nodes[0].skill_autosuggest, Some(true));

        let final_node = graph
            .nodes
            .iter()
            .find(|node| node.id.0 == "ops_final")
            .expect("search synthesis graph should have final synthesis node");
        let input_refs = final_node
            .input_refs
            .iter()
            .map(|input| input.0.as_str())
            .collect::<Vec<_>>();
        assert_eq!(input_refs, vec!["search_evidence"]);
        assert_eq!(final_node.skill_autosuggest, Some(false));
    }

    #[test]
    fn ecommerce_term_question_routes_to_light_business_explanation() {
        let rules =
            load_task_graph_rules(configured_rules_path()).expect("task graph rules should load");
        let graph = plan_task_graph_from_rules(
            &TaskId("task-term-explanation".to_string()),
            "电商转化率是什么？一句话解释。",
            None,
            &rules,
        )
        .expect("ecommerce term question should match direct explanation graph");

        assert_eq!(graph.nodes.len(), 1);
        assert_eq!(graph.graph_id.0, "graph-task-term-explanation");
        assert_eq!(graph.nodes[0].role, "data");
        assert_eq!(graph.nodes[0].title, "电商术语解释");
        assert_eq!(graph.nodes[0].skill_autosuggest, Some(false));
    }

    #[test]
    fn source_only_search_stays_evidence_only_without_final_synthesis() {
        let rules =
            load_task_graph_rules(configured_rules_path()).expect("task graph rules should load");
        let graph = plan_task_graph_from_rules(
            &TaskId("task-source-only-search".to_string()),
            "请联网搜索校园冷萃咖啡竞品价格，只输出来源标题、链接和发布日期。",
            None,
            &rules,
        )
        .expect("source-only search should match a graph");

        assert_eq!(graph.nodes.len(), 1);
        assert_eq!(graph.nodes[0].id.0, "search_evidence");
        assert_eq!(graph.nodes[0].role, "search");
    }

    #[test]
    fn explicit_source_search_route_beats_role_route_graph() {
        let rules =
            load_task_graph_rules(configured_rules_path()).expect("task graph rules should load");
        let route = RoleRoute {
            primary_role: RoleId("role.data-analyst".to_string()),
            primary_runtime_role: "data".to_string(),
            support_roles: vec![RoleId("role.ops-strategist".to_string())],
            support_runtime_roles: vec!["ops".to_string()],
            scores: vec![
                RoleScore {
                    role_id: RoleId("role.data-analyst".to_string()),
                    runtime_role: "data".to_string(),
                    score: 95.0,
                    keyword_hits: 2,
                },
                RoleScore {
                    role_id: RoleId("role.ops-strategist".to_string()),
                    runtime_role: "ops".to_string(),
                    score: 95.0,
                    keyword_hits: 2,
                },
            ],
        };
        let graph = plan_task_graph_from_rules(
            &TaskId("task-source-search-route".to_string()),
            "请联网搜索并给出来源链接：最近30天抖音电商关于价格合规、低价引流、\
             虚假宣传的官方公开规则或公告；只输出来源标题、链接、发布日期和适用风险点。",
            Some(&route),
            &rules,
        )
        .expect("explicit source search should match a graph");

        assert_eq!(graph.nodes.len(), 1);
        assert_eq!(graph.nodes[0].id.0, "search_evidence");
        assert_eq!(graph.nodes[0].role, "search");
    }

    #[test]
    fn explicit_team_enables_business_role_autosuggest_by_role_override() {
        let rules =
            load_task_graph_rules(configured_rules_path()).expect("task graph rules should load");
        let graph = plan_task_graph_from_rules(
            &TaskId("task-explicit-search-team".to_string()),
            "请指定运营和资料搜索员一起协作，搜索竞品价格后给定价建议。",
            None,
            &rules,
        )
        .expect("explicit team should match a graph");

        let ops = graph
            .nodes
            .iter()
            .find(|node| node.role == "ops")
            .expect("ops node should exist");
        let search = graph
            .nodes
            .iter()
            .find(|node| node.role == "search")
            .expect("search node should exist");

        assert_eq!(ops.skill_autosuggest, Some(true));
        assert_eq!(search.skill_autosuggest, Some(true));
        assert_eq!(search.llm_usage.as_deref(), Some("fast"));
        assert!(
            ops.input_refs
                .iter()
                .any(|input_ref| input_ref.0 == search.id.0),
            "ops should depend on search evidence when both roles are explicitly requested"
        );
    }

    #[test]
    fn explicit_team_search_remains_ready_when_data_and_other_roles_depend_on_search() {
        let rules =
            load_task_graph_rules(configured_rules_path()).expect("task graph rules should load");
        let graph = plan_task_graph_from_rules(
            &TaskId("task-explicit-search-data-creative-team".to_string()),
            "请指定运营、资料搜索员、数据、创意、视觉设计、抖音投放和网页岗位一起协作，\
             搜索校园冷萃咖啡竞品价格后判断数据口径、内容方向、设计表达、投放风险和页面承接。",
            None,
            &rules,
        )
        .expect("explicit search/data/creative team should match a graph");

        let search = graph
            .nodes
            .iter()
            .find(|node| node.role == "search")
            .expect("search node should exist");
        assert!(
            search.input_refs.is_empty(),
            "search should remain an upstream ready node when other roles depend on it; \
             input_refs={:?}",
            search.input_refs
        );

        for role in ["data", "creative", "design", "douyin_ads", "web"] {
            let node = graph
                .nodes
                .iter()
                .find(|node| node.role == role)
                .unwrap_or_else(|| panic!("{role} node should exist"));
            assert!(
                node.input_refs
                    .iter()
                    .any(|input_ref| input_ref.0 == search.id.0),
                "{role} should depend on search evidence; input_refs={:?}",
                node.input_refs
            );
        }
    }

    #[test]
    fn selected_route_keeps_manual_ops_data_roles_with_autosuggest() {
        let rules =
            load_task_graph_rules(configured_rules_path()).expect("task graph rules should load");
        let route = manual_route("ops", &["data"]);
        let graph = plan_task_graph_from_rules(
            &TaskId("task-manual-ops-data".to_string()),
            "帮我判断校园冷萃咖啡怎么定价",
            Some(&route),
            &rules,
        )
        .expect("manual multi-role route should match a graph");

        let roles = graph
            .nodes
            .iter()
            .map(|node| node.role.as_str())
            .collect::<Vec<_>>();
        assert!(roles.contains(&"data"), "data role must be scheduled");
        assert!(roles.contains(&"ops"), "ops role must be scheduled");
        for node in graph.nodes.iter().filter(|node| node.id.0 != "ops_final") {
            assert_eq!(node.llm_usage.as_deref(), Some("fast"));
            assert_eq!(node.skill_autosuggest, Some(true));
        }

        let final_node = graph
            .nodes
            .iter()
            .find(|node| node.id.0 == "ops_final")
            .expect("manual multi-role graph should have final synthesis node");
        let input_refs = final_node
            .input_refs
            .iter()
            .map(|input| input.0.as_str())
            .collect::<Vec<_>>();
        assert!(input_refs.contains(&"data"));
        assert!(input_refs.contains(&"ops"));
        assert_eq!(final_node.llm_usage.as_deref(), Some("fast"));
        assert_eq!(final_node.skill_autosuggest, Some(false));
    }

    #[test]
    fn selected_route_does_not_override_short_greeting_chat_graph() {
        let rules =
            load_task_graph_rules(configured_rules_path()).expect("task graph rules should load");
        let route = manual_route("ops", &["data", "creative", "service"]);
        let graph = plan_task_graph_from_rules(
            &TaskId("task-short-greeting-manual-route".to_string()),
            "hi",
            Some(&route),
            &rules,
        )
        .expect("short greeting should still match quick chat graph");

        assert_eq!(graph.nodes.len(), 1);
        assert_eq!(graph.nodes[0].id.0, "chat");
        assert_eq!(graph.nodes[0].role, "chat");
        assert_eq!(graph.nodes[0].title, "快速对话");
    }

    #[test]
    fn forced_single_business_role_keeps_autosuggest_enabled() {
        let graph = crate::agent::commander::task_graph::plan_forced_role_graph(
            &TaskId("task-forced-service".to_string()),
            "客户投诉保温杯漏水，请以客服岗位生成处理流程。",
            "service",
        );

        assert_eq!(graph.nodes.len(), 1);
        assert_eq!(graph.nodes[0].role, "service");
        assert_eq!(graph.nodes[0].skill_autosuggest, Some(true));
    }

    #[test]
    fn forced_chat_graph_keeps_autosuggest_disabled() {
        let graph = crate::agent::commander::task_graph::plan_forced_chat_graph(
            &TaskId("task-forced-chat".to_string()),
            "你好",
        );

        assert_eq!(graph.nodes.len(), 1);
        assert_eq!(graph.nodes[0].role, "chat");
        assert_eq!(graph.nodes[0].skill_autosuggest, Some(false));
    }

    #[test]
    fn negative_classroom_format_request_does_not_route_to_teacher_feedback() {
        let rules =
            load_task_graph_rules(configured_rules_path()).expect("task graph rules should load");
        let graph = plan_task_graph_from_rules(
            &TaskId("task-not-classroom-homework".to_string()),
            "分析这个商品，给出电商运营建议。不要写课堂作业格式。",
            None,
            &rules,
        )
        .expect("business analysis should still match a graph");

        assert_ne!(graph.graph_id.0, "direct_teacher_feedback_boundary");
        assert!(
            graph.nodes.iter().any(|node| node.role == "data"),
            "business analysis should keep commerce graph data role"
        );
        assert!(
            graph.nodes.iter().any(|node| node.id.0 == "ops_final"),
            "commerce graph should keep final synthesis node"
        );
        assert!(
            graph
                .nodes
                .iter()
                .all(|node| node.title != "老师点评边界回答"),
            "business analysis should not be misrouted to teacher feedback"
        );
    }

    #[test]
    fn generic_product_analysis_routes_to_commerce_team_with_final_synthesis() {
        let rules =
            load_task_graph_rules(configured_rules_path()).expect("task graph rules should load");
        let graph = plan_task_graph_from_rules(
            &TaskId("task-product-analysis".to_string()),
            "请分析这个商品，判断是否适合做电商实训选品。",
            None,
            &rules,
        )
        .expect("product analysis should match commerce team graph");

        let roles = graph
            .nodes
            .iter()
            .map(|node| node.role.as_str())
            .collect::<Vec<_>>();
        assert!(roles.contains(&"data"), "{roles:?}");
        assert!(roles.contains(&"creative"), "{roles:?}");
        let final_node = graph
            .nodes
            .iter()
            .find(|node| node.id.0 == "ops_final")
            .expect("commerce graph must schedule a distinct final synthesis node");
        assert_readable_answer_layout_objective(&final_node.objective);
        assert!(
            graph.nodes.iter().filter(|node| node.id.0 == "ops").count() <= 1,
            "ops worker node must not collide with ops_final synthesis"
        );
    }

    #[test]
    fn explicit_requested_team_final_node_uses_readable_answer_layout() {
        let rules =
            load_task_graph_rules(configured_rules_path()).expect("task graph rules should load");
        let graph = plan_task_graph_from_rules(
            &TaskId("task-explicit-team-format".to_string()),
            "只让运营、数据、创意三个岗位协作，给学生书包开学季活动方案。",
            None,
            &rules,
        )
        .expect("explicit team task should match a graph");

        let final_node = graph
            .nodes
            .iter()
            .find(|node| node.id.0 == "ops_final")
            .expect("explicit team graph should have final synthesis node");
        assert_readable_answer_layout_objective(&final_node.objective);
    }

    #[test]
    fn search_business_synthesis_final_node_uses_readable_answer_layout() {
        let rules =
            load_task_graph_rules(configured_rules_path()).expect("task graph rules should load");
        let graph = plan_task_graph_from_rules(
            &TaskId("task-search-business-format".to_string()),
            "请联网搜索校园冷萃咖啡竞品定价，并给我可执行定价建议和下一步动作。",
            None,
            &rules,
        )
        .expect("search plus business advice should match a graph");

        let final_node = graph
            .nodes
            .iter()
            .find(|node| node.id.0 == "ops_final")
            .expect("search business graph should have final synthesis node");
        assert_readable_answer_layout_objective(&final_node.objective);
    }

    #[test]
    fn default_product_metric_learning_routes_to_commerce_team_not_single_data() {
        let rules =
            load_task_graph_rules(configured_rules_path()).expect("task graph rules should load");
        let route = RoleRoute {
            primary_role: RoleId("role.data-analyst".to_string()),
            primary_runtime_role: "data".to_string(),
            support_roles: vec![RoleId("role.ops-strategist".to_string())],
            support_runtime_roles: vec!["ops".to_string()],
            scores: vec![
                RoleScore {
                    role_id: RoleId("role.data-analyst".to_string()),
                    runtime_role: "data".to_string(),
                    score: 6.5,
                    keyword_hits: 3,
                },
                RoleScore {
                    role_id: RoleId("role.ops-strategist".to_string()),
                    runtime_role: "ops".to_string(),
                    score: 2.5,
                    keyword_hits: 1,
                },
            ],
        };
        let graph = plan_task_graph_from_rules(
            &TaskId("task-product-metrics".to_string()),
            "请用新手学生能懂的话，说明电商商品分析先看哪三个指标，并用学生书包举例，\
             不要只给一句话。",
            Some(&route),
            &rules,
        )
        .expect("product metric learning task should match commerce team graph");

        let roles = graph
            .nodes
            .iter()
            .map(|node| node.role.as_str())
            .collect::<Vec<_>>();
        assert!(
            roles.contains(&"data") && roles.contains(&"creative") && roles.contains(&"ops"),
            "default product metric learning must use the commerce chain, got {roles:?}"
        );
        assert!(
            graph.nodes.iter().any(|node| node.id.0 == "ops_final"),
            "default product metric learning must include final synthesis: {:#?}",
            graph.nodes
        );
        assert!(
            graph.nodes.len() >= 3,
            "default product metric learning must not collapse to single data node: {:#?}",
            graph.nodes
        );
    }

    #[test]
    fn quick_field_analysis_routes_to_stable_single_chat() {
        let rules =
            load_task_graph_rules(configured_rules_path()).expect("task graph rules should load");
        let route = RoleRoute {
            primary_role: RoleId("role.data-analyst".to_string()),
            primary_runtime_role: "data".to_string(),
            support_roles: Vec::new(),
            support_runtime_roles: Vec::new(),
            scores: vec![
                RoleScore {
                    role_id: RoleId("role.data-analyst".to_string()),
                    runtime_role: "data".to_string(),
                    score: 4.5,
                    keyword_hits: 2,
                },
                RoleScore {
                    role_id: RoleId("role.ops-strategist".to_string()),
                    runtime_role: "ops".to_string(),
                    score: 2.5,
                    keyword_hits: 1,
                },
                RoleScore {
                    role_id: RoleId("role.creative-studio".to_string()),
                    runtime_role: "creative".to_string(),
                    score: 2.5,
                    keyword_hits: 1,
                },
                RoleScore {
                    role_id: RoleId("role.design-core".to_string()),
                    runtime_role: "design".to_string(),
                    score: 2.5,
                    keyword_hits: 1,
                },
            ],
        };
        let graph = plan_task_graph_from_rules(
            &TaskId("task-quick-phone-case-fields".to_string()),
            "连续追问测试B：我想卖手机壳，请简短分析标题、主图、价格、库存和售后风险。",
            Some(&route),
            &rules,
        )
        .expect("quick field analysis should match stable direct chat graph");

        assert_eq!(graph.nodes.len(), 1, "{:#?}", graph.nodes);
        assert_eq!(graph.nodes[0].role, "chat");
        assert_eq!(graph.nodes[0].llm_usage.as_deref(), Some("fast"));
        assert_eq!(graph.nodes[0].skill_autosuggest, Some(false));
    }

    #[test]
    fn followup_format_transform_routes_to_stable_single_chat() {
        let rules =
            load_task_graph_rules(configured_rules_path()).expect("task graph rules should load");
        let route = manual_route("data", &["creative", "design"]);
        let graph = plan_task_graph_from_rules(
            &TaskId("task-followup-format-transform".to_string()),
            "基于你刚才的回答，整理成一张对照表，要简单。",
            Some(&route),
            &rules,
        )
        .expect("follow-up format transform should match stable direct chat graph");

        assert_eq!(graph.nodes.len(), 1, "{:#?}", graph.nodes);
        assert_eq!(graph.nodes[0].role, "chat");
        assert_eq!(graph.nodes[0].llm_usage.as_deref(), Some("fast"));
        assert_eq!(graph.nodes[0].skill_autosuggest, Some(false));
        assert!(graph.nodes[0].objective.contains("上一轮用户材料"));
    }

    #[test]
    fn rich_followup_delivery_routes_to_thinking_chat() {
        let rules =
            load_task_graph_rules(configured_rules_path()).expect("task graph rules should load");
        let route = manual_route("data", &["creative", "design"]);
        let graph = plan_task_graph_from_rules(
            &TaskId("task-rich-followup-delivery".to_string()),
            "我不会给你提供任何其他信息，但是你必须根据我刚才所给资料和你的回答进行丰富回答，要有干货。",
            Some(&route),
            &rules,
        )
        .expect("rich follow-up should match forced delivery graph");

        assert_eq!(graph.nodes.len(), 1, "{:#?}", graph.nodes);
        assert_eq!(graph.nodes[0].role, "chat");
        assert_eq!(graph.nodes[0].title, "强制丰富交付");
        assert_eq!(graph.nodes[0].llm_usage.as_deref(), Some("thinking"));
        assert_eq!(graph.nodes[0].skill_autosuggest, Some(false));
        assert!(graph.nodes[0].objective.contains("不要再补资料"));
        assert!(graph.nodes[0].objective.contains("丰富回答"));
    }

    #[test]
    fn followup_homework_phrase_without_review_request_stays_business_graph() {
        let rules =
            load_task_graph_rules(configured_rules_path()).expect("task graph rules should load");
        let graph = plan_task_graph_from_rules(
            &TaskId("task-followup-business".to_string()),
            "继续刚才三个指标，整理已经稳定的部分、仍有风险的部分和下次作业。",
            None,
            &rules,
        );

        assert!(
            graph.as_ref().is_none_or(|graph| graph
                .nodes
                .iter()
                .all(|node| node.title != "老师点评边界回答")),
            "plain follow-up must not drift into teacher feedback graph: {graph:#?}"
        );
    }

    #[test]
    fn explicit_homework_review_still_routes_to_teacher_feedback() {
        let rules =
            load_task_graph_rules(configured_rules_path()).expect("task graph rules should load");
        let graph = plan_task_graph_from_rules(
            &TaskId("task-homework-review".to_string()),
            "请像老师点评一样批改这个作业，指出哪里只是口号。",
            None,
            &rules,
        )
        .expect("explicit teacher review should match a direct graph");

        assert_eq!(graph.nodes.len(), 1);
        assert_eq!(graph.nodes[0].role, "chat");
        assert_eq!(graph.nodes[0].title, "老师点评边界回答");
    }

    #[test]
    fn selected_route_keeps_search_autosuggest_enabled_but_fast() {
        let rules =
            load_task_graph_rules(configured_rules_path()).expect("task graph rules should load");
        let route = manual_route("search", &["ops"]);
        let graph = plan_task_graph_from_rules(
            &TaskId("task-manual-search-ops".to_string()),
            "请结合公开搜索资料判断抖音电商保温杯内容趋势，并说明哪些结论来自搜索资料。",
            Some(&route),
            &rules,
        )
        .expect("manual search and ops route should match a graph");

        let search = graph
            .nodes
            .iter()
            .find(|node| node.role == "search")
            .expect("search node must be scheduled");
        let ops = graph
            .nodes
            .iter()
            .find(|node| node.role == "ops" && node.id.0 == "ops")
            .expect("ops node must be scheduled");
        let final_node = graph
            .nodes
            .iter()
            .find(|node| node.id.0 == "ops_final")
            .expect("manual search graph should have final synthesis node");

        assert_eq!(search.llm_usage.as_deref(), Some("fast"));
        assert_eq!(search.skill_autosuggest, Some(true));
        assert_eq!(ops.llm_usage.as_deref(), Some("fast"));
        assert_eq!(ops.skill_autosuggest, Some(true));
        assert!(
            ops.input_refs
                .iter()
                .any(|input_ref| input_ref.0 == search.id.0),
            "route-selected ops should depend on search evidence"
        );
        assert_eq!(final_node.llm_usage.as_deref(), Some("fast"));
        assert_eq!(final_node.skill_autosuggest, Some(false));
        assert_readable_answer_layout_objective(&final_node.objective);
    }

    #[test]
    fn selected_route_keeps_manual_accounting_ops_roles_without_text_collaboration_keyword() {
        let rules =
            load_task_graph_rules(configured_rules_path()).expect("task graph rules should load");
        let route = manual_route("accounting", &["ops"]);
        let graph = plan_task_graph_from_rules(
            &TaskId("task-manual-accounting-ops".to_string()),
            "帮我算这个活动预算和毛利风险",
            Some(&route),
            &rules,
        )
        .expect("manual accounting and ops route should match a graph");

        let roles = graph
            .nodes
            .iter()
            .map(|node| node.role.as_str())
            .collect::<Vec<_>>();
        assert!(
            roles.contains(&"accounting"),
            "accounting role must be scheduled"
        );
        assert!(roles.contains(&"ops"), "ops role must be scheduled");

        let final_node = graph
            .nodes
            .iter()
            .find(|node| node.id.0 == "ops_final")
            .expect("manual multi-role graph should have final synthesis node");
        let input_refs = final_node
            .input_refs
            .iter()
            .map(|input| input.0.as_str())
            .collect::<Vec<_>>();
        assert!(input_refs.contains(&"accounting"));
        assert!(input_refs.contains(&"ops"));
        assert_readable_answer_layout_objective(&final_node.objective);
    }

    #[test]
    fn selected_route_rule_itself_keeps_manual_creative_design_image_roles() {
        let rules =
            load_task_graph_rules(configured_rules_path()).expect("task graph rules should load");
        let route = manual_route("creative", &["design"]);
        let graph = plan_task_graph_from_rules(
            &TaskId("task-manual-creative-design".to_string()),
            "只按创意和设计岗位分析这张主图，指出普通用户第一眼会不会看懂卖点。",
            Some(&route),
            &rules,
        )
        .expect("manual creative and design route should match selected-route graph");

        let roles = graph
            .nodes
            .iter()
            .map(|node| node.role.as_str())
            .collect::<Vec<_>>();
        assert!(
            roles.contains(&"creative"),
            "creative role must be scheduled"
        );
        assert!(roles.contains(&"design"), "design role must be scheduled");
        assert!(
            !roles.contains(&"data"),
            "selected route graph itself must not inject data"
        );
        assert!(
            !roles.contains(&"accounting"),
            "selected route graph itself must not inject accounting"
        );
    }
}
