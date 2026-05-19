use serde_json::{Value, json};

use super::super::support::{currency, deferred_accounting, num, percent, round2, string};

pub(in crate::skills::accounting) fn compliance_check(input: &Value) -> Value {
    let revenue = num(input, "monthly_revenue", 0.0);
    if revenue <= 0.0 {
        return deferred_accounting(
            "accounting_compliance_check",
            "请传入 monthly_revenue；纯计算版本不会自动读取店铺营收。",
            input,
        );
    }
    let tax_type = string(input, "tax_type", "小规模");
    let invoice_rate = num(input, "invoice_rate", 80.0);
    let cash_ratio = num(input, "cash_ratio", 10.0);
    let mut risks = Vec::new();
    if tax_type == "小规模" && revenue * 12.0 > 5_000_000.0 {
        risks.push(json!({"风险": "年化营收超500万需转为一般纳税人", "等级": "高", "建议": "提前完成纳税人身份切换和税负测算"}));
    }
    if invoice_rate < 70.0 {
        risks.push(
            json!({"风险": "开票率偏低", "等级": "中", "建议": "规范开票流程，补齐交易凭证"}),
        );
    }
    if cash_ratio > 30.0 {
        risks.push(
            json!({"风险": "现金收款占比过高", "等级": "中", "建议": "提高线上支付占比并保留流水"}),
        );
    }
    let vat_rate = if tax_type == "一般纳税人" {
        0.13
    } else {
        0.03
    };
    let vat = round2(revenue * vat_rate);
    let surcharge = round2(vat * 0.12);
    let income_tax = round2(revenue * 0.05 * 0.25);
    json!({
        "status": "success",
        "has_data": true,
        "data_readiness": if input.get("invoice_rate").is_none() || input.get("cash_ratio").is_none() { "calculated_from_defaults" } else { "user_provided" },
        "adapter_status": "manual_input",
        "summary": format!("合规检查完成，预估税额合计{}，发现{}个风险点", currency(vat + surcharge + income_tax), risks.len()),
        "数据来源": "用户提供",
        "月营收": currency(revenue),
        "纳税类型": tax_type,
        "预估税额": {
            "增值税": currency(vat),
            "附加税": currency(surcharge),
            "企业所得税": currency(income_tax),
            "合计": currency(vat + surcharge + income_tax),
        },
        "合规检查": {
            "开票率": percent(invoice_rate, 1),
            "现金占比": percent(cash_ratio, 1),
            "风险点数": risks.len(),
        },
        "风险详情": if risks.is_empty() { vec![json!({"提示": "暂未发现合规风险"})] } else { risks },
        "建议": ["按月申报纳税", "保留交易凭证和平台流水", "每月完成收入、发票、回款三方核对"],
    })
}
