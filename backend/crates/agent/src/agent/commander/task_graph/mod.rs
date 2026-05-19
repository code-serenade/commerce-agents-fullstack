mod builder;
mod chat;
mod config;
mod planner;
mod runtime;
mod util;
mod validation;

pub(super) const DEFAULT_NODE_ACCEPTANCE: &str =
    "输出必须围绕本岗位职责给出可复核的业务判断、依据、风险边界和下一步动作。";

pub use runtime::*;
