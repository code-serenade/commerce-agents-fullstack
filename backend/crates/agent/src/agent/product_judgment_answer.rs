#[derive(Debug, Clone)]
struct ProductJudgmentFocus {
    subject: Option<String>,
    price: Option<String>,
    audience: Option<String>,
    problem: Option<String>,
}

pub(crate) fn repair_product_judgment_answer(task: &str, answer: &str) -> Option<String> {
    if !task_requires_product_judgment(task) || product_judgment_answer_is_acceptable(task, answer)
    {
        return None;
    }
    format_product_judgment_fallback(task)
}

pub(crate) fn format_product_judgment_fallback(task: &str) -> Option<String> {
    if !task_requires_product_judgment(task) {
        return None;
    }
    let focus = product_judgment_focus(task);
    let subject = focus.subject.as_deref().unwrap_or("这款商品");
    let price = focus.price.as_deref().unwrap_or("当前价格");
    let audience = focus.audience.as_deref().unwrap_or("目标用户");
    let problem = focus.problem.as_deref().unwrap_or("页面入口表达还不够清楚");
    let entry = if problem.contains("主图") {
        "主图"
    } else if problem.contains("标题") {
        "标题"
    } else {
        "入口表达"
    };
    let concerns = product_judgment_concerns(subject);

    Some(format!(
        "## 先说判断\n\n\
         你担心判断不准是对的：现在缺少真实销量和转化反馈，不能把结论说死。我的判断是可以做，但先按小测处理，不建议直接重投入。{subject}面向{audience}，{price}能不能站得住，关键看{problem}改完后，用户是否愿意点击、咨询、收藏或加购；本轮只做低风险判断，不编造销量数据。\n\n\
         ## 三个优化方向\n\n\
         - {entry}首屏先改：把{audience}最容易理解的使用场景和核心利益点放到第一眼位置，让人一眼知道这是{subject}、适合谁、解决什么顾虑。\n\
         - 标题和卖点重排：围绕{price}写清楚真实卖点、适用场景、{proof_concerns}或待核验证据；没有证据的功效、销量和权威背书先不要写。\n\
         - 小范围验证反馈：记录点击、咨询、收藏/加购、未买原因、{hesitation_concerns}；如果主图和标题改完仍没有有效反馈，就先调整表达或暂缓放大。\n\n\
         ## 还要注意\n\n\
         - 放大前要补齐真实成本、库存、履约、售后和平台规则；这些没确认前，不把\"能做\"写成确定销售结果。\n\
         - 下一步最值得先做的是重做一版主图首屏和标题卖点，再用真实反馈判断是否继续。",
        proof_concerns = concerns.proof,
        hesitation_concerns = concerns.hesitation,
    ))
}

#[derive(Debug, Clone, Copy)]
struct ProductJudgmentConcerns {
    proof: &'static str,
    hesitation: &'static str,
}

fn product_judgment_concerns(subject: &str) -> ProductJudgmentConcerns {
    if contains_any(subject, &["收纳", "置物", "整理", "宿舍", "桌面"]) {
        return ProductJudgmentConcerns {
            proof: "尺寸、容量、材质、承重和桌面使用场景",
            hesitation: "尺寸不合适、容量不够、材质廉价、摆放不稳或售后顾虑",
        };
    }
    if contains_any(subject, &["防晒", "衣", "服", "鞋", "包", "穿搭", "面料"]) {
        return ProductJudgmentConcerns {
            proof: "尺码、面料、版型、穿着场景和功能证明",
            hesitation: "尺码不合适、面料质感、功能不达预期或售后顾虑",
        };
    }
    ProductJudgmentConcerns {
        proof: "规格、材质、使用场景、真实反馈和效果依据",
        hesitation: "规格不合适、质量不稳、使用效果不清楚或售后顾虑",
    }
}

fn task_requires_product_judgment(task: &str) -> bool {
    let task = task.trim();
    if task.is_empty() {
        return false;
    }
    let asks_judgment = [
        "能不能做",
        "能做吗",
        "可不可以做",
        "可以做吗",
        "值不值得做",
        "值不值",
        "是否值得",
        "是否适合做",
        "先判断",
        "判断能不能",
        "做不做",
        "要不要做",
    ]
    .iter()
    .any(|marker| task.contains(marker));
    if !asks_judgment {
        return false;
    }

    let focus = product_judgment_focus(task);
    focus.subject.is_some()
        || [
            "商品",
            "产品",
            "卖一款",
            "价格",
            "售价",
            "定价",
            "面向",
            "人群",
            "主图",
            "标题",
        ]
        .iter()
        .any(|marker| task.contains(marker))
}

fn contains_any(text: &str, markers: &[&str]) -> bool {
    markers.iter().any(|marker| text.contains(marker))
}

fn product_judgment_answer_is_acceptable(task: &str, answer: &str) -> bool {
    let answer = answer.trim();
    if answer.is_empty() {
        return false;
    }
    if [
        "先明确分析对象",
        "目标人群、目标动作",
        "当前需解决",
        "先确定活动目标",
        "可观察字段",
        "只给方向框架",
        "第一步",
        "第二步",
        "第三步",
    ]
    .iter()
    .any(|marker| answer.contains(marker))
    {
        return false;
    }

    let focus = product_judgment_focus(task);
    let compact_answer = compact_text(answer);
    let anchors_present = [
        focus.subject.as_deref(),
        focus.price.as_deref(),
        focus.audience.as_deref(),
        focus.problem.as_deref(),
    ]
    .into_iter()
    .flatten()
    .all(|value| compact_answer.contains(&compact_text(value)));
    if !anchors_present {
        return false;
    }

    let has_judgment = [
        "可以做",
        "能做",
        "可以先做",
        "先小测",
        "值得小测",
        "不建议直接",
        "暂缓",
        "不适合",
    ]
    .iter()
    .any(|marker| answer.contains(marker));
    let has_three_steps = answer.contains("三步") || answer.contains("三个优化方向");
    let has_business_actions = answer.contains("主图")
        && (answer.contains("标题") || answer.contains("卖点") || answer.contains("价格"))
        && (answer.contains("验证") || answer.contains("点击") || answer.contains("咨询"));

    has_judgment && has_three_steps && has_business_actions
}

fn product_judgment_focus(task: &str) -> ProductJudgmentFocus {
    ProductJudgmentFocus {
        subject: extract_task_value(
            task,
            &["一款", "这个", "该"],
            &[
                "，",
                "。",
                ",",
                ".",
                "价格",
                "售价",
                "售卖价",
                "定价",
                "面向",
                "人群",
                "主图",
                "标题",
                "请",
                "怎么",
                "能不能",
                "是否",
            ],
            28,
        )
        .filter(|value| !matches!(value.as_str(), "商品" | "产品" | "东西")),
        price: extract_task_value(
            task,
            &["价格", "售价", "定价"],
            &["，", "。", ",", ".", "面向", "人群", "主图", "标题", "请"],
            16,
        ),
        audience: extract_task_value(
            task,
            &["面向", "给", "针对"],
            &[
                "，", "。", ",", ".", "价格", "售价", "定价", "主图", "标题", "请",
            ],
            24,
        ),
        problem: product_judgment_problem(task),
    }
}

fn product_judgment_problem(task: &str) -> Option<String> {
    if task.contains("主图") && task.contains("普通") {
        return Some("主图比较普通".to_string());
    }
    if task.contains("主图") {
        return Some("主图需要优化".to_string());
    }
    if task.contains("标题") && (task.contains("点击率低") || task.contains("低")) {
        return Some("标题点击率低".to_string());
    }
    if task.contains("标题") {
        return Some("标题需要优化".to_string());
    }
    if task.contains("价格") || task.contains("定价") || task.contains("售价") {
        return Some("价格和卖点需要核对".to_string());
    }
    None
}

fn extract_task_value(
    task: &str,
    markers: &[&str],
    stops: &[&str],
    max_chars: usize,
) -> Option<String> {
    markers
        .iter()
        .find_map(|marker| {
            let start = task.find(marker)? + marker.len();
            let tail = &task[start..];
            let end = stops
                .iter()
                .filter_map(|stop| tail.find(stop))
                .min()
                .unwrap_or(tail.len());
            normalize_task_value(&tail[..end], max_chars)
        })
        .filter(|value| !value.is_empty())
}

fn normalize_task_value(value: &str, max_chars: usize) -> Option<String> {
    let mut cleaned = value
        .trim()
        .trim_matches(|ch: char| {
            matches!(
                ch,
                '：' | ':' | '是' | '的' | '、' | '，' | ',' | '。' | '.' | ' ' | '\n' | '\r'
            )
        })
        .chars()
        .take(max_chars)
        .collect::<String>();
    for prefix in ["一款", "一个", "这款", "这个"] {
        if let Some(rest) = cleaned.strip_prefix(prefix) {
            cleaned = rest.trim().to_string();
            break;
        }
    }
    (!cleaned.trim().is_empty()).then(|| cleaned.trim().to_string())
}

fn compact_text(value: &str) -> String {
    value.split_whitespace().collect::<String>()
}

#[cfg(test)]
mod tests {
    use super::{
        format_product_judgment_fallback, product_judgment_answer_is_acceptable,
        repair_product_judgment_answer, task_requires_product_judgment,
    };

    const SUNSCREEN_TASK: &str = "我想卖一款夏季防晒衣，价格129元，面向大学生，主图比较普通。请像平等伙伴一样先判断能不能做，并给出三步优化方向，不要编造销量数据。";

    #[test]
    fn formats_product_judgment_with_required_business_anchors() {
        let output = format_product_judgment_fallback(SUNSCREEN_TASK).expect("fallback");

        assert!(output.contains("可以做"), "{output}");
        assert!(output.contains("先按小测处理"), "{output}");
        assert!(output.contains("不建议直接重投入"), "{output}");
        assert!(output.contains("夏季防晒衣"), "{output}");
        assert!(output.contains("129元"), "{output}");
        assert!(output.contains("大学生"), "{output}");
        assert!(output.contains("主图比较普通"), "{output}");
        assert!(output.contains("你担心判断不准是对的"), "{output}");
        assert!(output.contains("## 三个优化方向"), "{output}");
        assert!(output.contains("主图首屏先改"), "{output}");
        assert!(output.contains("标题和卖点重排"), "{output}");
        assert!(output.contains("小范围验证反馈"), "{output}");
        assert!(output.contains("标题"), "{output}");
        assert!(output.contains("不编造销量数据"), "{output}");
        assert!(!output.contains("先明确分析对象"), "{output}");
        assert!(!output.contains("第一步"), "{output}");
        assert!(!output.contains("第二步"), "{output}");
        assert!(!output.contains("第三步"), "{output}");
        assert!(!output.contains("worker"), "{output}");
    }

    #[test]
    fn formats_storage_box_judgment_without_apparel_template_terms() {
        let task = "我想卖一款学生宿舍用桌面收纳盒，售价39元，只有普通白底图，没有销量数据。请像平等伙伴一样先判断能不能小测，再给主图、标题、验证数据三个方向，不要联网，不要编造销量。";

        let output = format_product_judgment_fallback(task).expect("fallback");

        assert!(output.contains("可以做"), "{output}");
        assert!(output.contains("桌面收纳盒"), "{output}");
        assert!(output.contains("39元"), "{output}");
        assert!(output.contains("不编造销量数据"), "{output}");
        assert!(output.contains("你担心判断不准是对的"), "{output}");
        assert!(output.contains("## 三个优化方向"), "{output}");
        assert!(output.contains("尺寸"), "{output}");
        assert!(output.contains("容量"), "{output}");
        assert!(output.contains("材质"), "{output}");
        assert!(!output.contains("防晒"), "{output}");
        assert!(!output.contains("尺码/面料"), "{output}");
        assert!(!output.contains("退款或尺码面料顾虑"), "{output}");
    }

    #[test]
    fn repairs_generic_product_judgment_answer() {
        let generic = "先给你一版可用判断。先明确分析对象、目标人群、目标动作和当前可用信息。把需要验证的判断拆成可观察字段。";

        let output = repair_product_judgment_answer(SUNSCREEN_TASK, generic).expect("repair");

        assert!(output.contains("夏季防晒衣"), "{output}");
        assert!(output.contains("129元"), "{output}");
        assert!(output.contains("大学生"), "{output}");
        assert!(output.contains("主图比较普通"), "{output}");
        assert!(output.contains("## 三个优化方向"), "{output}");
        assert!(!output.contains("第一步"), "{output}");
    }

    #[test]
    fn keeps_partner_style_answer_that_already_satisfies_product_judgment() {
        let answer = "可以做，但先小测，不建议直接重投入。夏季防晒衣面向大学生，129元要靠主图比较普通的问题先改掉。\n\n## 三个优化方向\n\n- 主图首屏先改，突出真实使用场景。\n- 标题和卖点重排，说明价格为什么站得住。\n- 小范围验证反馈，用点击、咨询和加购验证，不编造销量数据。";

        assert!(product_judgment_answer_is_acceptable(
            SUNSCREEN_TASK,
            answer
        ));
        assert!(repair_product_judgment_answer(SUNSCREEN_TASK, answer).is_none());
    }

    #[test]
    fn repairs_mechanical_step_product_judgment_answer() {
        let answer = "可以做，但先小测，不建议直接重投入。夏季防晒衣面向大学生，129元要靠主图比较普通的问题先改掉。\n\n## 三步优化方向\n\n- 第一步，先改主图。\n- 第二步，重排标题和卖点。\n- 第三步，用点击、咨询和加购验证，不编造销量数据。";

        let repaired = repair_product_judgment_answer(SUNSCREEN_TASK, answer).expect("repair");

        assert!(repaired.contains("你担心判断不准是对的"), "{repaired}");
        assert!(repaired.contains("## 三个优化方向"), "{repaired}");
        assert!(!repaired.contains("第一步"), "{repaired}");
        assert!(!repaired.contains("第二步"), "{repaired}");
        assert!(!repaired.contains("第三步"), "{repaired}");
    }

    #[test]
    fn does_not_trigger_for_non_judgment_task() {
        assert!(!task_requires_product_judgment("帮我写一版夏季防晒衣标题"));
        assert!(format_product_judgment_fallback("帮我写一版夏季防晒衣标题").is_none());
    }
}
