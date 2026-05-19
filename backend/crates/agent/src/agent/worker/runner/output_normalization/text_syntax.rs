use crate::core::protocol::RoleWorkOutput;

pub(super) fn normalize_user_facing_text(output: &mut RoleWorkOutput) {
    output.summary = normalize_text(&output.summary);
    normalize_items(&mut output.findings);
    normalize_items(&mut output.recommendations);
    normalize_items(&mut output.evidence);
    normalize_items(&mut output.risks);
    normalize_items(&mut output.open_questions);
}

fn normalize_items(items: &mut [String]) {
    for item in items {
        *item = normalize_text(item);
    }
}

fn normalize_text(value: &str) -> String {
    let mut text = value.replace('\r', " ");

    for (from, to) in [
        ("$\\rightarrow$", "→"),
        ("$ ightarrow$", "→"),
        ("$ightarrow$", "→"),
        ("\\rightarrow", "→"),
        ("$\\to$", "→"),
        ("\\to", "→"),
        ("$\\leftarrow$", "←"),
        ("$ leftarrow$", "←"),
        ("$leftarrow$", "←"),
        ("\\leftarrow", "←"),
        ("$\\Rightarrow$", "⇒"),
        ("$ Rightarrow$", "⇒"),
        ("$Rightarrow$", "⇒"),
        ("\\Rightarrow", "⇒"),
        ("$\\Leftarrow$", "⇐"),
        ("$ Leftarrow$", "⇐"),
        ("$Leftarrow$", "⇐"),
        ("\\Leftarrow", "⇐"),
    ] {
        text = text.replace(from, to);
    }

    compact_arrow_spacing(text)
}

fn compact_arrow_spacing(mut text: String) -> String {
    for arrow in ["→", "←", "⇒", "⇐"] {
        for from in [
            format!("  {arrow}  "),
            format!(" {arrow}  "),
            format!("  {arrow} "),
        ] {
            text = text.replace(&from, &format!(" {arrow} "));
        }
    }
    text.trim().to_string()
}

#[cfg(test)]
mod tests {
    use super::{normalize_text, normalize_user_facing_text};
    use crate::core::protocol::RoleWorkOutput;

    #[test]
    fn repairs_json_decoded_latex_arrow() {
        let text = "流量导入 $\rightarrow$ 高效转化";

        assert_eq!(normalize_text(text), "流量导入 → 高效转化");
    }

    #[test]
    fn repairs_visible_broken_latex_arrow() {
        let text = "数据指标 $ ightarrow$ 内容创意 $ ightarrow$ 视觉承接";

        assert_eq!(normalize_text(text), "数据指标 → 内容创意 → 视觉承接");
    }

    #[test]
    fn keeps_normal_text_unchanged() {
        let text = "当前方案进入产品特有级执行。";

        assert_eq!(normalize_text(text), text);
    }

    #[test]
    fn normalizes_all_user_facing_role_output_fields() {
        let mut output = RoleWorkOutput {
            summary: "流量导入 $\rightarrow$ 高效转化".to_string(),
            findings: vec!["复购 $ ightarrow$ 私域".to_string()],
            recommendations: vec!["曝光 \\rightarrow 转化".to_string()],
            evidence: vec!["任务输入".to_string()],
            risks: vec!["预算 $\\to$ 消耗需要复核".to_string()],
            open_questions: vec!["是否需要从预热 \\Rightarrow 爆发？".to_string()],
        };

        normalize_user_facing_text(&mut output);

        assert_eq!(output.summary, "流量导入 → 高效转化");
        assert_eq!(output.findings, vec!["复购 → 私域"]);
        assert_eq!(output.recommendations, vec!["曝光 → 转化"]);
        assert_eq!(output.evidence, vec!["任务输入"]);
        assert_eq!(output.risks, vec!["预算 → 消耗需要复核"]);
        assert_eq!(output.open_questions, vec!["是否需要从预热 ⇒ 爆发？"]);
    }
}
