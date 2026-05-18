# Post 8-95 Ops Ad Fatigue Product Ref

## 需求摘要

继续按“旧产品能力 = 后端内核能力来源，Kimi Agent 集群 = 前端体验参考，新产品架构 = 唯一落地框架”的路线迁移旧产品运营能力。本切片只处理广告疲劳检测：

- `ops_ad_fatigue_detector` 要支持新产品 `product_ref=product:{uuid}` 契约。
- 用户手动选择商品后，广告疲劳 / 素材疲劳 / CTR 衰减 / 曝光频次 / ROAS 健康等问法需要把本轮选中商品引用带入 skill input。
- 用户没有手动选择商品时，不允许注入历史商品、默认商品或空 `product_ref`。
- 本阶段只解析用户本轮显式提供的渠道广告指标，不读取或伪造淘宝、抖音、千川、广告平台或实时数仓数据。

## 后端设计

### 数据表

本切片不新增数据库表，不修改 migration。

原因：

- 广告疲劳检测仍是 `agent` crate 的 native skill。
- 成功结果已通过既有 workspace artifact 物化链路沉淀为 `ops_ad_fatigue_report`。
- skill run 历史已由 `agent_skill_runs` 统一记录，不需要为本 skill 建第二套审计表。

### Skill 输入

扩展 `ops_ad_fatigue_detector` schema：

```json
{
  "channels": [
    {
      "name": "抖音",
      "ctr_baseline": 3.2,
      "ctr_recent": 2.4,
      "impressions_per_user": 6,
      "budget": 5000,
      "roas": 3.2
    }
  ],
  "total_budget": 9000,
  "platform": "抖音",
  "product_ref": "product:11111111-1111-1111-1111-111111111111"
}
```

`product_ref` 是可选字段，只能来自当前 Agent task metadata 中的显式选品上下文。

### Skill 输出

成功输出新增：

```json
{
  "商品引用": "product:11111111-1111-1111-1111-111111111111"
}
```

未提供 `product_ref` 时输出 `null`。缺少 `channels` 时继续返回 `deferred / missing_user_input`，不会创建假广告疲劳报告。

### Worker 自动补偿

`ops` worker 在模型没有显式 `skill_requests` 时，可根据以下意图自动生成 `ops_ad_fatigue_detector` 请求：

- 广告疲劳
- 素材疲劳
- 创意疲劳
- CTR 衰减
- 点击率衰减
- 频次过高
- 曝光频次
- ROAS 健康

可从本轮文本解析：

- 渠道：抖音、淘宝、天猫、京东、拼多多、小红书、快手、直通车、千川等。
- 渠道指标：`CTR基准 / 近期CTR / 曝光频次 / 预算 / ROAS`。
- 总预算：`总预算 / 广告总预算 / total_budget`。
- 平台：优先显式选品平台，否则从文本推断。

## 鉴权和用户隔离

本切片不新增 HTTP API，不直接访问用户数据表。用户隔离继续依赖现有 SSE / Agent session 链路：

- 只有本轮显式 `context.selected_summary` 才能形成 `product_ref`。
- 未选商品时 worker 不写空 `product_ref`。
- 不从历史对话、预设商品或旧测试数据中推断商品。

## API / SSE

不新增接口。同步更新 `API_DOCS/sse.md` 中 `skill_result` 和 ops artifact 说明：

- 广告疲劳检测支持 `product_ref=product:{uuid}`。
- 自然语言广告疲劳意图可触发 `ops_ad_fatigue_detector`。
- 缺少渠道指标时仍为 `deferred`，不伪造平台数据。

## 验证计划

```text
cargo fmt --all
cargo test -p agent ops_ad_fatigue_detector -- --nocapture
cargo test -p agent autosuggests_ad_fatigue_detector_with_explicit_channel_rows -- --nocapture
cargo test -p agent selected_product_context_feeds_ad_fatigue_detector_ref -- --nocapture
cargo test -p agent ad_fatigue_detector_without_selection_has_no_empty_product_ref -- --nocapture
cargo test -p agent autosuggests_more_specific_ops_skills_by_intent -- --nocapture
cargo fmt --all --check
cargo check -p agent
cargo clippy -p agent
git diff --check
```
