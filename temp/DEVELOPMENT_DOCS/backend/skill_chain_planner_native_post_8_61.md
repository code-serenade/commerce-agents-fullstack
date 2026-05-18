# Post 8-61: Coordination Skill Chain Planner Native Hardening

## 1. Requirement Summary

The current migration strategy remains:

```text
旧产品能力 = 后端内核能力来源
Kimi Agent 集群 = 前端交互和视觉体验参考
新产品架构 = 唯一落地框架
```

This slice continues old-product skill migration by hardening the old `coordination_skill_chain_planner` capability inside the new Rust Agent framework.

The old product used this skill to help the agent decide which skill should run first, which skill should run next, and which dependencies should be loaded before generating final business output. The new product already has a Rust skill with the same ID, but the output is still too static for product use and frontend process panels.

## 2. Scope

In scope:

- Keep implementation inside `backend/crates/agent/src/skills/coordination.rs`.
- Rework `coordination_skill_chain_planner` output into a structured, frontend-consumable plan.
- Add deterministic scenario templates for common commerce tasks.
- Include prerequisites, execution phase, execution mode, role ownership, adapter/approval warnings, and availability filtering.
- Preserve the new product architecture: no Python copy, no handler-local LLM call, no database write, no platform execution.

Out of scope:

- Direct Taobao / Douyin / Qianchuan official platform adapters.
- Real external Agent HTTP calls.
- New HTTP API.
- Database schema changes.
- Frontend UI changes in this slice.

## 3. Data / Database Design

No new table is required.

The skill runs inside the existing Agent runtime and returns data through the existing `skill_result` SSE contract. It does not persist anything by itself. Downstream persistence remains owned by existing workspace run / artifact / task logic.

## 4. API / SSE Contract

No new endpoint is added.

Existing Agent `skill_result` output for `coordination_skill_chain_planner` will be richer:

```json
{
  "status": "success",
  "任务目标": "提升咖啡新品首周转化",
  "角色": "ops",
  "动作": "plan",
  "推荐执行链": [
    {
      "步骤": 1,
      "技能": "query_store_metrics",
      "原因": "加载当前业绩基准",
      "阶段": "data_loading",
      "负责角色": "data",
      "执行模式": "adapter_required",
      "风险等级": "medium",
      "前置依赖": [],
      "可用": true,
      "说明": "需要用户输入或外部数据 adapter；不可伪造成已读取平台实时数据"
    }
  ],
  "前置依赖说明": {
    "ops_execution_plan": ["query_store_metrics"]
  },
  "不可用技能": [],
  "审批提示": [],
  "adapter提示": ["query_store_metrics"],
  "执行建议": "先加载真实数据，再计算分析，最后生成内容或执行操作。",
  "总步骤数": 4,
  "规划来源": "Rust deterministic template"
}
```

## 5. Auth / User Isolation

No direct user data is read or written in this slice.

The skill only plans execution using request parameters and Agent context. Any future persistence or user-owned resource access must stay inside existing authenticated workspace/service boundaries.

## 6. Error and Degradation Rules

- Missing `goal` remains validation error through the existing skill input validation.
- Unknown role/action falls back to a generic but explicit commerce chain.
- If `skills_available` is provided, each recommended step is marked as available/unavailable; the planner does not pretend an unavailable skill can run.
- Adapter-required and approval-required steps are flagged. They are planning signals, not proof that the adapter has run.
- External Agent calls remain `pending_approval` and are not executed here.

## 7. Validation Plan

Run from `backend/`:

```bash
cargo fmt --all
cargo test -p agent coordination_skill_chain -- --nocapture
cargo test -p agent coordination_helpers -- --nocapture
cargo check -p agent
cargo clippy -p agent
git diff --check
```
