use crate::{
    agent::{
        context_text,
        worker::{Task, TaskMemory, WorkerPhase},
    },
    cognition::{CognitionInput, Context, Fact, Intent, IntentKind},
    core::protocol::{AgentId, WorkerDiagnostic, WorkerReport, WorkerReportStatus},
    roles::RoleProfile,
};

const MAX_HISTORY_CONTEXT_CHARS: usize = 1800;
const MAX_COMBINED_CONTEXT_CHARS: usize = 2400;
const MAX_NAMED_CONTEXT_CHARS: usize = 1200;
const MAX_CONTEXT_PACK_ITEMS_CHARS: usize = 1600;
const MAX_ASSIGNMENT_INPUT_CHARS: usize = 1600;
const MAX_ASSIGNMENT_EVIDENCE_CHARS: usize = 700;
const MAX_DIAGNOSTIC_FACT_CHARS: usize = 600;
const MAX_LIST_ITEM_CHARS: usize = 180;

pub(super) fn build_cognition_input(
    worker_id: AgentId,
    phase: WorkerPhase,
    memory: &TaskMemory,
    role: &RoleProfile,
    task: &Task,
) -> CognitionInput {
    CognitionInput {
        intent: Intent {
            id: task.id.clone(),
            kind: IntentKind::Planning,
            description: task.description.clone(),
        },
        context: build_context(worker_id, phase, memory, role, task),
    }
}

fn build_context(
    worker_id: AgentId,
    phase: WorkerPhase,
    memory: &TaskMemory,
    role: &RoleProfile,
    task: &Task,
) -> Context {
    let mut metadata = std::collections::HashMap::new();
    metadata.insert("worker_id".to_string(), worker_id.0);
    metadata.insert("role.id".to_string(), role.id.0.clone());
    metadata.insert("role.name".to_string(), role.name.clone());
    metadata.insert("role.runtime_role".to_string(), role.runtime_role.clone());
    metadata.insert("phase".to_string(), format!("{phase:?}"));

    for (key, value) in &task.context.metadata {
        metadata.insert(format!("request.{key}"), value.clone());
    }

    for (key, value) in &memory.state {
        metadata.insert(format!("memory.{key}"), value.clone());
    }

    if let Some(assignment) = &task.assignment {
        metadata.insert(
            "assignment.graph_id".to_string(),
            assignment.graph_id.0.clone(),
        );
        metadata.insert(
            "assignment.node_id".to_string(),
            assignment.node_id.0.clone(),
        );
        metadata.insert("assignment.role".to_string(), assignment.role.clone());
        metadata.insert(
            "assignment.attempt".to_string(),
            assignment.attempt.to_string(),
        );
        if let Some(llm_usage) = assignment
            .llm_usage
            .as_deref()
            .map(str::trim)
            .filter(|value| !value.is_empty())
        {
            metadata.insert("assignment.llm_usage".to_string(), llm_usage.to_string());
        }
        if let Some(skill_autosuggest) = assignment.skill_autosuggest {
            metadata.insert(
                "assignment.skill_autosuggest".to_string(),
                skill_autosuggest.to_string(),
            );
        }
        if let Some(timeout_sec) = assignment.timeout_sec {
            metadata.insert(
                "assignment.timeout_sec".to_string(),
                timeout_sec.to_string(),
            );
        }
        metadata.insert(
            "assignment.input_count".to_string(),
            assignment.inputs.len().to_string(),
        );
        if let Some(instruction) = &assignment.rework_instruction {
            metadata.insert(
                "assignment.rework_instruction".to_string(),
                instruction.clone(),
            );
        }
    }

    let mut facts = Vec::new();
    extend_runtime_strategy_facts(&task.context.metadata, &mut facts);
    extend_context_facts(&task.context.metadata, &mut facts);

    if let Some(assignment) = &task.assignment {
        facts.push(Fact {
            source: "worker.assignment".to_string(),
            content: format!(
                "node={} role={} attempt={} objective={}",
                assignment.node_id.0, assignment.role, assignment.attempt, assignment.objective
            ),
            reliability: 1.0,
        });
        for input in &assignment.inputs {
            facts.push(Fact {
                source: format!("worker.assignment.input.{}", input.node_id.0),
                content: summarize_assignment_input(input),
                reliability: 1.0,
            });
            facts.extend(input.evidence.iter().filter_map(|item| {
                summarize_assignment_evidence(item).map(|content| Fact {
                    source: format!("worker.assignment.input.{}.evidence", input.node_id.0),
                    content,
                    reliability: 1.0,
                })
            }));
            facts.extend(input.diagnostics.iter().filter_map(|diagnostic| {
                summarize_worker_diagnostic(diagnostic).map(|content| Fact {
                    source: format!("worker.assignment.input.{}.diagnostic", input.node_id.0),
                    content,
                    reliability: 1.0,
                })
            }));
        }
    }

    facts.extend(memory.progress.iter().map(|entry| Fact {
        source: "worker.runtime".to_string(),
        content: entry.clone(),
        reliability: 1.0,
    }));

    Context { facts, metadata }
}

fn extend_runtime_strategy_facts(
    metadata: &std::collections::BTreeMap<String, String>,
    facts: &mut Vec<Fact>,
) {
    let mode = runtime_response_mode(metadata);
    let answer_style = match mode {
        "quick" => {
            "回答模式：快速简答。优先给直接结论、关键理由和最多 3 \
             个下一步动作；减少铺垫、过程展开和长篇教学，除非用户明确要求详细推导。"
        }
        "execution" => {
            "回答模式：丰富执行。按用户要的交付物决定展开程度；用户要完整方案时给可执行动作、\
             风险边界和复盘字段；用户只要标题、话术、字段或短判断时仍直接交付，不强制岗位报告、\
             表格或完整方案。"
        }
        _ => {
            "回答模式：教学模式。默认面向电商学生和新手，先给可用结论，再用通俗语言解释概念、\
             判断过程、为什么这么做和下一步怎么练。"
        }
    };
    facts.push(Fact {
        source: "runtime.answer_style".to_string(),
        content: answer_style.to_string(),
        reliability: 1.0,
    });

    let explanation_depth = match runtime_learning_level(metadata) {
        "beginner" => {
            "讲解深度：低。尽量少用术语，遇到 \
             ROI、转化率、漏斗、客单价等专业词要用白话解释，并给简单例子。"
        }
        "professional" => {
            "讲解深度：高。保留关键指标、推导路径、约束假设、风险边界和可复核口径，\
             适合汇报和专业复盘。"
        }
        _ => "讲解深度：中。按电商岗位应用口径解释，术语不过度展开，但关键概念和动作口径要说清楚。",
    };
    facts.push(Fact {
        source: "runtime.explanation_depth".to_string(),
        content: explanation_depth.to_string(),
        reliability: 1.0,
    });
    facts.push(Fact {
        source: "runtime.user_visible_boundary".to_string(),
        content: "展示边界：最终回答必须写给普通学生或商家看。可以使用系统提供的商品、资料、RAG \
                  和工作区上下文做判断，但不得原样展示 \
                  context.selected_summary、context.combined_summary、Product source \
                  product、Knowledge source、Requested context \
                  refs、product:UUID、workspaceContext、内部 skill id、worker id \
                  或其它变量名；要改写成自然中文事实和业务判断。"
            .to_string(),
        reliability: 1.0,
    });
}

fn runtime_response_mode(metadata: &std::collections::BTreeMap<String, String>) -> &str {
    if let Some(response_mode) = metadata
        .get("strategy.response_mode")
        .map(String::as_str)
        .filter(|value| matches!(*value, "learning" | "quick" | "execution"))
    {
        return response_mode;
    }
    metadata
        .get("strategy.id")
        .map(String::as_str)
        .filter(|value| matches!(*value, "learning" | "quick" | "execution"))
        .unwrap_or("learning")
}

fn runtime_learning_level(metadata: &std::collections::BTreeMap<String, String>) -> &str {
    metadata
        .get("strategy.learning_level")
        .map(String::as_str)
        .filter(|value| matches!(*value, "beginner" | "higher_vocational" | "professional"))
        .unwrap_or("higher_vocational")
}

fn extend_context_facts(
    metadata: &std::collections::BTreeMap<String, String>,
    facts: &mut Vec<Fact>,
) {
    let has_selected_product_context = metadata
        .get("context.selected_summary")
        .map(|value| !value.trim().is_empty())
        .unwrap_or(false);

    extend_selected_product_priority_fact(metadata, facts);
    extend_selected_product_context_fact(
        metadata,
        facts,
        "context.selected_summary",
        "commerce.selected_product_context",
        "用户本轮手动选择的商品/知识上下文如下。\
         凡是用户说“这个商品”“该商品”“引用的商品”或要求单品分析，\
         都必须优先以这里的商品为当前对象；同会话历史只能补充背景，不能覆盖本轮手动选择。\n",
    );
    extend_conversation_history_fact(metadata, facts, has_selected_product_context);
    extend_context_summary_fact(
        metadata,
        facts,
        "context.combined_summary",
        "request.combined_context",
    );
    extend_named_context_fact(
        metadata,
        facts,
        "attachments.image_summaries_json",
        "conversation.image_preprocess_summary",
    );
    extend_named_context_fact(
        metadata,
        facts,
        "workspaceContext.summary",
        "workspace.context_summary",
    );
    extend_named_context_fact(
        metadata,
        facts,
        "workspaceContextPack.summary",
        "workspace.context_pack_summary",
    );
    extend_named_context_fact(
        metadata,
        facts,
        "workspaceContextPack.items_json",
        "workspace.context_pack.items",
    );
}

fn extend_conversation_history_fact(
    metadata: &std::collections::BTreeMap<String, String>,
    facts: &mut Vec<Fact>,
    has_selected_product_context: bool,
) {
    let Some(value) = metadata
        .get("conversation.history_summary")
        .map(|value| value.trim())
        .filter(|value| !value.is_empty())
    else {
        return;
    };
    let Some(value) = context_text::user_visible_conversation_history(value) else {
        return;
    };

    let instruction = if has_selected_product_context {
        "conversation.history：同一会话历史上下文如下。它只能用于理解追问背景、用户偏好和前后文。\
         本轮已经存在用户手动选择的商品/知识上下文时，回答“这个商品”“该商品”、单品分析、选品、\
         定价、运营或创意建议，必须以本轮手动选择的商品为准；不得把历史里的商品、价格、\
         数据或结论当成本轮当前商品。"
    } else {
        "conversation.history：同一会话历史上下文如下。回答追问、补充、继续、按前面数据、上一轮、\
         这些、同一个等明显延续问题时，必须优先结合这段历史；不要要求用户重复已经在这里提供的信息。\
         若当前问题明确说“新话题”“换个问题”“不引用任何商品/活动/资料”“不要沿用前面”，\
         或本轮没有显式选择商品、活动、文件、图片等引用且问题本身是独立泛问，历史只能作为背景，\
         不能把历史商品、活动、文件数据或上一轮结论当成本轮当前对象。"
    };

    facts.push(Fact {
        source: "conversation.history".to_string(),
        content: format!(
            "{instruction}\n{}",
            compact_text(&value, MAX_HISTORY_CONTEXT_CHARS)
        ),
        reliability: 1.0,
    });
}

fn extend_selected_product_priority_fact(
    metadata: &std::collections::BTreeMap<String, String>,
    facts: &mut Vec<Fact>,
) {
    let Some(value) = metadata
        .get("context.selected_summary")
        .map(|value| value.trim())
        .filter(|value| !value.is_empty())
    else {
        return;
    };

    let visible = context_text::user_visible_selected_product_context(value)
        .unwrap_or_else(|| "当前轮已手动选择商品，但商品资料摘要为空。".to_string());

    facts.push(Fact {
        source: "commerce.selected_product_priority".to_string(),
        content: format!(
            "本轮手动选择的商品/知识上下文优先于同会话历史。先据此识别当前商品，\
             再使用历史做辅助背景。若这里的商品与历史中的商品不一致，以这里为准；\
             若这里缺少某项指标，只能说明该指标缺失，不能借用历史商品的数据。\n{}",
            compact_text(&visible, MAX_NAMED_CONTEXT_CHARS)
        ),
        reliability: 1.0,
    });
}

fn extend_selected_product_context_fact(
    metadata: &std::collections::BTreeMap<String, String>,
    facts: &mut Vec<Fact>,
    key: &str,
    source: &str,
    prefix: &str,
) {
    let Some(value) = metadata
        .get(key)
        .map(|value| value.trim())
        .filter(|value| !value.is_empty())
    else {
        return;
    };
    let value = context_text::user_visible_selected_product_context(value)
        .unwrap_or_else(|| "当前轮已手动选择商品，但商品资料摘要为空。".to_string());

    facts.push(Fact {
        source: source.to_string(),
        content: format!("{prefix}{}", compact_text(&value, MAX_NAMED_CONTEXT_CHARS)),
        reliability: 1.0,
    });
}

fn extend_context_summary_fact(
    metadata: &std::collections::BTreeMap<String, String>,
    facts: &mut Vec<Fact>,
    key: &str,
    source: &str,
) {
    let Some(value) = metadata
        .get(key)
        .map(|value| value.trim())
        .filter(|value| !value.is_empty())
    else {
        return;
    };
    let Some(value) = context_text::user_visible_context_summary(value) else {
        return;
    };

    facts.push(Fact {
        source: source.to_string(),
        content: compact_text(&value, MAX_COMBINED_CONTEXT_CHARS),
        reliability: 1.0,
    });
}

fn extend_named_context_fact(
    metadata: &std::collections::BTreeMap<String, String>,
    facts: &mut Vec<Fact>,
    key: &str,
    source: &str,
) {
    extend_named_context_fact_with_prefix(metadata, facts, key, source, "");
}

fn extend_named_context_fact_with_prefix(
    metadata: &std::collections::BTreeMap<String, String>,
    facts: &mut Vec<Fact>,
    key: &str,
    source: &str,
    prefix: &str,
) {
    let Some(value) = metadata
        .get(key)
        .map(|value| value.trim())
        .filter(|value| !value.is_empty())
    else {
        return;
    };

    facts.push(Fact {
        source: source.to_string(),
        content: format!(
            "{prefix}{}",
            compact_text(value, named_context_limit_for_key(key))
        ),
        reliability: 1.0,
    });
}

fn named_context_limit_for_key(key: &str) -> usize {
    if key == "workspaceContextPack.items_json" {
        MAX_CONTEXT_PACK_ITEMS_CHARS
    } else {
        MAX_NAMED_CONTEXT_CHARS
    }
}

fn summarize_assignment_input(report: &WorkerReport) -> String {
    let mut lines = vec![format!(
        "上游岗位：{}；状态：{}。",
        report.role,
        worker_status_label(&report.status)
    )];

    if let Some(role_output) = &report.role_output {
        if !role_output.summary.trim().is_empty() {
            lines.push(format!(
                "摘要：{}",
                compact_text(&role_output.summary, MAX_ASSIGNMENT_INPUT_CHARS / 2)
            ));
        }
        push_labeled_items(&mut lines, "发现", &role_output.findings, 3);
        push_labeled_items(&mut lines, "建议", &role_output.recommendations, 3);
        push_labeled_items(&mut lines, "风险", &role_output.risks, 2);
        push_labeled_items(&mut lines, "待确认", &role_output.open_questions, 2);
    } else if !report.content.trim().is_empty() {
        lines.push(format!(
            "摘要：{}",
            compact_text(&report.content, MAX_ASSIGNMENT_INPUT_CHARS / 2)
        ));
    }

    push_labeled_items(&mut lines, "风险", &report.risks, 2);
    push_labeled_items(&mut lines, "待确认", &report.open_questions, 2);

    for diagnostic in &report.diagnostics {
        if let Some(summary) = summarize_worker_diagnostic(diagnostic) {
            lines.push(summary);
        }
    }

    compact_text(&lines.join("\n"), MAX_ASSIGNMENT_INPUT_CHARS)
}

fn push_labeled_items(lines: &mut Vec<String>, label: &str, items: &[String], max_items: usize) {
    let values = items
        .iter()
        .map(|item| compact_text(item, MAX_LIST_ITEM_CHARS))
        .filter(|item| !item.is_empty())
        .take(max_items)
        .collect::<Vec<_>>();
    if !values.is_empty() {
        lines.push(format!("{label}：{}", values.join("；")));
    }
}

fn worker_status_label(status: &WorkerReportStatus) -> &'static str {
    match status {
        WorkerReportStatus::Completed => "已完成",
        WorkerReportStatus::Failed => "失败",
        WorkerReportStatus::TimedOut => "超时",
    }
}

fn summarize_assignment_evidence(item: &str) -> Option<String> {
    let trimmed = item.trim();
    if trimmed.is_empty() {
        return None;
    }
    if let Some(packet) = trimmed.strip_prefix("search_evidence_packet:") {
        return summarize_search_evidence_packet(packet);
    }
    if trimmed.starts_with("search_evidence_trace:") {
        return None;
    }
    Some(compact_text(trimmed, MAX_ASSIGNMENT_EVIDENCE_CHARS))
}

fn summarize_search_evidence_packet(packet: &str) -> Option<String> {
    let value: serde_json::Value = serde_json::from_str(packet.trim()).ok()?;
    let usable_claims = value
        .get("usable_claims")
        .and_then(serde_json::Value::as_array)
        .map(|items| compact_json_string_list(items, 3))
        .unwrap_or_default();
    let gaps = value
        .get("gaps")
        .and_then(serde_json::Value::as_array)
        .map(|items| compact_json_string_list(items, 2))
        .unwrap_or_default();
    let source_count = value
        .get("sources")
        .and_then(serde_json::Value::as_array)
        .map(Vec::len)
        .unwrap_or(0);
    let source_state = value
        .get("source_state")
        .and_then(serde_json::Value::as_str)
        .unwrap_or("unknown");
    let adapter_status = value
        .get("adapter_status")
        .and_then(serde_json::Value::as_str)
        .unwrap_or("unknown");

    let mut parts = vec![format!("搜索证据摘要：来源{}个", source_count)];
    if !usable_claims.is_empty() {
        parts.push(format!("可用结论：{}", usable_claims.join("；")));
    }
    if !gaps.is_empty() {
        parts.push(format!("资料缺口：{}", gaps.join("；")));
    }
    if source_state != "unknown" {
        parts.push(format!("来源状态：{}", readable_source_state(source_state)));
    }
    if adapter_status != "unknown" {
        parts.push(format!(
            "搜索工具状态：{}",
            readable_adapter_status(adapter_status)
        ));
    }
    extend_search_filter_parts(
        &mut parts,
        json_count_at(&value, &["filter_stats", "stale_filtered_count"]),
        json_count_at(&value, &["filter_stats", "missing_url_filtered_count"]),
        json_count_at(
            &value,
            &["filter_stats", "untraceable_domain_filtered_count"],
        ),
        json_count_at(&value, &["filter_stats", "duplicate_filtered_count"]),
    );

    Some(compact_text(
        &parts.join("。"),
        MAX_ASSIGNMENT_EVIDENCE_CHARS,
    ))
}

fn compact_json_string_list(items: &[serde_json::Value], max_items: usize) -> Vec<String> {
    items
        .iter()
        .filter_map(serde_json::Value::as_str)
        .map(|item| compact_text(item, MAX_LIST_ITEM_CHARS))
        .filter(|item| !item.is_empty())
        .take(max_items)
        .collect()
}

fn summarize_worker_diagnostic(diagnostic: &WorkerDiagnostic) -> Option<String> {
    match diagnostic.kind.as_str() {
        "search_evidence" => summarize_search_diagnostic(diagnostic),
        "failure" => summarize_failure_diagnostic(diagnostic),
        _ => None,
    }
}

fn summarize_search_diagnostic(diagnostic: &WorkerDiagnostic) -> Option<String> {
    let status = diagnostic
        .fields
        .get("status")
        .map(String::as_str)
        .unwrap_or("unknown");
    let sources = diagnostic
        .fields
        .get("sources")
        .map(String::as_str)
        .unwrap_or("0");
    let claims = diagnostic
        .fields
        .get("usable_claims")
        .map(String::as_str)
        .unwrap_or("0");
    let gaps = diagnostic
        .fields
        .get("gaps")
        .map(String::as_str)
        .unwrap_or("0");
    let gap_preview = diagnostic
        .fields
        .get("gap_preview")
        .map(|value| compact_text(value, MAX_LIST_ITEM_CHARS));
    let stale_filtered_count = diagnostic_count_any(diagnostic, &["stale_filtered_count", "stale"]);
    let missing_url_filtered_count =
        diagnostic_count_any(diagnostic, &["missing_url_filtered_count", "missing_url"]);
    let untraceable_domain_filtered_count = diagnostic_count_any(
        diagnostic,
        &["untraceable_domain_filtered_count", "untraceable"],
    );
    let duplicate_filtered_count =
        diagnostic_count_any(diagnostic, &["duplicate_filtered_count", "duplicate"]);

    let mut text = format!(
        "搜索核对：{}；可采信来源{}个；可用结论{}条；资料缺口{}项。",
        readable_search_status(status),
        sources,
        claims,
        gaps
    );
    if let Some(gap_preview) = gap_preview.filter(|value| !value.is_empty()) {
        text.push_str(&format!("主要缺口：{gap_preview}。"));
    }
    let mut filter_parts = Vec::new();
    extend_search_filter_parts(
        &mut filter_parts,
        stale_filtered_count,
        missing_url_filtered_count,
        untraceable_domain_filtered_count,
        duplicate_filtered_count,
    );
    if !filter_parts.is_empty() {
        text.push_str(&filter_parts.join("。"));
        text.push('。');
    }
    Some(compact_text(&text, MAX_DIAGNOSTIC_FACT_CHARS))
}

fn summarize_failure_diagnostic(diagnostic: &WorkerDiagnostic) -> Option<String> {
    let failure_kind = diagnostic.fields.get("failure_kind")?;
    let text = match failure_kind.as_str() {
        value if value.starts_with("context_pack_") => {
            let readiness = diagnostic
                .fields
                .get("readiness")
                .map(String::as_str)
                .unwrap_or("unknown");
            format!(
                "资料核对：本轮资料读取状态为{}，部分资料可能不能作为已读全文判断。",
                readiness
            )
        }
        "skill_adapter_unavailable" => {
            "工具核对：外部工具当前不可用，本轮不能当作已完成外部查询或执行。".to_string()
        }
        "skill_adapter_timeout" => {
            "工具核对：外部工具超时，本轮不能当作已完成外部查询或执行。".to_string()
        }
        _ => return None,
    };
    Some(compact_text(&text, MAX_DIAGNOSTIC_FACT_CHARS))
}

fn readable_search_status(status: &str) -> &'static str {
    match status {
        "success" | "completed" => "已拿到搜索证据",
        "deferred" => "搜索暂未形成可靠证据",
        "failed" => "搜索失败",
        _ => "搜索状态未知",
    }
}

fn readable_adapter_status(status: &str) -> &'static str {
    match status {
        "live" => "搜索工具可用",
        "degraded" => "搜索已降级",
        "adapter_required" => "外部搜索工具未接入",
        _ => "搜索工具状态未知",
    }
}

fn readable_source_state(state: &str) -> &'static str {
    match state {
        "has_reliable_sources" => "有可采信来源",
        "no_reliable_sources" => "没有可采信来源",
        "adapter_required" => "需要接入外部搜索工具",
        _ => "未知",
    }
}

fn extend_search_filter_parts(
    parts: &mut Vec<String>,
    stale_filtered_count: usize,
    missing_url_filtered_count: usize,
    untraceable_domain_filtered_count: usize,
    duplicate_filtered_count: usize,
) {
    if stale_filtered_count > 0 {
        parts.push("本轮搜索只返回过期来源".to_string());
    }
    let untraceable_count = missing_url_filtered_count + untraceable_domain_filtered_count;
    if untraceable_count > 0 {
        parts.push("本轮搜索返回了不可追踪来源".to_string());
    }
    if duplicate_filtered_count > 0 {
        parts.push("重复来源已过滤".to_string());
    }
}

fn json_count_at(value: &serde_json::Value, path: &[&str]) -> usize {
    let mut current = value;
    for key in path {
        let Some(next) = current.get(key) else {
            return 0;
        };
        current = next;
    }
    current
        .as_u64()
        .and_then(|value| usize::try_from(value).ok())
        .unwrap_or(0)
}

fn diagnostic_count_any(diagnostic: &WorkerDiagnostic, keys: &[&str]) -> usize {
    keys.iter()
        .find_map(|key| {
            diagnostic
                .fields
                .get(*key)
                .and_then(|value| value.parse::<usize>().ok())
        })
        .unwrap_or(0)
}

fn compact_text(value: &str, max_chars: usize) -> String {
    let text = value
        .lines()
        .map(str::trim)
        .filter(|line| !line.is_empty())
        .collect::<Vec<_>>()
        .join("\n");
    truncate_chars(&text, max_chars)
}

fn truncate_chars(value: &str, max_chars: usize) -> String {
    let mut chars = value.chars();
    let head = chars.by_ref().take(max_chars).collect::<String>();
    if chars.next().is_some() {
        format!("{head}...（已截断）")
    } else {
        head
    }
}

#[cfg(test)]
mod tests {
    use std::collections::BTreeMap;

    use super::build_cognition_input;
    use crate::{
        agent::worker::{Task, TaskMemory, WorkerPhase},
        core::protocol::{
            AgentId, MessageContext, RoleWorkOutput, TaskGraphId, TaskId, TaskNodeId,
            WorkerAssignment, WorkerDiagnostic, WorkerId, WorkerReport, WorkerReportStatus,
        },
        roles::{RoleId, RoleProfile},
    };

    #[test]
    fn assignment_input_search_evidence_becomes_compact_context_fact() {
        let packet = r#"{"intent":"competitor_price_or_benchmark","adapter_status":"live","source_state":"has_reliable_sources","usable_claims":["competitor prices cluster at 12-19 CNY"],"gaps":[],"confidence":0.72}"#;
        let diagnostics = vec![WorkerDiagnostic {
            kind: "search_evidence".to_string(),
            source: "search_competitor".to_string(),
            fields: BTreeMap::from([
                ("skill".to_string(), "search_competitor".to_string()),
                ("adapter_status".to_string(), "live".to_string()),
                (
                    "source_state".to_string(),
                    "has_reliable_sources".to_string(),
                ),
                (
                    "intent".to_string(),
                    "competitor_price_or_benchmark".to_string(),
                ),
                ("sources".to_string(), "2".to_string()),
                ("usable_claims".to_string(), "1".to_string()),
                ("gaps".to_string(), "0".to_string()),
                ("confidence".to_string(), "0.72".to_string()),
            ]),
        }];
        let search_report = WorkerReport {
            graph_id: TaskGraphId("graph-search".to_string()),
            node_id: TaskNodeId("search".to_string()),
            task_id: TaskId("task-search".to_string()),
            worker_id: WorkerId("worker.search".to_string()),
            role: "search".to_string(),
            content: "Search evidence prepared.".to_string(),
            role_output: Some(RoleWorkOutput {
                summary: "Search evidence prepared.".to_string(),
                evidence: vec![format!("search_evidence_packet:{packet}")],
                ..RoleWorkOutput::default()
            }),
            evidence: vec![format!("search_evidence_packet:{packet}")],
            risks: Vec::new(),
            open_questions: Vec::new(),
            diagnostics,
            status: WorkerReportStatus::Completed,
        };
        let task = Task {
            id: "task-final".to_string(),
            context: MessageContext::default(),
            description: "Give a pricing recommendation from search evidence.".to_string(),
            requester: AgentId("commander".to_string()),
            assignment: Some(WorkerAssignment {
                graph_id: TaskGraphId("graph-final".to_string()),
                node_id: TaskNodeId("ops_final".to_string()),
                task_id: TaskId("task-final".to_string()),
                worker_id: WorkerId("worker.ops".to_string()),
                role: "ops".to_string(),
                attempt: 1,
                root_task: "Give a pricing recommendation.".to_string(),
                objective: "Synthesize search evidence into an action plan.".to_string(),
                llm_usage: None,
                skill_autosuggest: Some(false),
                timeout_sec: None,
                inputs: vec![search_report],
                rework_instruction: None,
            }),
        };
        let role = RoleProfile {
            id: RoleId("role.ops".to_string()),
            name: "Operations".to_string(),
            runtime_role: "ops".to_string(),
            priority: 1,
            domains: Vec::new(),
            keywords: Vec::new(),
            preferred_actions: Vec::new(),
            required_capabilities: Vec::new(),
            optional_capabilities: Vec::new(),
        };
        let input = build_cognition_input(
            AgentId("worker.ops".to_string()),
            WorkerPhase::Thinking,
            &TaskMemory::default(),
            &role,
            &task,
        );

        assert!(
            input.context.facts.iter().any(|fact| {
                fact.source == "worker.assignment.input.search.evidence"
                    && fact.content.contains("搜索证据摘要")
                    && fact
                        .content
                        .contains("competitor prices cluster at 12-19 CNY")
                    && !fact.content.contains("search_evidence_packet")
                    && !fact.content.contains("\"adapter_status\"")
            }),
            "search evidence should be available as compact downstream cognition fact"
        );
        assert!(
            input.context.facts.iter().any(|fact| {
                fact.source == "worker.assignment.input.search.diagnostic"
                    && fact.content.contains("搜索核对")
                    && fact.content.contains("可采信来源2个")
                    && !fact.content.contains("\"adapter_status\"")
            }),
            "typed search diagnostics should be compact and user-readable"
        );
    }

    #[test]
    fn image_preprocess_summary_becomes_context_fact() {
        let mut context = MessageContext::default();
        context.metadata.insert(
            "attachments.image_summaries_json".to_string(),
            r#"[{"sourceRef":"conversation_file:file-1","summary":"画面中心是一位年轻男性肖像。"}]"#
                .to_string(),
        );
        let task = Task {
            id: "task-image".to_string(),
            context,
            description: "图片的内容是什么？".to_string(),
            requester: AgentId("commander".to_string()),
            assignment: None,
        };
        let role = RoleProfile {
            id: RoleId("role.chat".to_string()),
            name: "Chat".to_string(),
            runtime_role: "chat".to_string(),
            priority: 1,
            domains: Vec::new(),
            keywords: Vec::new(),
            preferred_actions: Vec::new(),
            required_capabilities: Vec::new(),
            optional_capabilities: Vec::new(),
        };

        let input = build_cognition_input(
            AgentId("worker.chat".to_string()),
            WorkerPhase::Thinking,
            &TaskMemory::default(),
            &role,
            &task,
        );

        assert!(input.context.facts.iter().any(|fact| {
            fact.source == "conversation.image_preprocess_summary"
                && fact.content.contains("年轻男性肖像")
        }));
    }

    #[test]
    fn history_fact_marks_independent_new_topic_as_background_only() {
        let mut context = MessageContext::default();
        context.metadata.insert(
            "conversation.history_summary".to_string(),
            "上一轮分析了 QA矩阵智能保温杯 和 618 活动。".to_string(),
        );
        let task = Task {
            id: "task-new-topic".to_string(),
            context,
            description: "新话题，不引用任何商品：学生开店第一周看什么指标？".to_string(),
            requester: AgentId("commander".to_string()),
            assignment: None,
        };
        let role = RoleProfile {
            id: RoleId("role.chat".to_string()),
            name: "Chat".to_string(),
            runtime_role: "chat".to_string(),
            priority: 1,
            domains: Vec::new(),
            keywords: Vec::new(),
            preferred_actions: Vec::new(),
            required_capabilities: Vec::new(),
            optional_capabilities: Vec::new(),
        };

        let input = build_cognition_input(
            AgentId("worker.chat".to_string()),
            WorkerPhase::Thinking,
            &TaskMemory::default(),
            &role,
            &task,
        );
        let history = input
            .context
            .facts
            .iter()
            .find(|fact| fact.source == "conversation.history")
            .expect("history fact should be injected");

        assert!(history.content.contains("新话题"));
        assert!(history.content.contains("不引用任何商品"));
        assert!(history.content.contains("历史只能作为背景"));
        assert!(history.content.contains("不能把历史商品"));
    }

    #[test]
    fn history_fact_filters_internal_memory_and_source_debug_lines() {
        let mut context = MessageContext::default();
        context.metadata.insert(
            "conversation.history_summary".to_string(),
            "User: 上一轮直播间曝光8000、点击640、付款72。\n\
             sourceMap={\"conversation\":\"debug\"}\n\
             memoryBudget=4096\n\
             worker.assignment.input={\"context\":{\"selected_summary\":\"商品A\"}}\n\
             Assistant: 已经判断主要问题是成交承接弱。"
                .to_string(),
        );
        let task = Task {
            id: "task-history-clean".to_string(),
            context,
            description: "继续刚才的直播复盘，给今天的整改动作。".to_string(),
            requester: AgentId("commander".to_string()),
            assignment: None,
        };
        let role = RoleProfile {
            id: RoleId("role.chat".to_string()),
            name: "Chat".to_string(),
            runtime_role: "chat".to_string(),
            priority: 1,
            domains: Vec::new(),
            keywords: Vec::new(),
            preferred_actions: Vec::new(),
            required_capabilities: Vec::new(),
            optional_capabilities: Vec::new(),
        };

        let input = build_cognition_input(
            AgentId("worker.chat".to_string()),
            WorkerPhase::Thinking,
            &TaskMemory::default(),
            &role,
            &task,
        );
        let history = input
            .context
            .facts
            .iter()
            .find(|fact| fact.source == "conversation.history")
            .expect("history fact should be injected");
        let lower = history.content.to_ascii_lowercase();

        assert!(history.content.contains("曝光8000"));
        assert!(history.content.contains("成交承接弱"));
        assert!(history.content.contains("不要要求用户重复"));
        assert!(!lower.contains("sourcemap"));
        assert!(!lower.contains("memorybudget"));
        assert!(!lower.contains("worker.assignment.input"));
        assert!(!lower.contains("selected_summary"));
    }
}
