fn round1(value: f64) -> f64 {
    (value * 10.0).round() / 10.0
}

fn round2(value: f64) -> f64 {
    (value * 100.0).round() / 100.0
}

fn optional_string_vec_param_or_empty(
    params: &Value,
    name: &str,
) -> Result<Vec<String>, SkillError> {
    Ok(optional_string_vec_param(params, name)?.unwrap_or_default())
}

fn nps_grade(nps: f64) -> &'static str {
    if nps >= 70.0 {
        "极优秀（世界级）"
    } else if nps >= 50.0 {
        "优秀"
    } else if nps >= 30.0 {
        "良好"
    } else if nps >= 0.0 {
        "正常"
    } else {
        "需改善（批评者过多）"
    }
}

fn csat_grade(csat: f64) -> &'static str {
    if csat >= 90.0 {
        "优秀"
    } else if csat >= 75.0 {
        "良好"
    } else {
        "需改善"
    }
}

struct SentimentScore {
    intensity: f64,
    urgency: &'static str,
    priority: i64,
    triggered: Vec<&'static str>,
}

fn score_sentiment(text: &str) -> SentimentScore {
    let keywords = [
        ("骗子", -3.0),
        ("欺诈", -3.0),
        ("举报", -2.8),
        ("曝光", -2.5),
        ("投诉到", -2.5),
        ("差评", -2.0),
        ("投诉", -2.0),
        ("维权", -2.0),
        ("退款", -1.5),
        ("破损", -1.5),
        ("质量差", -1.5),
        ("假货", -2.0),
        ("不满意", -1.2),
        ("等了好久", -1.0),
        ("还没到", -1.0),
        ("赔偿", -1.5),
        ("退货", -0.8),
        ("问题", -0.5),
        ("延误", -0.8),
        ("发错", -1.0),
        ("缺货", -0.8),
        ("好评", 0.8),
        ("满意", 0.8),
        ("感谢", 0.6),
        ("棒", 0.5),
        ("超快", 0.5),
        ("完美", 1.0),
        ("推荐", 0.8),
        ("再次购买", 1.0),
        ("愤怒", -0.9),
        ("气死", -0.9),
        ("太差了", -0.75),
        ("垃圾", -0.75),
        ("失望", -0.6),
        ("挺好", 0.3),
        ("不错", 0.3),
        ("很好", 0.45),
        ("太好了", 0.6),
    ];
    let mut score = 5.0;
    let mut triggered = Vec::new();
    for (keyword, delta) in keywords {
        if text.contains(keyword) {
            score += delta;
            triggered.push(keyword);
        }
    }
    if text.chars().count() > 200 && score < 4.0 {
        score -= 0.5;
    }
    let exclamations = text.matches('！').count() + text.matches("!!").count();
    if exclamations >= 2 && score < 5.0 {
        score -= 0.3 * exclamations.min(3) as f64;
    }
    let intensity = round1(score.clamp(0.0, 10.0));
    let (urgency, priority) = if intensity < 2.0 {
        ("高危-立即介入（2分钟内）", 1)
    } else if intensity < 3.5 {
        ("负面-优先处理（30分钟内）", 2)
    } else if intensity < 5.5 {
        ("中性-正常排队（2小时内）", 3)
    } else if intensity < 7.5 {
        ("正面-无需人工干预", 4)
    } else {
        ("高满意-可邀请好评", 5)
    };
    SentimentScore {
        intensity,
        urgency,
        priority,
        triggered: triggered.into_iter().take(5).collect(),
    }
}

fn sentiment_state(intensity: f64) -> &'static str {
    if intensity < 2.0 {
        "极度负面"
    } else if intensity < 4.0 {
        "负面"
    } else if intensity < 6.0 {
        "中性"
    } else if intensity < 8.0 {
        "正面"
    } else {
        "极度正面"
    }
}

fn sentiment_advice(intensity: f64) -> &'static str {
    if intensity < 2.0 {
        "立即人工介入，先承认问题并给出明确补偿或升级路径"
    } else if intensity < 4.0 {
        "优先响应，围绕事实核验、解决时限和补偿预期沟通"
    } else if intensity < 6.0 {
        "按常规SLA处理，保持信息透明"
    } else if intensity < 8.0 {
        "维持友好沟通，可引导收藏或复购"
    } else {
        "可邀请评价或沉淀为好评素材"
    }
}

fn truncate_chars(text: &str, limit: usize) -> String {
    text.chars().take(limit).collect()
}

fn parse_string_i64_map(value: Option<&Value>) -> Result<BTreeMap<String, i64>, SkillError> {
    let mut out = BTreeMap::new();
    let Some(value) = value else {
        return Ok(out);
    };
    let object = value
        .as_object()
        .ok_or_else(|| SkillError::invalid_parameter("complaint_distribution", "object"))?;
    for (key, value) in object {
        let count = value.as_i64().ok_or_else(|| {
            SkillError::invalid_parameter("complaint_distribution", "integer counts")
        })?;
        out.insert(key.clone(), count);
    }
    Ok(out)
}

fn nps_coefficient(category: &str) -> f64 {
    match category {
        "物流配送" => -0.35,
        "商品质量" => -0.48,
        "客服响应" => -0.22,
        "售后退款" => -0.30,
        "包装体验" => -0.12,
        "价格感知" => -0.18,
        "商品描述" => -0.20,
        "平台体验" => -0.10,
        _ => -0.15,
    }
}
