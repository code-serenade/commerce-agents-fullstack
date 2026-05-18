#[path = "output_normalization/benchmark.rs"]
mod benchmark;
#[path = "output_normalization/category_boundary.rs"]
mod category_boundary;
#[path = "output_normalization/evidence.rs"]
mod evidence;
#[path = "output_normalization/failure_report.rs"]
mod failure_report;
#[path = "output_normalization/merge.rs"]
mod merge;
#[path = "output_normalization/normalize.rs"]
mod normalize;
#[path = "output_normalization/policy.rs"]
mod normalization_policy;
#[path = "output_normalization/runtime_helpers.rs"]
mod runtime_helpers;
#[path = "output_normalization/runtime_noise.rs"]
mod runtime_noise;
#[path = "output_normalization/skill_result.rs"]
mod skill_result;

use benchmark::sanitize_unsupported_benchmark_claims;
use category_boundary::{
    asks_for_specific_product_identity, contains_runtime_data_gap,
    format_category_level_skill_boundary, is_category_level_task, is_non_blocking_category_skill_gap,
    normalize_category_level_product_requests,
};
use evidence::{merge_unique, normalize_product_context_evidence, normalize_upstream_evidence};
use runtime_noise::{
    contains_refusal_summary, is_category_runtime_noise, sanitize_category_level_runtime_artifacts,
};
use skill_result::{
    format_failed_skill_risk, format_incomplete_skill_question, format_incomplete_skill_risk,
    format_success_skill_evidence,
};

use category_boundary::{
    assignment_user_task, category_level_skill_boundary_output,
    should_downgrade_category_worker_failure,
};
use failure_report::category_boundary_failure_report;
use merge::merge_skill_results;
use normalize::normalize_worker_role_output;
use runtime_helpers::{
    compact_event_text, metadata_string, skill_executor_unavailable_result, skill_status_name,
};
