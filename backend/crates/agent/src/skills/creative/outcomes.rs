fn native_creative_outcome(skill_id: &str, input: Map<String, Value>) -> SkillOutcome {
    match skill_id {
        "creative_video_script" => {
            let product = normalized_string(&input, "product_name", "待推广商品");
            let video_type = normalized_string(&input, "video_type", "种草");
            let duration = normalized_i64(&input, "duration", 30).clamp(10, 180);
            let platform = normalized_string(&input, "platform", "抖音");
            SkillOutcome::new(json!({
                "能力": "短视频脚本",
                "商品": product,
                "平台": platform,
                "视频类型": video_type,
                "时长秒": duration,
                "脚本结构": [
                    {"时间": "0-3秒", "模块": "开场钩子", "画面": "直接展示使用前后的强对比或高频痛点", "台词": format!("还在为{}的选择纠结？先看这个真实场景。", product), "字幕": "先看效果，再决定要不要入手"},
                    {"时间": "3-10秒", "模块": "痛点场景", "画面": "用户在通勤、办公或直播间场景中遇到具体问题", "台词": "很多人不是不想买，而是不知道它到底解决什么问题。", "字幕": "把痛点讲具体"},
                    {"时间": format!("10-{}秒", (duration - 5).max(12)), "模块": "卖点展示", "画面": "用近景、手部动作和对比镜头展示核心卖点", "台词": format!("这款{}重点看三件事：场景适配、使用便利、转化利益点。", product), "字幕": "卖点1 / 卖点2 / 卖点3"},
                    {"时间": "最后5秒", "模块": "行动号召", "画面": "回到商品和优惠利益点", "台词": "想要同款体验，先收藏，再看当前活动价。", "字幕": "收藏 / 加购 / 领券"}
                ],
                "拍摄提示": ["首屏必须给结果或冲突", "单镜头不超过4秒", "每个卖点都配一个可见动作", "避免夸大功效和绝对化承诺"],
                "复盘指标": ["3秒留存", "完播率", "评论问题", "商品点击率", "加购率"],
                "数据边界": "本地 native 产物未读取外部实时趋势和商品详情，可在接入商品/搜索 adapter 后增强。"
            }))
            .with_summary("短视频脚本已生成")
        }
        "creative_seeding_copy" => {
            let product = normalized_string(&input, "product_name", "待推广商品");
            let audience = normalized_string(&input, "target_audience", "年轻女性");
            let tone = normalized_string(&input, "tone", "闺蜜分享");
            let selling_points = normalized_string_vec(&input, "selling_points");
            let points = if selling_points.is_empty() {
                vec![
                    "真实使用场景".to_owned(),
                    "决策门槛低".to_owned(),
                    "适合日常复购".to_owned(),
                ]
            } else {
                selling_points
            };
            SkillOutcome::new(json!({
                "能力": "种草文案",
                "商品": product,
                "目标人群": audience,
                "语气": tone,
                "标题备选": [
                    format!("用了{}之后，我才知道以前踩了多少坑", product),
                    format!("{}到底值不值得买？我按真实场景说", product),
                    format!("给{}人群的{}选择清单", audience, product)
                ],
                "正文草稿": [
                    format!("最近在看{}，我最关心的不是参数堆得多不多，而是它能不能解决真实场景里的问题。", product),
                    format!("实际体验下来，最值得说的是：{}。", points.join("、")),
                    "如果你和我一样更在意省心、稳定和日常使用感，可以先从核心卖点和售后承诺两件事判断。".to_owned(),
                    "建议先收藏，再对比同价位竞品的评价和真实买家反馈，别只看单条爆款内容下单。".to_owned()
                ],
                "话题标签": ["#电商好物", "#真实测评", "#种草笔记", "#购物决策", "#好物分享"],
                "发布建议": {"平台": "小红书/抖音图文", "时间": "工作日20:00-22:30或周末10:00-12:00", "配图": "首图给结果，二图列卖点，三图放使用场景"},
                "转化动作": ["评论区收集顾虑", "置顶优惠和规格说明", "把高频问题沉淀为下一条内容"]
            }))
            .with_summary("种草文案已生成")
        }
        "creative_ip_branding" => {
            let brand = normalized_string(&input, "brand_name", "品牌");
            let category = normalized_string(&input, "category", "通用类目");
            let audience = normalized_string(&input, "target_audience", "18-35岁年轻消费者");
            let differentiator = normalized_string(&input, "differentiator", "专业+有温度");
            let platform = normalized_string(&input, "platform", "小红书/抖音");
            SkillOutcome::new(json!({
                "能力": "IP定位",
                "品牌": brand,
                "类目": category,
                "目标受众": audience,
                "差异化": differentiator,
                "平台": platform,
                "人设定位": format!("{}领域里的实用型选购顾问，表达风格是{}。", category, differentiator),
                "账号简介": format!("帮{}用户少踩坑、会选择、买得值。", audience),
                "内容支柱": [
                    {"栏目": "选购避坑", "作用": "建立专业可信度"},
                    {"栏目": "真实场景测评", "作用": "让用户看到使用后变化"},
                    {"栏目": "价格与规格解释", "作用": "降低决策成本"},
                    {"栏目": "用户问题答疑", "作用": "沉淀客服与内容共用资产"}
                ],
                "语气规范": ["少喊口号，多讲证据", "不做绝对化承诺", "把参数翻译成用户能理解的场景利益"],
                "30天起号动作": ["每天1条场景内容", "每周2条测评/对比", "每周复盘评论问题并改选题"]
            }))
            .with_summary("IP定位方案已生成")
        }
        "creative_trend_catch" => {
            let category = normalized_string(&input, "category", "通用类目");
            let platform = normalized_string(&input, "platform", "抖音");
            SkillOutcome::new(json!({
                "能力": "趋势捕捉",
                "类目": category,
                "平台": platform,
                "趋势方向": [
                    {"方向": "真实测评透明化", "内容切入": "把优缺点、适用人群和不适用人群讲清楚", "适合指标": "收藏率/评论问题量"},
                    {"方向": "场景解决方案", "内容切入": "从通勤、办公、家庭、礼赠等场景切入", "适合指标": "完播率/商品点击率"},
                    {"方向": "价格价值解释", "内容切入": "拆解为什么值这个价，和竞品差异在哪里", "适合指标": "加购率/转化率"},
                    {"方向": "评论区共创", "内容切入": "把用户质疑直接做成下一条内容", "适合指标": "互动率/复访率"}
                ],
                "首批选题": [
                    format!("{}用户最容易忽略的3个选择标准", category),
                    format!("{}到底怎么选才不浪费钱", category),
                    format!("同价位{}，我会先看这4件事", category)
                ],
                "风险": ["未接实时外部搜索，不能代表当天热点榜单", "涉及功效、销量、排名时必须用真实证据"],
                "下一步": ["接入平台热搜或站内搜索词", "把趋势方向映射到内容排期", "用标题CTR和完播率做淘汰"]
            }))
            .with_summary("内容趋势方向已生成")
        }
        "creative_live_script" => {
            let duration = normalized_f64(&input, "duration_hours", 3.0).clamp(0.5, 12.0);
            let product_count = normalized_i64(&input, "product_count", 6).clamp(1, 30);
            let live_type = normalized_string(&input, "live_type", "日常带货");
            let platform = normalized_string(&input, "platform", "抖音");
            SkillOutcome::new(json!({
                "能力": "直播脚本",
                "直播类型": live_type,
                "平台": platform,
                "时长小时": duration,
                "商品数": product_count,
                "阶段脚本": [
                    {"阶段": "开场暖场", "时间": "0-15分钟", "话术": "欢迎刚进来的朋友，今天先不急着下单，我会把适合谁、不适合谁、怎么选讲清楚。先点关注，后面有限时福利。", "目标": "建立信任和停留"},
                    {"阶段": "福利引流款", "时间": "15-30分钟", "话术": "这款先给新朋友试水，重点不是便宜，而是用最低门槛确认你适不适合。库存和券都有限，先领券再决定。", "目标": "拉动互动和首单"},
                    {"阶段": "主推款讲解", "时间": "循环讲解", "话术": "看这个商品先抓三点：使用场景、核心卖点、售后保障。适合的朋友可以加购，不适合我也会直接说。", "目标": "提高商品点击和加购"},
                    {"阶段": "返场收尾", "时间": "最后30分钟", "话术": "最后把今天问得最多的问题再统一回答一次，错过本轮福利的先关注，下次开播会继续复盘真实反馈。", "目标": "收口转化和沉淀粉丝"}
                ],
                "高频话术": ["先领券，不急着拍", "适合再买，不适合别冲动", "这个问题问得好，我现场拆一下", "库存不做夸张承诺，以后台为准", "拍下前先看规格和售后"],
                "运营提醒": ["每15分钟复述核心利益点", "把高频问题交给客服沉淀FAQ", "避免虚假最低价和绝对化效果承诺"]
            }))
            .with_summary("直播话术脚本已生成")
        }
        "creative_product_article" => {
            let product = normalized_string(&input, "product_name", "待推广商品");
            let article_type = normalized_string(&input, "article_type", "测评软文");
            let word_count = normalized_i64(&input, "word_count", 800).clamp(300, 3000);
            let platform = normalized_string(&input, "platform", "微信公众号");
            SkillOutcome::new(json!({
                "能力": "产品软文",
                "商品": product,
                "文章类型": article_type,
                "目标字数": word_count,
                "平台": platform,
                "标题": format!("{}怎么选？一篇讲清真实场景、核心卖点和避坑点", product),
                "文章结构": [
                    {"模块": "开头", "写法": "用具体痛点切入，说明为什么需要重新评估这个商品"},
                    {"模块": "场景", "写法": "把目标用户、使用场景和购买动机讲清楚"},
                    {"模块": "卖点", "写法": "每个卖点都对应一个使用结果，不只列参数"},
                    {"模块": "对比", "写法": "说明同价位选择标准，不编造竞品数据"},
                    {"模块": "结尾", "写法": "给出适合/不适合人群和购买前检查项"}
                ],
                "正文草稿": [
                    format!("很多人在选{}时，容易先看价格或单条推荐，但真正影响复购和满意度的是使用场景是否匹配。", product),
                    "建议先判断三个问题：你在哪个场景使用、最不能接受的缺点是什么、售后和规格是否清楚。".to_owned(),
                    format!("如果{}能在核心场景里稳定解决问题，再结合价格、评价和服务承诺判断是否值得入手。", product),
                    "购买前建议保留对比清单，不要只被限时话术推动。".to_owned()
                ],
                "CTA": "先收藏这份选购清单，再结合你的预算和使用场景做最终选择。",
                "合规提醒": ["不写绝对功效", "不编造销量排名", "涉及数据时标注来源"]
            }))
            .with_summary("产品软文草稿已生成")
        }
        _ => SkillOutcome::new(json!({
            "能力": skill_id,
            "输入": input,
            "说明": "未知创意 skill，已返回输入摘要。"
        }))
        .with_summary("创意技能已返回输入摘要"),
    }
}
