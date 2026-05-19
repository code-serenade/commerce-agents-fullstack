use serde_json::{Map, Value, json};

pub(super) fn normalized_string(input: &Map<String, Value>, key: &str, fallback: &str) -> String {
    input
        .get(key)
        .and_then(Value::as_str)
        .map(str::trim)
        .filter(|value| !value.is_empty())
        .map(ToString::to_string)
        .unwrap_or_else(|| fallback.to_owned())
}

pub(super) fn normalized_i64(input: &Map<String, Value>, key: &str, fallback: i64) -> i64 {
    input.get(key).and_then(Value::as_i64).unwrap_or(fallback)
}

pub(super) fn normalized_f64(input: &Map<String, Value>, key: &str, fallback: f64) -> f64 {
    input.get(key).and_then(Value::as_f64).unwrap_or(fallback)
}

pub(super) fn normalized_string_vec(input: &Map<String, Value>, key: &str) -> Vec<String> {
    input
        .get(key)
        .and_then(Value::as_array)
        .map(|items| {
            items
                .iter()
                .filter_map(Value::as_str)
                .map(str::trim)
                .filter(|value| !value.is_empty())
                .map(ToString::to_string)
                .collect()
        })
        .unwrap_or_default()
}

pub(super) struct AidaScore {
    pub(super) weighted: f64,
    pub(super) hits: Value,
}

const AIDA_GROUPS: &[(&str, f64, &[&str])] = &[
    (
        "Attention",
        0.30,
        &[
            "限时", "爆款", "独家", "首发", "全网", "新品", "震撼", "惊喜", "超级",
        ],
    ),
    (
        "Interest",
        0.25,
        &[
            "功能",
            "特点",
            "原因",
            "为什么",
            "如何",
            "秘密",
            "揭秘",
            "技巧",
            "方法",
            "攻略",
        ],
    ),
    (
        "Desire",
        0.25,
        &[
            "完美",
            "必买",
            "值得",
            "推荐",
            "好用",
            "高品质",
            "升级",
            "加强",
            "专业",
            "极致",
        ],
    ),
    (
        "Action",
        0.20,
        &[
            "抢",
            "买",
            "下单",
            "立即",
            "马上",
            "赶快",
            "别错过",
            "收藏",
            "加购",
            "点击",
        ],
    ),
];

pub(super) fn score_aida(title: &str) -> AidaScore {
    let mut weighted = 0.0;
    let mut hits = serde_json::Map::new();
    for (name, weight, words) in AIDA_GROUPS {
        let matched = words
            .iter()
            .filter(|word| title.contains(**word))
            .cloned()
            .collect::<Vec<_>>();
        let raw = (0.5 + matched.len() as f64 * 0.25).min(1.0);
        weighted += raw * weight;
        if !matched.is_empty() {
            hits.insert((*name).to_owned(), json!(matched));
        }
    }
    AidaScore {
        weighted: (weighted * 1000.0).round() / 1000.0,
        hits: Value::Object(hits),
    }
}

pub(super) struct TriggerScore {
    pub(super) normalized: f64,
    pub(super) words: Vec<&'static str>,
}

const TRIGGER_GROUPS: &[(f64, &[&str])] = &[
    (
        1.8,
        &[
            "限时",
            "今天",
            "最后",
            "仅剩",
            "抢购",
            "秒杀",
            "即将",
            "截止",
            "倒计时",
            "马上",
        ],
    ),
    (
        1.6,
        &[
            "限量",
            "仅余",
            "稀缺",
            "孤品",
            "断码",
            "最后一批",
            "售完为止",
            "库存告急",
        ],
    ),
    (
        1.4,
        &[
            "爆款",
            "热卖",
            "畅销",
            "好评",
            "口碑",
            "网红",
            "明星",
            "万人",
            "口碑推荐",
            "人气",
        ],
    ),
    (
        1.3,
        &[
            "官方", "正品", "认证", "权威", "专业", "国家", "医院", "大牌", "品牌",
        ],
    ),
    (
        1.5,
        &[
            "超值",
            "划算",
            "省钱",
            "折扣",
            "优惠",
            "打折",
            "特价",
            "白菜价",
            "半价",
        ],
    ),
    (
        1.2,
        &[
            "惊喜", "感动", "完美", "心动", "必买", "真香", "安心", "放心", "幸福", "快乐",
        ],
    ),
];

pub(super) fn score_triggers(title: &str) -> TriggerScore {
    let mut raw: f64 = 0.0;
    let mut words = Vec::new();
    for (weight, group) in TRIGGER_GROUPS {
        for word in *group {
            if title.contains(word) {
                raw += weight;
                words.push(*word);
            }
        }
    }
    TriggerScore {
        normalized: (raw / 3.0).min(1.0),
        words,
    }
}

pub(super) fn ctr_grade(total: i64) -> &'static str {
    if total >= 85 {
        "S 高转化"
    } else if total >= 70 {
        "A 良好"
    } else if total >= 55 {
        "B 中等"
    } else if total >= 40 {
        "C 待改进"
    } else {
        "D 需重写"
    }
}
