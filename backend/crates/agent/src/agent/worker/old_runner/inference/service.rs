fn infer_return_reason(text: &str) -> &'static str {
    if text.contains("质量") {
        "质量问题"
    } else if text.contains("发错") {
        "发错货"
    } else if text.contains("破损") {
        "破损"
    } else if text.contains("尺码") || text.contains("大小") {
        "尺码不合"
    } else {
        "不喜欢"
    }
}

fn infer_service_issue_type(text: &str) -> &'static str {
    policy_match_label(text, "service.issue_type").unwrap_or("售后咨询")
}

fn infer_service_emotion(text: &str) -> &'static str {
    policy_match_label(text, "service.emotion").unwrap_or("普通")
}

fn infer_faq_questions(text: &str) -> Vec<String> {
    let mut questions = Vec::new();
    if contains_any(text, &["发货", "物流", "时效"]) {
        questions.push("什么时候发货？".to_owned());
    }
    if contains_any(text, &["退", "换", "退款", "售后"]) {
        questions.push("支持退货退款吗？怎么操作？".to_owned());
    }
    if contains_any(text, &["质量", "正品"]) {
        questions.push("质量有保障吗？".to_owned());
    }
    if contains_any(text, &["优惠", "价格", "活动"]) {
        questions.push("有优惠活动吗？".to_owned());
    }
    if contains_any(text, &["尺码", "规格"]) {
        questions.push("怎么选尺码/规格？".to_owned());
    }
    if questions.is_empty() {
        questions.extend(
            [
                "什么时候发货？",
                "支持退货退款吗？怎么操作？",
                "质量有保障吗？",
                "有优惠活动吗？",
                "怎么选尺码/规格？",
            ]
            .iter()
            .map(|value| (*value).to_owned()),
        );
    }
    questions
}
