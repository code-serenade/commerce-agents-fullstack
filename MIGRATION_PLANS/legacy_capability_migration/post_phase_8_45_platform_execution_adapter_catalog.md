# Post 8-45：平台执行 Adapter 能力目录

## 本轮结论

本轮把高风险平台写操作从“只在审批和执行结果里看到失败原因”推进到“平台连接页可以先看到 adapter 能力目录和当前可执行边界”。

当前仍不执行真实淘宝/JD/PDD/抖音写接口，也不伪造成功。它解决的是能力透明度：用户在配置平台连接时，可以提前看到哪些平台写动作受治理、需要哪些参数、为什么当前还不能执行。

## 涉及文件

- `temp/DEVELOPMENT_DOCS/backend/platform_execution_adapter_catalog_post_8_45.md`
- `backend/crates/service/src/dto/platform_connect.rs`
- `backend/crates/service/src/api/platform_connect.rs`
- `backend/crates/web-server/src/dto/app.rs`
- `backend/crates/web-server/src/handlers/app/platform_connect.rs`
- `backend/crates/web-server/src/routes/app/platform_connect.rs`
- `frontend/apps/web/src/api/commerce.ts`
- `frontend/apps/web/src/pages/site/commerce/platform-connect/use-platform-queries.ts`
- `frontend/apps/web/src/pages/site/commerce/platform-connect/index.tsx`
- `frontend/apps/web/src/pages/site/commerce/platform-connect/components/platform-execution-adapters-panel.tsx`
- `frontend/apps/web/src/pages/site/commerce/platform-connect/components/platform-panels.tsx`
- `frontend/apps/web/src/pages/site/commerce/components/commerce-status-utils.ts`
- `frontend/apps/web/lang/en/messages.ts`
- `frontend/apps/web/lang/zhCN/messages.ts`
- `API_DOCS/app.md`
- `temp/DEVELOPMENT_DOCS/backend/commerce_capability_migration_master_plan.md`
- `MIGRATION_PLANS/legacy_capability_migration/development_log.md`
- `MIGRATION_PLANS/legacy_capability_migration/phase_acceptance_log.md`
- `MIGRATION_PLANS/legacy_capability_migration/skill_migration_matrix.md`
- `MIGRATION_PLANS/legacy_capability_migration/api_migration_checklist.md`
- `MIGRATION_PLANS/legacy_capability_migration/frontend_experience_checklist.md`

## 后端变化

新增 `PlatformConnectApi::execution_adapter_catalog`，按当前 App 用户读取平台连接状态，并返回平台执行 adapter 目录。

当前目录覆盖：

- `platform_update_price`：更新商品或 SKU 价格。
- `platform_update_inventory`：更新商品或 SKU 库存。
- `platform_sync_products`：同步平台商品资料。

每个动作都返回：

- 风险等级：`L3`。
- 是否需要审批：`approvalRequired=true`。
- 执行模式：`adapter`。
- adapter 状态：当前固定为 `adapter_not_implemented`。
- 当前是否可执行：当前固定为 `canExecuteNow=false`。
- 必填参数、选填参数和幂等键字段。
- readiness 和不可执行原因。

## 前端变化

平台连接页新增只读“平台执行能力”面板，展示平台、动作、连接状态、adapter 状态、readiness 和必要参数。

前端只消费后端真实字段，没有新增执行按钮，没有把 adapter 未实现显示成可执行。

## 用户可感知变化

用户进入平台连接页时，可以看到当前产品已经纳入治理的平台写能力，以及每个平台动作为什么还不能执行。

如果平台连接缺失、连接被禁用或真实 adapter 未实现，页面会直接展示后端返回的原因。用户不会误以为当前已经能真实改价、改库存或同步平台商品。

## 当前边界

- 当前真实平台写 adapter 仍未实现。
- 当前不会返回 `ready` 或 `canExecuteNow=true`。
- 当前不新增数据库表，不新增第二套审批系统。
- 后续接入真实平台 adapter 时，应复用该目录、审批、执行队列和审计闭环。
