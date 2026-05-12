# Post 8-19：产品生命周期状态 API

## 状态

done

## 本轮定位

旧产品能力迁移主体计划已完成 `12/12`。本轮是主体计划后的第 19 个补强项：把旧产品商品生命周期 `draft -> listing -> active -> archived` 迁成新产品后端的真实 API 能力。

## 产品内核变化

- 商品不再只有 `active / archived` 两个粗粒度状态。
- 商品生命周期被明确拆成选品、上架准备、运营、归档四个阶段。
- Agent 和前端可从同一套后端状态机读取当前阶段、可执行下一步和上下文。

## 后端变化

- 复用现有 `products.status` 字段，不新增数据库表。
- 扩展 ProductStatus：`draft`、`listing`、`active`、`archived`。
- 新增生命周期选项、单商品生命周期查询、生命周期流转 API。
- 生命周期流转会把 `lastTransition` 和最近流转历史写入 `products.metadata.lifecycle`。

## 前端变化

- `apps/web/src/api/commerce.ts` 同步生命周期响应类型。
- 新增生命周期 options、detail、transition 三个 API adapter。
- 本轮不新增页面入口，后续商品页可直接接入这些真实后端契约。

## 用户可感知变化

- 后续前端接入后，用户能在商品页看到商品处于“选品中 / 上架中 / 运营中 / 已归档”的哪个阶段。
- 用户可以按后端允许的路径推进商品阶段，而不是前端随意改状态。
- Agent 后续回答商品问题时可以读取生命周期阶段，给出更贴合当前阶段的建议。

## 当前验证

- [x] `cargo fmt`
- [x] `cargo fmt --check`
- [x] `cargo test -p service product_lifecycle`
- [x] `cargo check -p service`
- [x] `cargo check -p web-server`
- [x] `cargo check`
- [x] `cargo clippy`
- [x] `pnpm -C frontend/apps/web rules`
- [x] `pnpm -C frontend/apps/web lint`
- [x] `pnpm -C frontend/apps/web test`
- [x] `pnpm -C frontend/apps/web build`
- [x] `pnpm -C frontend rules`
- [x] `git diff --check`

## 后续缺口

- 还需要把商品页 UI 接到生命周期 API，展示阶段和下一步动作。
- 还需要让 Agent task graph 在商品相关任务中消费 lifecycle agentContext。
