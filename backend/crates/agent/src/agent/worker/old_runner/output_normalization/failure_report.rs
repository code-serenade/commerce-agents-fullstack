use super::*;

pub(super) fn category_boundary_failure_report(
    assignment: &WorkerAssignment,
    task: &Task,
    _reason: &str,
    worker_id: &AgentId,
) -> crate::core::protocol::WorkerReport {
    let task_line = compact_task_text(&task.description);
    let role = assignment.role.as_str();
    let mut output = RoleWorkOutput {
        summary: format!(
            "{}按类目级或未指定商品链路完成边界判断：{}",
            role_display_name(role),
            task_line
        ),
        findings: default_role_findings(role, &task_line),
        recommendations: default_role_recommendations(role, &task_line),
        evidence: vec![
            "本轮未检测到 context.selected_summary；不注入任何预设商品资料，只按用户文本、\
             会话历史和工作区上下文理解问题。"
                .to_string(),
            CATEGORY_LEVEL_BOUNDARY_MARKER.to_string(),
            "部分岗位原始输出结构不完整，已按类目级业务边界降级为可复核的方向判断。".to_string(),
        ],
        risks: vec![
            "类目级或未指定商品结论只能用于方向判断；内部执行细节不会作为用户结论展示，\
             后续需要用真实经营数据复核。"
                .to_string(),
        ],
        open_questions: Vec::new(),
    };
    normalize_role_specific_findings(role, &mut output.findings, &task_line);
    normalize_role_specific_recommendations(role, &mut output.recommendations, &task_line);
    while output.findings.len() < required_findings_for_role(role) {
        output.findings.push(default_role_finding(
            role,
            output.findings.len(),
            &task_line,
        ));
    }
    while output.recommendations.len() < required_recommendations_for_role(role) {
        output.recommendations.push(default_role_recommendation(
            role,
            output.recommendations.len(),
            &task_line,
        ));
    }
    if output.risks.is_empty() && role_requires_risk(role) {
        output.risks = default_role_risks(role);
    }
    normalize_role_specific_risks(role, &mut output.risks);
    normalize_upstream_evidence(task, &mut output);
    let content = format_role_work_output_content(&output);

    crate::core::protocol::WorkerReport {
        graph_id: assignment.graph_id.clone(),
        node_id: assignment.node_id.clone(),
        task_id: TaskId(task.id.clone()),
        worker_id: WorkerId(worker_id.0.clone()),
        role: assignment.role.clone(),
        content,
        evidence: output.evidence.clone(),
        risks: output.risks.clone(),
        open_questions: Vec::new(),
        role_output: Some(output),
        status: WorkerReportStatus::Completed,
    }
}

