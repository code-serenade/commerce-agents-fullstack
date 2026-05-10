use db_core::{
    DbContext,
    error::{BIZ_INTERNAL_ERROR, BizError, BizResult},
};
use error_code::app as app_error;
use repo::table::app_users::{AppUserService, AppUserStatus};
use uuid::Uuid;

use crate::dto::commerce_playbook::{
    CommerceAgentPlaybookListResponse, CommerceAgentPlaybookResponse,
};

pub struct CommercePlaybookApi {
    app_user_svc: AppUserService,
}

impl CommercePlaybookApi {
    pub fn new(db: DbContext) -> Self {
        Self {
            app_user_svc: AppUserService::new(db),
        }
    }

    pub async fn list_agent_playbooks(
        &self,
        user_id: String,
    ) -> BizResult<CommerceAgentPlaybookListResponse> {
        let user_id = parse_user_id(&user_id)?;
        self.ensure_app_user_enabled(user_id).await?;
        Ok(CommerceAgentPlaybookListResponse {
            playbooks: commerce_agent_playbooks(),
        })
    }

    pub async fn ensure_current_user_enabled(&self, user_id: String) -> BizResult<()> {
        let user_id = parse_user_id(&user_id)?;
        self.ensure_app_user_enabled(user_id).await
    }

    async fn ensure_app_user_enabled(&self, user_id: Uuid) -> BizResult<()> {
        let user = self
            .app_user_svc
            .get_by_user_id(user_id)
            .await?
            .ok_or_else(|| {
                BizError::new(
                    app_error::APP_USER_NOT_FOUND,
                    "app user not found".to_string(),
                )
            })?;
        if user.status != AppUserStatus::Enabled {
            return Err(BizError::new(
                app_error::APP_USER_DISABLED,
                "app user is disabled".to_string(),
            ));
        }
        Ok(())
    }
}

fn commerce_agent_playbooks() -> Vec<CommerceAgentPlaybookResponse> {
    vec![
        playbook(PlaybookSpec(
            "new_product_launch",
            "新品冷启动",
            "从商品事实、首批人群、内容种草、投放测试和复盘节奏生成冷启动方案。",
            "launch",
            "collab",
            &[
                "product_knowledge",
                "product_material",
                "campaign",
                "platform_metrics",
            ],
            &["定位诊断", "首批动作清单", "预算与止损线", "复盘指标"],
            &["product", "data", "creative", "ads", "operations"],
            "请以电商多智能体虚拟员工方式，帮我做一次新品冷启动方案。请先基于商品知识和现有业务上下文判断目标人群、核心卖点、内容钩子、投放测试和运营承接，再输出可执行步骤、预算建议、风险提醒和复盘指标。如果上下文不足，请明确列出需要补充的数据。",
        )),
        playbook(PlaybookSpec(
            "ad_review",
            "投放复盘",
            "复盘投放漏斗、预算节奏、人群、素材和转化承接，输出下一轮调优动作。",
            "review",
            "collab",
            &[
                "campaign",
                "platform_metrics",
                "product_knowledge",
                "workspace_history",
            ],
            &["漏斗诊断", "预算调优", "素材判断", "下一轮动作"],
            &["data", "ads", "creative", "operations"],
            "请复盘一次电商投放问题。请围绕曝光、点击、转化、客单、ROI、人群包、素材和直播间/详情页承接来判断主要瓶颈，并给出下一轮预算调整、素材替换、出价节奏、运营配合和止损条件。不要假设你已经访问外部平台；只使用后端已提供的上下文和用户补充数据。",
        )),
        playbook(PlaybookSpec(
            "customer_complaint_crisis",
            "客诉危机",
            "把售后、客服、商品、内容和运营协同起来，形成安抚话术、处置路径和风险升级规则。",
            "service",
            "execution",
            &[
                "product_knowledge",
                "support_feedback",
                "approval",
                "workspace_history",
            ],
            &["问题分级", "客服话术", "处置清单", "升级规则"],
            &["support", "product", "operations", "risk"],
            "请处理一次电商客诉危机。请先判断问题类型、影响范围、责任边界和用户情绪，再输出客服安抚话术、补偿/退款建议、内部排查清单、舆情风险和需要人工审批的动作。涉及平台写操作或高风险承诺时必须标记为需要审批。",
        )),
        playbook(PlaybookSpec(
            "selection_pricing",
            "选品定价",
            "结合商品成本、竞品、需求场景、毛利和价格带，生成选品与定价建议。",
            "pricing",
            "learning",
            &[
                "product_knowledge",
                "competitor",
                "platform_metrics",
                "campaign",
            ],
            &["机会判断", "价格带建议", "毛利护栏", "试销计划"],
            &["product", "finance", "data", "operations"],
            "请帮我做选品和定价判断。请说明你的判断逻辑，结合商品卖点、目标场景、竞品、价格带、毛利、库存和渠道特性，输出推荐 SKU、定价区间、组合售卖方式、试销计划和风险护栏。上下文不足时请列出必须补充的数据。",
        )),
        playbook(PlaybookSpec(
            "creative_optimization",
            "素材优化",
            "从卖点、场景、受众和转化漏斗出发，生成短视频、直播间和详情页素材优化方案。",
            "creative",
            "collab",
            &[
                "product_material",
                "product_knowledge",
                "platform_metrics",
                "campaign",
            ],
            &["素材诊断", "脚本方向", "A/B 测试", "转化承接"],
            &["creative", "data", "ads", "operations"],
            "请帮我优化电商素材。请从用户场景、开头钩子、卖点表达、信任证明、价格福利、直播间/详情页承接和 A/B 测试角度诊断，并给出可直接执行的短视频脚本方向、直播间话术、详情页模块和测试指标。",
        )),
        playbook(PlaybookSpec(
            "daily_business_review",
            "日报复盘",
            "把平台指标、告警、执行任务、审批和交付物汇总为每日经营复盘。",
            "briefing",
            "execution",
            &[
                "platform_metrics",
                "alerts",
                "workspace_history",
                "approval",
            ],
            &["经营摘要", "异常提醒", "今日动作", "明日重点"],
            &["data", "operations", "risk", "delivery"],
            "请生成一份电商经营日报复盘。请汇总今天的核心指标、异常信号、已完成动作、未完成事项、审批/告警风险、交付物和明天优先级。请区分已知事实、推断和需要补充的数据。",
        )),
    ]
}

struct PlaybookSpec<'a>(
    &'a str,
    &'a str,
    &'a str,
    &'a str,
    &'a str,
    &'a [&'a str],
    &'a [&'a str],
    &'a [&'a str],
    &'a str,
);

fn playbook(spec: PlaybookSpec<'_>) -> CommerceAgentPlaybookResponse {
    let PlaybookSpec(
        id,
        title,
        description,
        category,
        recommended_strategy,
        required_context,
        expected_outputs,
        roles,
        prompt,
    ) = spec;

    CommerceAgentPlaybookResponse {
        id: id.to_string(),
        title: title.to_string(),
        description: description.to_string(),
        category: category.to_string(),
        recommended_strategy: recommended_strategy.to_string(),
        prompt: prompt.to_string(),
        required_context: required_context
            .iter()
            .map(|item| item.to_string())
            .collect(),
        expected_outputs: expected_outputs
            .iter()
            .map(|item| item.to_string())
            .collect(),
        roles: roles.iter().map(|item| item.to_string()).collect(),
    }
}

fn parse_user_id(user_id: &str) -> BizResult<Uuid> {
    Uuid::parse_str(user_id).map_err(|err| {
        BizError::new(
            BIZ_INTERNAL_ERROR,
            format!("invalid user_id uuid for commerce playbooks: {err}"),
        )
    })
}

#[cfg(test)]
mod tests {
    use super::commerce_agent_playbooks;

    #[test]
    fn commerce_agent_playbooks_cover_required_scenarios() {
        let playbooks = commerce_agent_playbooks();
        let ids = playbooks
            .iter()
            .map(|item| item.id.as_str())
            .collect::<Vec<_>>();

        assert_eq!(playbooks.len(), 6);
        assert!(ids.contains(&"new_product_launch"));
        assert!(ids.contains(&"ad_review"));
        assert!(ids.contains(&"customer_complaint_crisis"));
        assert!(ids.contains(&"selection_pricing"));
        assert!(ids.contains(&"creative_optimization"));
        assert!(ids.contains(&"daily_business_review"));
        assert!(playbooks.iter().all(|item| !item.prompt.trim().is_empty()
            && !item.required_context.is_empty()
            && !item.expected_outputs.is_empty()
            && !item.roles.is_empty()));
    }
}
