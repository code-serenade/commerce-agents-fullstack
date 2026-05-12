# Post 8-42 Engineering Native Reports

## 需求摘要

继续按旧产品能力迁移计划推进，保持后端优先，并且必须落在同事给的新产品框架内。本切片处理工程岗位能力：把旧产品中架构评估、Bug 排查、性能优化、技术选型和 SLA 监控能力，迁成新产品 Rust Agent skill 的真实结构化输出，并接入现有 workspace artifact 沉淀链路。

## 后端设计

不新增数据库表，不新增 HTTP 路由，不新增独立 Agent 框架。

落点沿用现有框架：

- `backend/crates/agent/src/skills/engineering.rs`
- `backend/crates/service/src/api/workspace.rs`
- `WorkspaceApi::materialize_agent_content_artifact`
- Agent SSE 中已有的 `skill_result -> workspace_artifact_created` 持久化流程

## 能力边界

本阶段不读取真实代码仓库、日志、APM、监控系统或外部基准数据。所有工程报告均基于用户输入和内置确定性规则生成，并在输出中明示：

- `status=success`
- `has_data=true`
- `adapter_status=manual_input`
- `data_readiness=user_provided` 或 `calculated_from_defaults`
- `data_boundary`
- `summary`

`engineering_system_check` 继续保持 `deferred` / `pending_approval`，因为真实平台 API 健康检查需要平台 adapter 和审批闭环，不能伪造成已执行。

## Artifact 设计

新增工程类内容交付物映射：

- `engineering_arch_review` -> `engineering_arch_review_report`
- `engineering_bug_analysis` -> `engineering_bug_analysis_report`
- `engineering_perf_optimize` -> `engineering_perf_plan`
- `engineering_tech_selection` -> `engineering_tech_selection_report`
- `engineering_sla_monitor` -> `engineering_sla_report`

Artifact 内容继续采用 markdown 包裹完整 JSON 输出，metadata 继续使用 `kind=agent_generated_content`，`contentCategory=engineering`。

## 鉴权与 API

不新增 API。鉴权、workspace 归属校验和 artifact 写入权限沿用现有 `WorkspaceApi`。

需要同步根目录 `API_DOCS/`，说明工程类 skill 成功输出会创建工程交付物。

## 验证计划

- `cargo fmt`
- `cargo test -p agent engineering_ -- --nocapture`
- `cargo test -p service content_artifact_builds_from_engineering_ -- --nocapture`
- `cargo check -p agent`
- `cargo check -p service`
- `cargo check -p web-server`
- `cargo clippy -p agent -p service -p web-server`
- `git diff --check`
