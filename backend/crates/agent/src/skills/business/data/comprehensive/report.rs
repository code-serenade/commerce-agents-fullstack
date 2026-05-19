use serde_json::Value;

use super::input::focus_instruction;
use crate::skills::data::common::value_string;

pub(super) struct DiagnosisReportDraft<'a> {
    pub(super) days: i64,
    pub(super) focus: &'a str,
    pub(super) score: i64,
    pub(super) grade: &'a str,
    pub(super) findings: &'a [String],
    pub(super) risks: &'a [String],
    pub(super) opportunities: &'a [String],
    pub(super) actions: &'a [Value],
}

pub(super) fn build_diagnosis_report(draft: DiagnosisReportDraft<'_>) -> String {
    let mut report = format!(
        "【诊断基于{}天数据】\n整体健康评分：{}分（{}）。\n分析重点：{}。\n\n",
        draft.days,
        draft.score,
        draft.grade,
        focus_instruction(draft.focus)
    );
    report.push_str("核心发现：\n");
    for finding in draft.findings.iter().take(5) {
        report.push_str(&format!("- {finding}\n"));
    }
    report.push_str("\n风险预警：\n");
    for risk in draft.risks.iter().take(3) {
        report.push_str(&format!("- {risk}\n"));
    }
    report.push_str("\n增长机会：\n");
    for opportunity in draft.opportunities.iter().take(3) {
        report.push_str(&format!("- {opportunity}\n"));
    }
    report.push_str("\n7天行动计划：\n");
    for action in draft.actions.iter().take(5) {
        report.push_str(&format!(
            "- {}：{}\n",
            value_string(action, "优先级", "P?"),
            value_string(action, "动作", "待补充动作")
        ));
    }
    report
}
