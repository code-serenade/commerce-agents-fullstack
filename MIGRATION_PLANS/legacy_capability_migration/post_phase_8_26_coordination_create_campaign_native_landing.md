# Post 8-26：Coordination 创建营销活动草稿真实落地归档

## 本轮结论

本轮已把 `coordination_create_campaign` 从骨架状态推进为真实后端能力：

1. agent 侧已改为 native deterministic skill
2. skill 输出会带可持久化的 `campaign_creation` 结构
3. service 层已可把 skill result 幂等落成真实 `campaign draft`
4. SSE 已可回传 `campaign_draft_created`
5. 同时会生成 `campaign_draft` 类型的 `workspace_artifact`

## 本轮涉及文件

- `backend/crates/agent/src/skills/coordination.rs`
- `backend/crates/service/src/dto/workspace.rs`
- `backend/crates/service/src/api/workspace.rs`
- `backend/crates/web-server/src/handlers/sse.rs`
- `API_DOCS/sse.md`
- `temp/DEVELOPMENT_DOCS/backend/commerce_capability_migration_master_plan.md`
- `MIGRATION_PLANS/legacy_capability_migration/development_log.md`
- `MIGRATION_PLANS/legacy_capability_migration/skill_migration_matrix.md`

## 后端变化

### 1. agent skill

`coordination_create_campaign` 不再返回 `pending_approval` 骨架，而是返回：

- `status=success`
- `campaign_creation.should_persist=true`
- 名称 / 描述 / 类型 / 预算 / 商品 / 渠道 / 策略 / 时间范围
- 结构化 `draft` 与 `normalized_inputs`

### 2. service 持久化

新增了 skill result 到真实 `campaign draft` 的落地能力：

- 仅处理 `coordination_create_campaign`
- 仅处理 `status=success`
- 以 `request_id -> campaignId` 做幂等
- 成功后回写 `agent_skill_run.metadata.campaignId`
- 同时生成一个 `campaign_draft` artifact

### 3. SSE 事件

当活动草稿成功落地后，SSE 会新增：

- `campaign_draft_created`

同时还会发：

- `workspace_artifact_created`

该 artifact 是真实 `campaign_draft` 交付物，前端后续可以直接消费。

## 用户可感知变化

用户现在不只是收到“建议建一个活动”的回答。

如果当前问答过程命中 `coordination_create_campaign`：

- 当前工作区会新增真实 campaign draft
- 后续活动管理和工作区都可继续读取它
- SSE 过程里会看到真实活动草稿创建事件
- 同时会生成一个可预览、可导出的草稿 artifact

## 验证结果

- `cargo fmt` 通过
- `cargo check -p agent` 通过
- `cargo check -p service` 通过
- `cargo check -p web-server` 通过
- `cargo clippy` 通过
- `cargo test -p agent coordination_create_campaign_returns_native_campaign_draft_payload -- --nocapture` 通过
- `cargo test -p service campaign_draft_builds_from_coordination_skill_output -- --nocapture` 通过

## 当前意义

这轮不是前端补一个“创建活动”按钮，而是把“活动方案建议”真正推进成“后端可治理的活动草稿”。

它让新产品更接近“用户在指挥电商虚拟员工产出并沉淀业务方案”，而不是只得到一段文本回答。
