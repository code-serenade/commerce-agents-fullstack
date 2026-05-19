mod budget;
mod cash_flow;
mod compliance;
mod cost;
mod pl;
mod profit;
mod roi;
mod scenario;

use serde_json::Value;

pub(super) fn cost_calc(input: &Value) -> Value {
    cost::cost_calc(input)
}

pub(super) fn profit_analysis(input: &Value) -> Value {
    profit::profit_analysis(input)
}

pub(super) fn roi_calc(input: &Value) -> Value {
    roi::roi_calc(input)
}

pub(super) fn budget_plan(input: &Value) -> Value {
    budget::budget_plan(input)
}

pub(super) fn break_even_calc(input: &Value) -> Value {
    profit::break_even_calc(input)
}

pub(super) fn compliance_check(input: &Value) -> Value {
    compliance::compliance_check(input)
}

pub(super) fn pl_statement(input: &Value) -> Value {
    pl::pl_statement(input)
}

pub(super) fn budget_vs_actual(input: &Value) -> Value {
    budget::budget_vs_actual(input)
}

pub(super) fn gmv_waterfall(input: &Value) -> Value {
    pl::gmv_waterfall(input)
}

pub(super) fn scenario_analysis(input: &Value) -> Value {
    scenario::scenario_analysis(input)
}

pub(super) fn cash_flow_forecast(input: &Value) -> Value {
    cash_flow::cash_flow_forecast(input)
}

pub(super) fn financial_narrative(input: &Value) -> Value {
    pl::financial_narrative(input)
}
