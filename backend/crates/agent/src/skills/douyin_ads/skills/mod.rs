mod audience;
mod budget;
mod funnel;
mod plan;

use serde_json::Value;

pub(super) fn plan_structure(input: &Value) -> Value {
    plan::plan_structure(input)
}

pub(super) fn audience_package(input: &Value) -> Value {
    audience::audience_package(input)
}

pub(super) fn budget_pacing(input: &Value) -> Value {
    budget::budget_pacing(input)
}

pub(super) fn live_funnel_diagnosis(input: &Value) -> Value {
    funnel::live_funnel_diagnosis(input)
}
