# Post 8-125：客服平台规则搜索链路

## 状态

done

## 本次目标

把旧产品客服处理里隐含的最新平台规则、售后政策、赔付标准、运费规则、投诉处理和平台介入规则查询能力，迁成新产品框架中的可见 skill 链路。用户明确提出平台规则或最新政策诉求时，客服 native skill 先完成本地 SOP / 话术 / 升级 / 退换货处理，再追加统一 `search_platform_policy`。

## 产品内核变化

- 客服岗位不再把平台规则需求隐藏进单个 native 输出，而是拆成“客服处理 native skill + 平台规则搜索 skill”两步。
- 普通客服处理仍只走 native skill，避免没有规则诉求时误触发搜索。
- 平台规则搜索继续遵守统一 search adapter 降级契约：provider 未配置、失败或无可信来源时，不伪造平台政策、赔付标准或售后规则。
- 商品上下文仍是显式选择才进入链路；未选择商品时，不从历史、预设商品、测试商品或默认商品推断商品上下文。

## 后端变化

- 修改 `backend/crates/agent/src/agent/worker/runner.rs`。
- 新增客服平台规则编排：
  - `service_ticket_handler` -> `search_platform_policy(policy_type=general)`
  - `service_faq_playbook` -> `search_platform_policy(policy_type=general)`
  - `service_escalation_flow` -> `search_platform_policy(policy_type=general)`
  - `service_return_handler` -> `search_platform_policy(policy_type=general)`
- 复用已有 SSE skill 事件、搜索 adapter、搜索证据 artifact 和内容 artifact 机制，没有新增 HTTP API 或数据库表。

## 前端 / 用户可感知变化

- 用户说“客户要求退款退货，处理售后，并联网查抖音最新退换货规则和赔付标准”时，过程区会看到 `service_return_handler` 和 `search_platform_policy` 两个步骤。
- 用户说“整理售后 FAQ，并查淘宝最新七天无理由和运费险规则”时，过程区会看到 FAQ 话术库和平台规则搜索两个步骤。
- 用户说“主管升级并查最新平台介入规则”时，过程区会看到升级处理和平台规则搜索两个步骤。
- 用户只做普通售后、工单、FAQ 或升级处理时，仍只看到 native 客服处理步骤。

## 文档同步

- `API_DOCS/sse.md`：补充客服 native + 平台规则搜索双 skill 链路。
- `API_DOCS/app.md`：补充客服内容 artifact 与平台规则搜索证据 artifact 的并行沉淀关系。
- `MIGRATION_PLANS/legacy_capability_migration/skill_migration_matrix.md`：补充 Post 8-125 客服能力矩阵。
- `MIGRATION_PLANS/legacy_capability_migration/development_log.md`：记录本次开发。
- `MIGRATION_PLANS/legacy_capability_migration/phase_acceptance_log.md`：记录本次验收。

## 验收记录

- [x] 客服平台规则搜索链路按新产品 Agent worker 框架实现。
- [x] 普通客服 native skill 不因没有平台规则诉求而触发搜索。
- [x] 平台规则查询使用已有 `search_platform_policy` skill。
- [x] 未显式选择商品时，不注入默认商品上下文。
- [x] 定向 agent 测试覆盖平台规则链路、显式选品上下文和 native-only 回归。

## 后续缺口

- 真实平台规则 provider 的配置、部署和质量优化仍属于外部搜索能力的后续事项。
- 前端可继续把 native + search 双 skill 链路展示为可折叠过程卡片和来源面板。
