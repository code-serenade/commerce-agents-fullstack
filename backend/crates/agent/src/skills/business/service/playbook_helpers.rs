use super::*;

pub(super) fn push_unique_non_empty(out: &mut Vec<String>, value: String) {
    let value = value.trim();
    if !value.is_empty() && !out.iter().any(|item| item == value) {
        out.push(value.to_owned());
    }
}

pub(super) fn eq_ignore_ascii_case_trimmed(left: &str, right: &str) -> bool {
    left.trim().eq_ignore_ascii_case(right.trim())
}

pub(super) fn text_contains_case_insensitive(text: &str, needle: &str) -> bool {
    let needle = needle.trim();
    !needle.is_empty() && text.to_lowercase().contains(&needle.to_lowercase())
}

pub(super) fn infer_customer_emotion(text: &str) -> &'static str {
    if contains_any_local(
        text,
        &[
            "威胁投诉",
            "投诉",
            "曝光",
            "举报",
            "维权",
            "骗子",
            "垃圾",
            "气死",
            "差评",
        ],
    ) {
        "威胁投诉/愤怒"
    } else if contains_any_local(text, &["着急", "急", "催", "怎么还", "等了", "马上"]) {
        "焦虑"
    } else if contains_any_local(text, &["谢谢", "满意", "好评", "不错"]) {
        "满意"
    } else if contains_any_local(text, &["不满", "失望", "太差", "不满意"]) {
        "不满"
    } else {
        "普通"
    }
}

pub(super) fn service_priority(
    issue_type: &str,
    customer_emotion: &str,
    customer_message: &str,
) -> &'static str {
    if contains_any_local(
        &format!("{issue_type}\n{customer_emotion}\n{customer_message}"),
        &[
            "威胁投诉",
            "投诉",
            "曝光",
            "举报",
            "维权",
            "骗子",
            "假货",
            "平台介入",
        ],
    ) {
        "P0"
    } else if contains_any_local(
        &format!("{issue_type}\n{customer_emotion}\n{customer_message}"),
        &[
            "质量", "破损", "发错", "退款", "退货", "换货", "赔偿", "愤怒", "不满",
        ],
    ) {
        "P1"
    } else {
        "P2"
    }
}

pub(super) fn service_sla(priority: &str) -> &'static str {
    match priority {
        "P0" => "2分钟",
        "P1" => "30分钟",
        _ => "2小时",
    }
}

pub(super) fn is_seller_fault_issue(text: &str) -> bool {
    contains_any_local(
        text,
        &["质量", "发错", "破损", "少发", "漏发", "假货", "描述不符"],
    )
}

pub(super) fn ticket_first_action(issue_type: &str) -> &'static str {
    if contains_any_local(issue_type, &["物流", "延迟", "未到", "快递"]) {
        "先核实物流节点和预计送达时间，必要时同步催派或补发方案"
    } else if contains_any_local(issue_type, &["质量", "破损", "发错"]) {
        "先收集凭证并判断卖家责任，给出退换货、补发或补偿路径"
    } else if contains_any_local(issue_type, &["退款", "退货", "换货", "售后"]) {
        "先确认收货天数、商品状态和责任归属，再明确运费承担与退款时效"
    } else if contains_any_local(issue_type, &["投诉", "差评", "维权"]) {
        "先升级主管关注，避免一线反复解释导致情绪升级"
    } else {
        "先确认订单和商品信息，再按客户诉求给出下一步"
    }
}

pub(super) fn ticket_solution(issue_type: &str, seller_fault: bool) -> String {
    if contains_any_local(issue_type, &["物流", "延迟", "未到", "快递"]) {
        "我会马上核实物流轨迹并帮您催派；如果超过承诺时效，会同步补偿或补发可选方案。".to_owned()
    } else if contains_any_local(issue_type, &["退款", "退货", "换货", "售后"]) {
        let freight = if seller_fault {
            "运费由我们承担"
        } else {
            "运费按店铺/平台规则确认"
        };
        format!(
            "我先帮您登记售后申请，确认商品状态后给出退换货路径，{freight}，退款进度会持续同步。"
        )
    } else if seller_fault {
        "如果核实确属我们发错、破损或质量问题，会优先给您补发/退换，并同步可执行补偿方案。"
            .to_owned()
    } else {
        "我会先核实订单、商品和沟通记录，给您一个明确处理结论；涉及特殊政策的部分会提交主管复核。"
            .to_owned()
    }
}

pub(super) fn ticket_empathy_script(customer_emotion: &str) -> &'static str {
    if contains_any_local(customer_emotion, &["威胁投诉", "愤怒"]) {
        "确实让您有很差的体验了，这个问题我先帮您推进处理，不让您反复解释。"
    } else if contains_any_local(customer_emotion, &["焦虑", "不满"]) {
        "理解您现在比较着急，我会把处理进度和时间点说清楚，避免您一直等待。"
    } else if contains_any_local(customer_emotion, &["满意"]) {
        "感谢您的理解，我们会继续把后续服务处理完整。"
    } else {
        "收到，我会按订单情况帮您核实，并尽快给出明确答复。"
    }
}

pub(super) fn faq_answer(question: &str, platform: &str, policies: &Map<String, Value>) -> Value {
    let policy_text = |keys: &[&str]| {
        keys.iter()
            .find_map(|key| {
                policies
                    .get(*key)
                    .and_then(Value::as_str)
                    .map(ToString::to_string)
            })
            .unwrap_or_default()
    };
    let q = question.trim();
    let (short, detail, guide) = if contains_any_local(q, &["发货", "多久", "时效"]) {
        let policy = policy_text(&["发货时效", "发货时间", "delivery"]);
        let detail = if policy.is_empty() {
            format!(
                "正常会按{platform}店铺承诺时效发出；如遇大促、预售或缺货，\
                 会以订单页和客服确认为准。"
            )
        } else {
            format!("本店当前发货规则：{policy}。如订单异常，会主动同步进度。")
        };
        (
            "正常按店铺承诺时效发货，异常会同步处理。",
            detail,
            "查询订单状态；预售/缺货/大促订单需单独标记并同步预计时间。",
        )
    } else if contains_any_local(q, &["退", "换", "退款", "售后"]) {
        let policy = policy_text(&["退货政策", "售后政策", "return"]);
        let detail = if policy.is_empty() {
            "支持按平台和店铺规则申请售后；质量、发错、破损等卖家责任问题优先处理，\
             非质量原因按商品状态确认。"
                .to_owned()
        } else {
            format!("售后规则：{policy}。具体责任和运费以订单、凭证和平台规则复核为准。")
        };
        (
            "可以按规则申请售后，我们会协助处理。",
            detail,
            "先确认收货天数、商品状态、责任方和凭证，再选择退货、换货、补发或补偿。",
        )
    } else if contains_any_local(q, &["质量", "正品", "保障", "假"]) {
        (
            "商品会按页面承诺和店铺标准发出。",
            "如收到后发现破损、发错、明显质量异常，请保留外包装、商品照片和订单号，\
             我们会优先核实处理。"
                .to_owned(),
            "质量类问题需要凭证留存；疑似批量问题要升级质检和商品负责人。",
        )
    } else if contains_any_local(q, &["优惠", "活动", "价格", "保价"]) {
        (
            "优惠以当前商品页和活动页为准。",
            "您可以先领取店铺券/平台券后再下单；价格保护、活动补差等需要按平台当期规则核实。"
                .to_owned(),
            "价格类问题不要承诺固定补差，先核对活动规则和订单时间。",
        )
    } else if contains_any_local(q, &["尺码", "规格", "颜色", "怎么选"]) {
        (
            "可以按页面规格说明选择，不确定可以发使用场景给客服。",
            "建议结合商品规格、使用场景和个人偏好选择；如果您告诉我们身高体重/设备型号/使用需求，\
             可以进一步建议。"
                .to_owned(),
            "规格建议不能替代用户最终选择；高退货风险规格要提示确认。",
        )
    } else if contains_any_local(q, &["包邮", "运费"]) {
        (
            "运费以订单结算页显示为准。",
            "不同地区、活动和商品组合可能影响运费；请以下单页展示为准，特殊地区可能有额外运费。"
                .to_owned(),
            "不要口头承诺所有地区包邮，先引导用户查看结算页。",
        )
    } else if contains_any_local(q, &["发票", "开票"]) {
        (
            "支持情况以店铺开票规则为准。",
            "如需发票，请提供抬头、税号和开票信息；电子或纸质发票类型需按店铺配置确认。".to_owned(),
            "开票主体、类目和时效需要人工确认后回复。",
        )
    } else {
        (
            "这个问题我可以先帮您核实。",
            "请您补充订单号、商品规格或具体诉求，我们会按订单情况给出明确处理方案。".to_owned(),
            "无法判断的问题进入人工工单，不要编造商品参数或平台政策。",
        )
    };

    json!({
        "问题": q,
        "简短版": short,
        "详细版": detail,
        "处理指引": guide
    })
}

pub(super) fn escalation_severity(
    issue_description: &str,
    customer_emotion: &str,
    previous_contacts: i64,
) -> &'static str {
    let text = format!("{issue_description}\n{customer_emotion}");
    if contains_any_local(
        &text,
        &["报警", "媒体", "曝光", "监管", "诉讼", "假货", "人身"],
    ) {
        "S1"
    } else if contains_any_local(&text, &["威胁投诉", "投诉", "维权", "平台介入", "赔偿"])
        || previous_contacts >= 3
    {
        "S2"
    } else if contains_any_local(&text, &["愤怒", "差评", "退款", "破损", "质量", "发错"])
        || previous_contacts >= 2
    {
        "S3"
    } else {
        "S4"
    }
}

pub(super) fn escalation_path(severity: &str) -> Vec<&'static str> {
    match severity {
        "S1" => vec![
            "一线客服立即停止争辩并记录完整事实",
            "客服主管10分钟内接管",
            "法务/平台运营/商品负责人同步复核",
            "形成书面处理说明和证据包",
        ],
        "S2" => vec![
            "一线客服先安抚并承诺明确回访时间",
            "客服主管30分钟内复核责任和补偿方案",
            "需要金额动作时提交审批",
            "处理后回填纠纷根因",
        ],
        "S3" => vec![
            "一线客服核实订单和凭证",
            "按SOP给出退换/补发/解释方案",
            "2小时内回访结果",
            "未解决再升级主管",
        ],
        _ => vec![
            "一线客服常规处理",
            "记录客户诉求和处理结论",
            "必要时同步商品/物流信息",
        ],
    }
}

pub(super) fn escalation_manager_script(severity: &str) -> &'static str {
    match severity {
        "S1" => {
            "您好，我是本店客服主管。这个问题我已经接管，会先保护您的权益并核实全部证据，\
             接下来由我直接同步处理进展。"
        }
        "S2" => {
            "您好，我是客服主管。看到您已经多次沟通仍未解决，我先向您道歉，\
             我会在约定时间内给出明确处理方案。"
        }
        "S3" => "您好，我来协助跟进这个售后问题，会先核实凭证和订单情况，再给您同步可执行方案。",
        _ => "您好，我会帮您确认当前问题并推动处理，有进展会及时告知您。",
    }
}

pub(super) fn compensation_suggestion(severity: &str) -> Value {
    match severity {
        "S1" => json!({
            "建议": "暂停一线承诺，主管/法务/平台运营复核后给最终方案",
            "可选动作": ["全额退款", "补发", "专项补偿", "平台申诉协助"],
            "审批要求": "必须审批"
        }),
        "S2" => json!({
            "建议": "优先给出可执行补救方案，金额补偿走审批",
            "可选动作": ["退换货优先处理", "补发", "优惠券补偿", "运费补贴"],
            "审批要求": "金额或平台动作需审批"
        }),
        "S3" => json!({
            "建议": "按SOP处理，必要时给低风险安抚补偿",
            "可选动作": ["售后登记", "物流催派", "退换货协助"],
            "审批要求": "超SOP补偿需复核"
        }),
        _ => json!({
            "建议": "常规处理，不主动承诺赔付",
            "可选动作": ["解释规则", "协助查询", "引导补充信息"],
            "审批要求": "通常无需审批"
        }),
    }
}

pub(super) fn contains_any_local(text: &str, needles: &[&str]) -> bool {
    let lower = text.to_ascii_lowercase();
    needles
        .iter()
        .any(|needle| text.contains(needle) || lower.contains(&needle.to_ascii_lowercase()))
}
