//! Session-scoped worker runner.
//!
//! The worker is created by `SessionRuntime`, handles assignments inside that
//! session, and is shut down by the runtime after the session completes.

mod autosuggest;
mod context;
mod execution;
mod inference;
mod output;
mod output_normalization;
mod policy;
mod skill_requests;
mod types;
mod util;

pub use execution::Worker;
pub use types::BoxedAction;
