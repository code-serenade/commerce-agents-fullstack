use async_trait::async_trait;
use serde_json::{Map, Value, json};

use super::{
    common::{
        compact_task_text_value, compact_text_for_task_body, compact_text_for_task_title,
        infer_primary_product_id, infer_primary_product_ref, metadata_csv_list,
        metadata_string_value, product_id_from_ref,
    },
    specs::coordination_create_campaign_spec,
};
use crate::skills::{
    Skill, SkillContext, SkillOutcome, SkillResult, SkillSpec, object_params, optional_f64_param,
    optional_string_param, optional_string_vec_param, string_param,
};

pub struct CoordinationCreateCampaign {
    spec: SkillSpec,
}

impl CoordinationCreateCampaign {
    pub fn new() -> Self {
        Self {
            spec: coordination_create_campaign_spec(),
        }
    }
}

impl Default for CoordinationCreateCampaign {
    fn default() -> Self {
        Self::new()
    }
}

#[async_trait]
impl Skill for CoordinationCreateCampaign {
    fn spec(&self) -> &SkillSpec {
        &self.spec
    }

    async fn run(&self, params: Value, context: SkillContext) -> SkillResult {
        object_params(&params)?;
        let name = compact_text_for_task_title(&string_param(&params, "name")?);
        let description = optional_string_param(&params, "description")?
            .map(|value| compact_text_for_task_body(&value))
            .filter(|value| !value.is_empty())
            .or_else(|| metadata_string_value(&context.metadata, "task_description"));
        let campaign_type = normalize_campaign_type(
            optional_string_param(&params, "campaign_type")?
                .unwrap_or_else(|| infer_campaign_type(&name, description.as_deref())),
        );
        let budget = optional_f64_param(&params, "budget")?;
        let product_ref = optional_string_param(&params, "product_ref")?
            .or_else(|| infer_primary_product_ref(&context.metadata));
        let product_id = optional_string_param(&params, "product_id")?
            .or_else(|| {
                product_ref
                    .as_deref()
                    .and_then(product_id_from_ref)
                    .map(ToOwned::to_owned)
            })
            .or_else(|| infer_primary_product_id(&context.metadata));
        let goal = optional_string_param(&params, "goal")?
            .map(|value| compact_text_for_task_body(&value))
            .filter(|value| !value.is_empty())
            .or_else(|| description.clone())
            .or_else(|| Some(name.clone()));
        let target_audience = optional_string_param(&params, "target_audience")?
            .map(|value| compact_text_for_task_body(&value))
            .filter(|value| !value.is_empty());
        let channels = normalize_campaign_channels(
            optional_string_vec_param(&params, "channels")?.unwrap_or_default(),
        );
        let strategy = optional_string_param(&params, "strategy")?
            .map(|value| compact_text_for_task_body(&value))
            .filter(|value| !value.is_empty());
        let start_date = optional_string_param(&params, "start_date")?
            .map(|value| compact_task_text_value(&value, 64))
            .filter(|value| !value.is_empty());
        let end_date = optional_string_param(&params, "end_date")?
            .map(|value| compact_task_text_value(&value, 64))
            .filter(|value| !value.is_empty());
        let workspace_id = metadata_string_value(&context.metadata, "workspace_id")
            .or_else(|| metadata_string_value(&context.metadata, "message.workspace_id"))
            .or_else(|| metadata_string_value(&context.metadata, "request.workspace.workspace_id"));
        let role = metadata_string_value(&context.metadata, "role_runtime")
            .or_else(|| metadata_string_value(&context.metadata, "assignment_role"))
            .unwrap_or_else(|| "agent".to_string());
        let source_summary =
            metadata_string_value(&context.metadata, "request.context.selected_summary")
                .map(|value| compact_text_for_task_body(&value))
                .or_else(|| description.clone());
        let draft = build_campaign_draft_blueprint(
            &name,
            &campaign_type,
            goal.as_deref(),
            target_audience.as_deref(),
            &channels,
            strategy.as_deref(),
            budget,
        );

        let normalized = json!({
            "name": name,
            "description": description,
            "campaign_type": campaign_type,
            "budget": budget,
            "product_ref": product_ref,
            "product_id": product_id,
            "goal": goal,
            "target_audience": target_audience,
            "channels": channels,
            "strategy": strategy,
            "start_date": start_date,
            "end_date": end_date,
            "workspace_id": workspace_id,
            "role": role,
            "source_summary": source_summary,
            "draft": draft,
            "source": "session_skill",
        });

        Ok(SkillOutcome::new(json!({
            "status": "success",
            "campaign_creation": {
                "should_persist": true,
                "name": normalized.get("name").cloned().unwrap_or(Value::Null),
                "description": normalized.get("description").cloned().unwrap_or(Value::Null),
                "campaign_type": normalized.get("campaign_type").cloned().unwrap_or(Value::Null),
                "budget": normalized.get("budget").cloned().unwrap_or(Value::Null),
                "product_ref": normalized.get("product_ref").cloned().unwrap_or(Value::Null),
                "product_id": normalized.get("product_id").cloned().unwrap_or(Value::Null),
                "goal": normalized.get("goal").cloned().unwrap_or(Value::Null),
                "target_audience": normalized.get("target_audience").cloned().unwrap_or(Value::Null),
                "channels": normalized.get("channels").cloned().unwrap_or_else(|| json!([])),
                "strategy": normalized.get("strategy").cloned().unwrap_or(Value::Null),
                "start_date": normalized.get("start_date").cloned().unwrap_or(Value::Null),
                "end_date": normalized.get("end_date").cloned().unwrap_or(Value::Null),
                "workspace_id": normalized.get("workspace_id").cloned().unwrap_or(Value::Null),
                "role": normalized.get("role").cloned().unwrap_or(Value::Null),
                "source_summary": normalized.get("source_summary").cloned().unwrap_or(Value::Null),
                "draft": normalized.get("draft").cloned().unwrap_or_else(|| json!({})),
                "source": "session_skill"
            },
            "normalized_inputs": normalized,
            "no_side_effects": true
        }))
        .with_summary(format!("营销活动草稿已生成：{}", name))
        .with_metadata("campaign_creation_kind", json!("campaign_draft"))
        .with_metadata("campaign_creation_should_persist", json!(true)))
    }
}

fn normalize_campaign_type(value: String) -> String {
    match value.trim().to_ascii_lowercase().as_str() {
        "launch" | "上新" | "冷启动" => "launch".to_string(),
        "promotion" | "promo" | "campaign" | "促销" | "大促" => "promotion".to_string(),
        "ad_review" | "review" | "复盘" => "ad_review".to_string(),
        "agent_draft" | "draft" | "草稿" => "agent_draft".to_string(),
        _ => compact_task_text_value(value.trim(), 64),
    }
}

fn infer_campaign_type(name: &str, description: Option<&str>) -> String {
    let text = format!("{} {}", name, description.unwrap_or_default()).to_ascii_lowercase();
    if text.contains("复盘") || text.contains("review") {
        "ad_review".to_string()
    } else if text.contains("上新") || text.contains("冷启动") || text.contains("launch") {
        "launch".to_string()
    } else if text.contains("促销")
        || text.contains("大促")
        || text.contains("618")
        || text.contains("双11")
        || text.contains("promotion")
    {
        "promotion".to_string()
    } else {
        "agent_draft".to_string()
    }
}

fn normalize_campaign_channels(values: Vec<String>) -> Vec<String> {
    let mut normalized = Vec::new();
    for value in values {
        let value = value.trim();
        if value.is_empty() {
            continue;
        }
        let channel = match value.trim().to_ascii_lowercase().as_str() {
            "抖音" | "douyin" => "douyin".to_string(),
            "淘宝" | "taobao" => "taobao".to_string(),
            "京东" | "jd" => "jd".to_string(),
            "拼多多" | "pdd" => "pdd".to_string(),
            other => compact_task_text_value(other, 64),
        };
        if !normalized.iter().any(|item| item == &channel) {
            normalized.push(channel);
        }
    }
    normalized
}

fn build_campaign_draft_blueprint(
    name: &str,
    campaign_type: &str,
    goal: Option<&str>,
    target_audience: Option<&str>,
    channels: &[String],
    strategy: Option<&str>,
    budget: Option<f64>,
) -> Value {
    let execution_steps = match campaign_type {
        "launch" => vec![
            "确认主推商品、定价和核心卖点".to_string(),
            "完成首批渠道内容和素材排期".to_string(),
            "先小预算冷启动测点击率和加购率".to_string(),
            "根据首轮数据放大高转化渠道".to_string(),
        ],
        "ad_review" => vec![
            "回收活动期投放、转化和客服反馈数据".to_string(),
            "定位高 ROI 渠道与低效素材".to_string(),
            "沉淀保留、暂停和重做动作".to_string(),
            "形成下一轮预算与节奏建议".to_string(),
        ],
        _ => vec![
            "锁定活动商品、价格带和优惠机制".to_string(),
            "按渠道准备内容、素材和节奏".to_string(),
            "跟踪点击、加购、成交和退款信号".to_string(),
            "按表现及时调预算和策略".to_string(),
        ],
    };
    let metrics = match campaign_type {
        "launch" => vec![
            "点击率".to_string(),
            "加购率".to_string(),
            "首单成本".to_string(),
        ],
        "ad_review" => vec![
            "ROI".to_string(),
            "转化率".to_string(),
            "退款率".to_string(),
        ],
        _ => vec!["GMV".to_string(), "ROI".to_string(), "转化率".to_string()],
    };

    json!({
        "name": name,
        "campaignType": campaign_type,
        "goal": goal,
        "targetAudience": target_audience,
        "channels": channels,
        "strategy": strategy,
        "budget": budget,
        "executionSteps": execution_steps,
        "successMetrics": metrics,
        "guardrails": [
            "平台执行动作后续仍需审批后再触发",
            "草稿阶段先沉淀方案，不直接写真实平台",
            "素材、商品和工作区来源应保持可追溯"
        ]
    })
}
