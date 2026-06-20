pub(crate) const MIN_FORCED_RICH_DELIVERY_CHARS: usize = 900;

pub(crate) fn text_requires_rich_delivery(value: &str) -> bool {
    [
        "不补资料",
        "不再补资料",
        "不会提供任何其他信息",
        "不会给你提供任何其他信息",
        "不提供任何其他信息",
        "不要让我再提供资料",
        "必须根据我所给资料",
        "必须根据现有资料",
        "必须根据我刚才所给资料",
        "基于我刚才所给资料",
        "基于当前已知资料",
        "丰富回答",
        "丰富一点",
        "详细回答",
        "深入研究",
        "给干货",
        "要有干货",
        "不能只回答几句",
        "不要只回答几句",
    ]
    .iter()
    .any(|marker| value.contains(marker))
}

pub(crate) fn forced_rich_answer_is_too_thin(value: &str) -> bool {
    let text = value.trim();
    if text.is_empty() || text.chars().count() < MIN_FORCED_RICH_DELIVERY_CHARS {
        return true;
    }

    let delivery_hits = [
        "## ",
        "| ---",
        "今天就能执行",
        "可直接",
        "待复核",
        "风险边界",
    ]
    .iter()
    .filter(|marker| text.contains(*marker))
    .count();
    delivery_hits < 2
}

pub(crate) fn format_forced_rich_delivery_fallback(known_material: &str) -> String {
    let known_material = if known_material.trim().is_empty() {
        "当前只确认了用户要求必须基于已有资料直接交付，不能把资料缺口作为主体拒答。".to_string()
    } else {
        known_material.trim().to_string()
    };

    if looks_like_secondhand_textbook_material(&known_material) {
        return format_secondhand_textbook_fallback(&known_material);
    }

    format!(
        "## 当前结论\n\n\
         基于目前已知资料，不能再把“缺资料”当成主体回答。先直接给一版可执行的低风险方案：\
         先用已确认的对象和上一轮判断做业务拆解，把现在能判断的部分讲清楚，把不能确定的数字放进复核字段，不用再让用户重新补资料。\n\n\
         ## 已知资料怎么用\n\n\
         - {known_material}\n\n\
         ## 先落地的三件事\n\n\
         1. 先把对象、适配范围、页面表达、成本和售后边界拆成字段，不等完整数据也能开始整理。\n\
         2. 页面和客服只承诺已确认内容，把未确认的价格、库存、政策和效果写成“待复核”，避免编造事实。\n\
         3. 每次只验证一两个关键变量，用咨询、成交、未买原因、退款投诉和毛利变化判断下一步。\n\n\
         ## 可直接执行表\n\n\
         | 环节 | 现在怎么做 | 为什么影响结果 | 记录字段 | 风险边界 |\n\
         | --- | --- | --- | --- | --- |\n\
         | 对象匹配 | 先把已有商品、品类或服务拆清楚，逐项确认适配范围。 | 买家买错或预期不一致，会直接影响咨询、成交和退换。 | 品类、版本、型号、ISBN、适配课程或服务范围、书况/规格。 | 没有真实适配信息时，不写成“全部适用”。 |\n\
         | 页面表达 | 标题、主图和详情页只写已知卖点，先突出用户最容易核对的信息。 | 页面越清楚，无效咨询和收到货后的争议越少。 | 标题关键词、主图要展示的内容、详情页说明、买家高频问题。 | 不承诺未验证功效、最低价、官方背书或平台权益。 |\n\
         | 成本售价 | 先列成本项，再决定售价是否能覆盖损耗和退换。 | 只看成交价容易误判赚钱，真实毛利会被运费、平台费和售后吃掉。 | 收入、收书/采购成本、清洁打包成本、运费、平台费、退款损耗。 | 没有真实成本前，不下确定利润结论。 |\n\
         | 库存履约 | 把可售数量、发货状态和可能滞销的部分分开看。 | 库存不清会导致缺货、压货或履约投诉。 | 库存数量、上架时间、可发货状态、滞销原因、缺货记录。 | 不把未确认库存写成稳定供货。 |\n\
         | 售后退换 | 先把退换边界写清楚，发货前确认关键匹配信息。 | 售后规则不清会放大投诉、退款和差评风险。 | 退换原因、投诉点、处理结果、是否提前说明瑕疵或限制。 | 不承诺超出店铺能力的赔付、质保或时效。 |\n\
         | 复盘调整 | 每次只改最影响判断的一两项，再看真实反馈。 | 同时改太多变量，后面看不出到底是哪一项影响成交。 | 咨询、成交、未买原因、退款投诉、毛利、库存变化。 | 样本太少时，只做阶段判断，不写成最终结论。 |\n\n\
         ## 哪些结论待复核\n\n\
         - 价格、成本、库存、版本适配、平台规则和利润只能在拿到真实数据后定量确认。\n\
         - 当前可以直接执行的是整理字段、规范页面、明确售后边界和做小范围验证；不能编造用户没有给出的数字。"
    )
}

fn looks_like_secondhand_textbook_material(value: &str) -> bool {
    let hits = [
        "二手教材",
        "教材",
        "二手书",
        "高数",
        "英语",
        "专业课",
        "ISBN",
        "版本",
    ]
    .iter()
    .filter(|marker| value.contains(*marker))
    .count();
    hits >= 2
}

fn format_secondhand_textbook_fallback(known_material: &str) -> String {
    let categories = secondhand_textbook_categories(known_material);
    let known_scope = if categories.is_empty() {
        "你已经给出二手教材这个业务方向".to_string()
    } else {
        format!("你已经给出{}这些教材线索", categories.join("、"))
    };
    let known_material_points = secondhand_known_material_points(&categories);

    format!(
        "## 当前结论\n\n\
         二手教材不是只看“有没有人要”，核心是版本和课程能不能对上、书况会不会引发退换、收书和处理成本会不会吃掉毛利、开学季后是否会压库存。\
         {known_scope}，所以可以直接按“三类教材分层 + 页面字段 + 定价核算 + 售后边界”往下做，不需要再让你重新补资料。\n\n\
         ## 已知资料怎么用\n\n\
         {known_material_points}\n\n\
         ## 三类教材先这样拆\n\n\
         | 类别 | 先判断什么 | 页面怎么写 | 主要风险 | 立刻记录 |\n\
         | --- | --- | --- | --- | --- |\n\
         | 高数 | 先看版本、出版社、ISBN、适配课程和老师是否指定同一本。高数需求通常更集中，但版本错了就会直接退。 | 标题写“高数/教材名/版次/出版社/ISBN/适配课程”，详情页放目录页、版次页、书脊和瑕疵图。 | 不同学院或老师可能用不同版本，不能写“全校通用”。 | 版本、ISBN、出版社、课程名、适配年级、是否有习题册、缺页/笔记情况。 |\n\
         | 英语 | 先区分是大学英语教材、视听说、读写教程还是考试辅导资料；不同用途的买家搜索词不一样。 | 标题把“教材用途 + 册数/级别 + 版次 + 书况”放前面，避免只写“英语教材”。 | 英语书常有练习答案、光盘/码/配套资源问题，缺失要提前说。 | 用途、册数/级别、配套资源是否齐全、答案/笔记状态、是否影响课堂使用。 |\n\
         | 专业课 | 先按专业、课程名、老师指定书和考核用途匹配。专业课客群窄，但匹配准时咨询质量更高。 | 标题写“专业/课程名/教材名/版次/ISBN”，详情页说明适配范围，不要泛泛写“专业课教材”。 | 需求窄，错配会压库存；不能把一个专业的书写成所有专业都适用。 | 专业、课程名、老师/班级适配线索、版次、ISBN、是否有重点笔记、库存数量。 |\n\n\
         ## 今天就能执行的动作\n\n\
         1. 先建一张教材清单：类别、书名、版次、出版社、ISBN、课程名、书况、缺页、笔记、配套资料、可售数量、收书成本。\n\
         2. 先上架“版本最明确、书况最好、课程适配最清楚”的书；版本不清、缺页严重或配套资料缺失的书先单独标记，不要混在主推里。\n\
         3. 每本书的标题按这个顺序写：课程/专业 + 教材名 + 版次/出版社 + ISBN 或关键识别信息 + 书况。标题不要只写“高数教材”“英语书”“专业课书”。\n\
         4. 定价不要凭感觉，先记录三个字段：同版本可见在售价、你的收书/整理/包装成本、预留退换损耗。没有真实成交价时，只能给定价区间思路，不能写确定利润。\n\
         5. 客服先问两个问题：你是哪门课/哪个老师要求的书？你需要的版本或 ISBN 是哪一个？确认后再引导下单，减少错买退换。\n\n\
         ## 可直接复用的话术\n\n\
         - 标题模板：`课程/专业 + 教材名 + 版次 + 出版社 + ISBN后4位 + 书况`。\n\
         - 详情页说明：`这本书适合先核对课程名、版次和 ISBN；书况、笔记、缺页和配套资料已在图片/说明里标出，下单前建议按课程要求确认。`\n\
         - 客服确认：`你上课要求的是这个版次和 ISBN 吗？如果老师指定了教材截图，可以先对一下，避免买错版本。`\n\n\
         ## 哪些结论待复核\n\n\
         - 真实售价、同校需求量、库存数量、退换比例、平台规则和利润不能凭空确定，需要用真实记录复核。\n\
         - 当前可以直接推进的是分类清单、标题详情、客服确认、成本字段和售后边界；这些动作不依赖用户继续补资料。"
    )
}

fn secondhand_textbook_categories(value: &str) -> Vec<&'static str> {
    ["高数", "英语", "专业课"]
        .iter()
        .copied()
        .filter(|category| value.contains(category))
        .collect()
}

fn secondhand_known_material_points(categories: &[&str]) -> String {
    let category_line = if categories.is_empty() {
        "- 已知事实：当前问题是二手教材销售或运营分析。".to_string()
    } else {
        format!(
            "- 已知事实：当前问题是二手教材销售或运营分析，已提到的教材类别是{}。",
            categories.join("、")
        )
    };

    [
        category_line,
        "- 本轮处理方式：用户明确要求不要再补资料，所以先基于已知方向给出可执行方案，而不是把资料缺口作为主体拒答。"
            .to_string(),
        "- 安全边界：版本、ISBN、书况、成本、库存、售后和退换先作为核对字段；真实售价、利润、平台规则和需求量只做待复核，不编造数字。"
            .to_string(),
    ]
    .join("\n")
}

#[cfg(test)]
mod tests {
    use super::{forced_rich_answer_is_too_thin, format_forced_rich_delivery_fallback};

    #[test]
    fn forced_rich_answer_quality_rejects_mid_length_template_answer() {
        let output = "当前结论：针对高数、英语、专业课三类二手教材售卖，需落地5项核心管控动作，明确各环节影响及风险边界，具体如下：\n\n\
             1. 教材匹配度管控：错配会直接降低买家购买意愿，还可能引发退换纠纷；需记录每本教材的版本、ISBN、适配课程。\n\n\
             2. 书况与信任维护：成色、缺页情况、笔记内容会影响买家信任度，瑕疵过多易引发差评或退换。\n\n\
             3. 成本管控动作：收书成本、处理成本直接影响毛利，需明确每类教材的收书成本与处理成本。\n\n\
             4. 库存周转管理：开学季前后的上架时间会影响库存周转，冷门教材易积压。\n\n\
             5. 售后复盘优化：咨询量、成交情况、未买原因、退换争议会影响店铺口碑。";

        assert!(forced_rich_answer_is_too_thin(output), "{output}");
    }

    #[test]
    fn secondhand_textbook_fallback_absorbs_old_direct_business_style() {
        let output = format_forced_rich_delivery_fallback(
            "User: 我准备卖二手教材，手里有高数、英语、专业课三类书。\n\
             Assistant: 要看版本、ISBN、书况、成本、库存和退换。",
        );

        assert!(output.contains("高数"), "{output}");
        assert!(output.contains("英语"), "{output}");
        assert!(output.contains("专业课"), "{output}");
        assert!(output.contains("今天就能执行的动作"), "{output}");
        assert!(output.contains("客服确认"), "{output}");
        assert!(output.contains("标题模板"), "{output}");
        assert!(output.contains("ISBN"), "{output}");
        assert!(
            output.contains("已提到的教材类别是高数、英语、专业课"),
            "{output}"
        );
        assert!(!output.contains("Assistant:"), "{output}");
        assert!(!output.contains("请补充"), "{output}");
        assert!(
            !output.contains("这次没有拿到可直接展示的完整内容"),
            "{output}"
        );
    }

    #[test]
    fn generic_forced_rich_fallback_still_has_structured_delivery() {
        let output = format_forced_rich_delivery_fallback("用户要求基于已有商品资料给丰富回答。");

        assert!(output.contains("## 当前结论"), "{output}");
        assert!(output.contains("## 先落地的三件事"), "{output}");
        assert!(output.contains("## 可直接执行表"), "{output}");
        assert!(output.contains("## 哪些结论待复核"), "{output}");
    }
}
