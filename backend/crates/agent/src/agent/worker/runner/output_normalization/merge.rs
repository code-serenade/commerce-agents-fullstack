use super::*;

pub(super) fn merge_skill_results(
    mut parsed: ParsedWorkerRoleOutput,
    results: &[SkillExecutionResult],
    task: &Task,
) -> ParsedWorkerRoleOutput {
    if results.is_empty() {
        return parsed;
    }

    let role_output = parsed
        .role_output
        .get_or_insert_with(crate::core::protocol::RoleWorkOutput::default);

    for result in results {
        match result.status {
            SkillExecutionStatus::Success => {
                let evidence = format_success_skill_evidence(result);
                push_unique(&mut parsed.evidence, evidence.clone());
                push_unique(&mut role_output.evidence, evidence);
            }
            SkillExecutionStatus::Deferred | SkillExecutionStatus::PendingApproval => {
                let is_non_blocking_gap = is_non_blocking_category_skill_gap(result, task);
                let risk = if is_non_blocking_gap {
                    format_category_level_skill_boundary(result)
                } else {
                    format_incomplete_skill_risk(result)
                };
                let question = format_incomplete_skill_question(result);
                push_unique(&mut parsed.risks, risk.clone());
                if !is_non_blocking_gap {
                    push_unique(&mut parsed.open_questions, question.clone());
                }
                push_unique(&mut role_output.risks, risk);
                if is_non_blocking_gap {
                    push_unique(
                        &mut role_output.evidence,
                        CATEGORY_LEVEL_BOUNDARY_MARKER.to_string(),
                    );
                    push_unique(
                        &mut role_output.evidence,
                        format!(
                            "本轮返回缺数据或待适配状态；由于用户询问类目级问题且未选择预设商品，\
                             按数据边界降级展示，不阻塞岗位产物。能力边界：{}。",
                            category_skill_boundary_label(result.skill_id.as_str())
                        ),
                    );
                } else {
                    push_unique(&mut role_output.open_questions, question);
                }
            }
            SkillExecutionStatus::NotFound
            | SkillExecutionStatus::ValidationError
            | SkillExecutionStatus::Failed => {
                let is_non_blocking_gap = is_non_blocking_category_skill_gap(result, task);
                let risk = if is_non_blocking_gap {
                    format_category_level_skill_boundary(result)
                } else {
                    format_failed_skill_risk(result)
                };
                push_unique(&mut parsed.risks, risk.clone());
                if !is_non_blocking_gap {
                    push_unique(&mut parsed.open_questions, risk.clone());
                }
                push_unique(&mut role_output.risks, risk.clone());
                if is_non_blocking_gap {
                    push_unique(
                        &mut role_output.evidence,
                        CATEGORY_LEVEL_BOUNDARY_MARKER.to_string(),
                    );
                    push_unique(
                        &mut role_output.evidence,
                        format!(
                            "本轮有能力未形成可用结果；由于这是未选商品的类目级问题，\
                             该缺口只作为数据边界，不要求用户先指定预设商品。能力边界：{}。",
                            category_skill_boundary_label(result.skill_id.as_str())
                        ),
                    );
                } else {
                    push_unique(&mut role_output.open_questions, risk);
                }
            }
        }
    }

    parsed.content = parsed
        .role_output
        .as_ref()
        .map(format_role_work_output_content)
        .filter(|content| !content.trim().is_empty())
        .unwrap_or(parsed.content);
    parsed
}

