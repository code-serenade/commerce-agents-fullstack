# Post 8-136 Backend Design: Learning Teaching Explain Skill

## 1. Requirement Summary

The old product contains a visible teaching / learning mode. Typical requests include:

- Explain a commerce concept such as ROI vs ROAS.
- Give a 30-second conclusion first, then explain why and how.
- Include common pitfalls, examples, and a self-check question.
- Keep teaching separate from high-risk execution.

The new product should migrate this as a real backend Agent capability first, not as a frontend-only prompt or fake mode.

## 2. Business Understanding

This slice adds one deterministic Agent skill:

```text
learning_teaching_explain
```

It turns explicit teaching / explanation intent into a structured commerce learning output:

- 30-second conclusion
- plain-language term explanation
- why it matters
- how to apply it
- commerce scenario example
- common pitfalls
- self-check question
- next practice task

The skill is native and deterministic. It does not call external LLM, external search, platform APIs, or user data stores.

## 3. Data Model

No new table is required.

Existing persistence is reused:

- Agent SSE `skill_requested`
- Agent SSE `skill_result`
- Agent skill run persistence
- Workspace content artifact materialization

Successful output can become a workspace artifact with:

```text
artifactType = learning_teaching_explain
contentCategory = learning
```

## 4. API Design

No new HTTP API is added.

The skill is exposed through existing Agent SSE and skill runtime contracts. Existing events remain:

- `skill_requested`
- `skill_result`
- `workspace_artifact_created`

## 5. Auth And User Isolation

No new auth path is added.

When the skill result is materialized as a workspace artifact, existing workspace service ownership checks continue to apply:

- workspace belongs to current `user_id`
- run belongs to current workspace
- artifact is written under the current user and workspace

## 6. Product Context Boundary

The skill may receive explicit product context only when this turn selected a product through existing context metadata.

Rules:

- If `product_ref=product:{uuid}` is present, output may include product-specific learning examples.
- If no product is selected, output stays generic and does not inject historical, preset, or demo products.
- No default product such as any previous coffee sample can be inserted.

## 7. Skill Input

```json
{
  "topic": "ROI和ROAS的区别",
  "learning_level": "undergraduate",
  "scenario": "电商投放复盘",
  "product_ref": "product:{uuid}",
  "product_name": "可选，只有显式选品时传入",
  "platform": "可选",
  "include_self_check": true
}
```

## 8. Skill Output

The skill returns:

```json
{
  "status": "success",
  "adapter_status": "manual_input",
  "data_readiness": "user_provided",
  "summary": "教学解释已生成",
  "教学主题": "...",
  "教学层级": "...",
  "30秒结论": "...",
  "术语白话解释": [...],
  "为什么": [...],
  "怎么做": [...],
  "场景示例": "...",
  "常见误区": [...],
  "自检问题": {...},
  "下一步练习": [...]
}
```

## 9. Worker Autosuggest

Worker autosuggest should create `learning_teaching_explain` when text includes teaching / explanation intent:

- 教学
- 讲解
- 解释
- 什么是
- 区别
- 原理
- 为什么
- 怎么做
- 常见误区
- 自检

The autosuggest path must still pass role skill governance. The skill is allowed for chat and all commerce roles because teaching mode is a cross-role explanation capability.

## 10. Validation Plan

Run:

```bash
cargo fmt --all
cargo test -p agent learning
cargo test -p agent keeps_roles_from_requesting_unowned_skills
cargo test -p agent executor_runs_representative_native_categories
cargo test -p service content_artifact_builds_from_learning_skill_output
cargo check -p agent -p service -p web-server
cargo clippy -p agent -p service -p web-server
git diff --check
```
