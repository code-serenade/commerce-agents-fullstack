# Post 8-18：Campaign draft 创建 API

## 状态

done

## 本轮定位

旧产品迁移主体计划已完成 `12/12`。本轮是主体计划后的第 18 个补强项：把旧产品中 Agent/执行动作可以创建营销活动草稿的能力，迁成新产品后端的独立 draft API。

## 产品内核变化

- 营销活动不再只有普通 CRUD，也有面向 Agent 输出和工作区交付物的草稿入口。
- 后端能把商品、工作区、交付物、目标、渠道、策略、预算等上下文沉淀到 campaign 的结构化字段中。
- 草稿默认仍是本地 draft，不直接平台执行，后续可接审批和平台 adapter。

## 后端变化

- 新增设计文档：`temp/DEVELOPMENT_DOCS/backend/campaign_draft_creation_post_8_18.md`。
- Service 层新增 `CampaignApi::create_campaign_draft`。
- Web-server 新增 `POST /api/app/campaigns/draft`。
- 复用已有 `campaigns`、`workspaces`、`workspace_artifacts` 表，不新增 migration。
- API 文档同步 `API_DOCS/app.md`。

## 前端变化

- `apps/web/src/api/commerce.ts` 新增 `createCampaignDraftApi`。
- 同步新增 `CampaignDraftResponse` 和 `CampaignDraftSourceResponse` 类型。
- 本轮不新增页面入口，只同步真实后端契约。

## 用户可感知变化

- 后续前端接入后，用户可以把 Agent 生成的投放/营销方案保存为营销活动草稿。
- 用户能从工作区交付物创建活动草稿，并保留来源引用。
- 草稿不会自动发布到平台，需要后续审批或平台执行能力接入。

## 当前验证

- [x] `cargo fmt`
- [x] `cargo fmt --check`
- [x] `cargo test -p service campaign_draft`
- [x] `cargo check -p service`
- [x] `cargo check -p web-server`
- [x] `cargo check`
- [x] `cargo clippy`
- [x] `pnpm -C apps/web rules`
- [x] `pnpm -C apps/web lint`
- [x] `pnpm -C apps/web test`
- [x] `pnpm -C apps/web build`
- [x] `pnpm rules`
- [x] `git diff --check`

## 后续缺口

- 还需要把 Agent 最终交付物卡片接到 `createCampaignDraftApi`。
- 还需要将草稿提交审批和平台执行 adapter 打通。
