use super::*;

pub struct ServiceDsrImprovement {
    spec: SkillSpec,
}

impl ServiceDsrImprovement {
    pub fn new() -> Self {
        Self {
            spec: service_dsr_improvement_spec(),
        }
    }
}

impl Default for ServiceDsrImprovement {
    fn default() -> Self {
        Self::new()
    }
}

#[async_trait]
impl Skill for ServiceDsrImprovement {
    fn spec(&self) -> &SkillSpec {
        &self.spec
    }

    async fn run(&self, params: Value, _context: SkillContext) -> SkillResult {
        object_params(&params)?;
        let desc = optional_f64_param(&params, "description_score")?.unwrap_or(4.6);
        let svc = optional_f64_param(&params, "service_score")?.unwrap_or(4.7);
        let logi = optional_f64_param(&params, "logistics_score")?.unwrap_or(4.5);
        let platform =
            optional_string_param(&params, "platform")?.unwrap_or_else(|| "淘宝".to_owned());
        let overall = round2((desc + svc + logi) / 3.0);
        let lowest = [("描述相符", desc), ("服务态度", svc), ("物流服务", logi)]
            .into_iter()
            .min_by(|a, b| a.1.partial_cmp(&b.1).unwrap_or(std::cmp::Ordering::Equal))
            .unwrap_or(("描述相符", desc));
        let has_user_scores = params.get("description_score").is_some()
            || params.get("service_score").is_some()
            || params.get("logistics_score").is_some();
        outcome(
            json!({
                "status": "success",
                "has_data": true,
                "adapter_status": "manual_input",
                "data_readiness": if has_user_scores { "user_provided" } else { "calculated_from_defaults" },
                "summary": "DSR提升方案已生成",
                "平台": platform,
                "执行模式": "native_dsr_improvement",
                "数据来源": "用户提供/默认样例；未读取实时店铺指标和行业基准",
                "DSR总评": overall,
                "各维度评分": {"描述相符": desc, "服务态度": svc, "物流服务": logi},
                "健康状态": if overall >= 4.8 { "优秀" } else if overall >= 4.6 { "正常" } else { "需改善" },
                "优先改善维度": lowest.0,
                "改善方案": dsr_action_plan(lowest.0),
                "7天动作": [
                    "拉取低分评价和售后工单，按商品描述/服务响应/物流时效分组",
                    "为低分订单建立回访池，优先处理可补救客户",
                    "把高频问题同步商品、仓配和客服负责人",
                    "每日复盘新低分原因并更新客服话术"
                ],
                "30天追踪指标": ["DSR总评", "低分评价占比", "售后响应时长", "退款纠纷率", "物流异常率"],
                "后续适配器": "接入 store_metrics/fresh_benchmark 后可用真实店铺指标替换默认样例。"
            }),
            "DSR提升方案已生成",
        )
    }
}

fn dsr_action_plan(dimension: &str) -> Vec<&'static str> {
    match dimension {
        "描述相符" => vec![
            "排查主图、详情页、规格参数和实际商品不一致点",
            "把容易误解的尺寸、材质、颜色、适配范围前置说明",
            "对高退货SKU补充实拍图和购买前确认话术",
        ],
        "服务态度" => vec![
            "把P0/P1客诉SLA前置到客服排班和质检规则",
            "沉淀高频售后问题的标准话术，减少反复追问",
            "对差评/投诉威胁场景启动主管接管流程",
        ],
        "物流服务" => vec![
            "标记高异常地区和高延迟仓配线路",
            "对延迟订单提前触达并给出预计时间",
            "将物流异常与补偿/催派流程合并到客服工单",
        ],
        _ => vec![
            "定位低分订单与评价关键词",
            "建立问题分类",
            "制定客服/物流/描述修正动作",
        ],
    }
}

pub(super) fn default_faq_questions() -> Vec<String> {
    vec![
        "什么时候发货？".to_owned(),
        "支持退货退款吗？怎么操作？".to_owned(),
        "质量有保障吗？".to_owned(),
        "有优惠活动吗？".to_owned(),
        "怎么选尺码/规格？".to_owned(),
        "包邮吗？运费多少？".to_owned(),
        "可以开发票吗？".to_owned(),
        "商品是正品吗？".to_owned(),
    ]
}
