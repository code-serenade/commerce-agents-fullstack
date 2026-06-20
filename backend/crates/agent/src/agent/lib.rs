//! # Agent
//!
//! Agent 聚合系统中的核心生命体实现：
//! - Commander
//! - Worker

pub mod commander;
pub(crate) mod context_text;
pub(crate) mod forced_rich_delivery;
pub mod worker;
