use async_trait::async_trait;
use serde_json::Value;

use super::{
    context::explicit_product_context,
    output::{matched_context_output, missing_context_output, unmatched_context_output},
};
use crate::skills::service::*;

pub struct ServiceQueryProduct {
    spec: SkillSpec,
}

impl ServiceQueryProduct {
    pub fn new() -> Self {
        Self {
            spec: service_query_product_spec(),
        }
    }
}

impl Default for ServiceQueryProduct {
    fn default() -> Self {
        Self::new()
    }
}

#[async_trait]
impl Skill for ServiceQueryProduct {
    fn spec(&self) -> &SkillSpec {
        &self.spec
    }

    async fn run(&self, params: Value, context: SkillContext) -> SkillResult {
        object_params(&params)?;
        let query_product_ref = optional_string_param(&params, "product_ref")?;
        let query_product_id = optional_product_id_param(&params, "product_id")?;
        let query_product_name = optional_string_param(&params, "product_name")?;
        let product_context = explicit_product_context(&context.metadata);

        if !product_context.has_explicit_context() {
            return outcome(
                missing_context_output(
                    &self.spec.id,
                    &query_product_ref,
                    &query_product_id,
                    &query_product_name,
                ),
                "未发现本轮显式选择的商品上下文",
            );
        }

        let match_result = product_context.match_query(
            query_product_ref.as_deref(),
            query_product_id.as_deref(),
            query_product_name.as_deref(),
        );
        if !match_result.matched {
            return outcome(
                unmatched_context_output(
                    &self.spec.id,
                    &query_product_ref,
                    &query_product_id,
                    &query_product_name,
                    product_context,
                ),
                "已读取商品上下文，但没有匹配请求的商品条件",
            );
        }

        outcome(
            matched_context_output(
                &self.spec.id,
                &query_product_ref,
                &query_product_id,
                &query_product_name,
                product_context,
                match_result,
            ),
            "商品上下文查询已完成",
        )
    }
}

fn optional_product_id_param(params: &Value, name: &str) -> Result<Option<String>, SkillError> {
    Ok(match object_params(params)?.get(name) {
        None | Some(Value::Null) => None,
        Some(Value::String(value)) => {
            let value = value.trim();
            if value.is_empty() {
                None
            } else {
                Some(value.to_owned())
            }
        }
        Some(Value::Number(value)) => Some(value.to_string()),
        Some(_) => {
            return Err(SkillError::invalid_parameter(
                name,
                "string or integer product id",
            ));
        }
    })
}
