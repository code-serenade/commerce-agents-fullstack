# Post 8-59 Backend Design: Data Comprehensive Diagnosis Native Orchestrator

## Context

Old product `DataComprehensiveDiagnosis` is a data analyst orchestration skill. It loads store metrics, runs anomaly detection, customer segmentation, refund decomposition, demand forecast, attribution analysis, LTV health, and then writes an integrated diagnosis.

The new product already has most of these data skills as Rust Agent skills. This slice re-implements the old product idea inside the new product framework instead of copying the old Python runtime.

## Goals

1. Make `data_comprehensive_diagnosis` a native deterministic Rust Agent skill.
2. Compose existing data skill outputs in-process:
   - `data_dashboard`
   - `data_funnel_analysis`
   - `data_anomaly_diagnosis`
   - `data_customer_segmentation`
   - `data_refund_decomposition`
   - `data_demand_forecast`
   - `data_attribution_analysis`
   - `data_ltv_calculator`
3. Return an honest degraded report when some required inputs are missing.
4. Materialize successful or partial outputs as a workspace artifact.
5. Avoid direct platform, warehouse, or old Python access.

## Input

`data_comprehensive_diagnosis` accepts:

- `days`: integer, optional, default `30`
- `focus`: string, optional, one of `growth`, `profit`, `retention`, `risk`, `all`
- dashboard metrics such as `gmv`, `previous_gmv`, `orders`, `visitors`, `clicks`, `ad_spend`, `refund_amount`, `gross_profit`, `target_gmv`
- `values`: array of numbers for anomaly, trend, seasonality, and demand forecast
- `channels`: array of channel objects for ROI and attribution
- `items`: array of refund issue objects
- customer metrics such as `total_customers`, `avg_order_value`, `avg_orders_per_customer`, `recency_days`, `cac`, `purchase_frequency`, `gross_margin`, `monthly_retention`

## Output Contract

On full or partial diagnosis:

- `status=success`
- `has_data=true` when at least one real user-provided sub-diagnosis succeeds
- `has_partial_data=true` when some sub-diagnoses are deferred
- `adapter_status=manual_input` or `partial_manual_input`
- `data_readiness=user_provided` or `partial_user_input`
- `summary`
- `整体健康评分`
- `综合诊断报告`
- `分项诊断`
- `核心发现`
- `风险预警`
- `增长机会`
- `7天行动计划`
- `缺失数据`

When no useful user input exists:

- `status=deferred`
- `skill=data_comprehensive_diagnosis`
- `adapter_status=user_input_required`
- `data_readiness=missing_user_input`
- `deferred_reason`
- no workspace artifact is created

## Artifact

Successful or partial `data_comprehensive_diagnosis` output maps to:

- `contentCategory=data`
- `artifactType=data_comprehensive_diagnosis_report`
- title `Data comprehensive diagnosis report`

## LLM

This slice does not call LLM directly. LLM-owned synthesis remains in the Agent runtime and the final answer layer. The skill output is deterministic structured evidence that the runtime can cite.

## Validation

- `cargo fmt --all`
- `cargo fmt --all --check`
- `cargo test -p agent --lib data_comprehensive_diagnosis -- --nocapture`
- `cargo test -p service content_artifact_builds_from_data_comprehensive_skill_output -- --nocapture`
- `cargo check -p agent -p service`
- `cargo clippy -p agent -p service`
