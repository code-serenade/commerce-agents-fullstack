use serde_json::Value;

use super::{
    calculations::{
        break_even_calc, budget_plan, budget_vs_actual, cash_flow_forecast, compliance_check,
        cost_calc, financial_narrative, gmv_waterfall, pl_statement, profit_analysis, roi_calc,
        scenario_analysis,
    },
    support::{invalid_input, outcome},
};
use crate::skills::{SkillContext, SkillError, SkillOutcome};

pub async fn execute(
    skill_name: &str,
    input: Value,
    _context: &SkillContext,
) -> Result<SkillOutcome, SkillError> {
    let value = match skill_name {
        "accounting_cost_calc" => cost_calc(&input),
        "accounting_profit_analysis" => profit_analysis(&input),
        "accounting_roi_calc" => roi_calc(&input),
        "accounting_budget_plan" => budget_plan(&input),
        "accounting_break_even_calc" => break_even_calc(&input),
        "accounting_compliance_check" => compliance_check(&input),
        "accounting_pl_statement" => pl_statement(&input),
        "accounting_budget_vs_actual" => budget_vs_actual(&input),
        "accounting_gmv_waterfall" => gmv_waterfall(&input),
        "accounting_scenario_analysis" => scenario_analysis(&input),
        "accounting_cash_flow_forecast" => cash_flow_forecast(&input),
        "accounting_financial_narrative" => financial_narrative(&input),
        other => return Err(invalid_input(format!("unknown accounting skill: {other}"))),
    };
    Ok(outcome(value))
}
