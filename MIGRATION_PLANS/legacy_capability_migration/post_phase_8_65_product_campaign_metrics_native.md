# Post 8-65：商品 / 活动指标查询 native skill

## 1. 本轮定位

继续迁移旧产品指标仓能力。前面已经完成店铺级 `query_store_metrics`，本轮把商品级和活动级指标查询补进新产品 Rust Agent 框架。

本轮仍以后端为主，不新增前端假入口，不接淘宝、抖音、千川官方 API。

## 2. 修改内容

- 后端 agent：新增 `query_product_metrics`。
- 后端 agent：新增 `query_campaign_metrics`。
- 后端 agent：两个新 skill 都走 `SkillExecutorSet`、角色白名单和既有 SSE `skill_result`。
- 后端 agent：有显式指标时输出 `status=success`、`adapter_status=manual_input`。
- 后端 agent：没有显式指标时输出 `status=deferred`，分别标记 `product_metrics_required` / `campaign_metrics_required`。
- Worker autosuggest：新增商品指标和活动指标自然语言触达。
- 协同 skill chain：把新旧指标 skill 规范进数据加载阶段。
- API 文档：更新 `API_DOCS/sse.md`。
- 迁移文档：更新矩阵、开发日志、验收记录和总计划。

## 3. 用户可感知变化

用户可以直接问：

```text
查询商品指标：冷萃咖啡 GMV9.6万，订单1200，UV2.4万，广告费2.4万，库存8
```

后端会进入 `query_product_metrics`，输出商品维度的转化率、CTR、客单价、ROI、退款率、毛利率、库存风险和后续动作。

用户也可以直接问：

```text
复盘618活动数据：预算5万，活动花费4.2万，GMV21万，订单1800，曝光18万，点击7200
```

后端会进入 `query_campaign_metrics`，输出活动维度的预算使用率、CTR、转化率、ROAS、ROI、退款率、风险和动作建议。

如果用户没有提供真实指标，系统会明确返回缺数据 / 待 adapter，不会假装读取平台实时数据。

## 4. 边界

- 本轮不读取数据库。
- 本轮不接官方平台 API。
- 本轮不默认注入任何商品。
- 本轮不创建假指标报告 artifact。
- 真实商品 / 活动指标仓读取后续应通过 service/repo adapter 设计。

## 5. 验证

- [x] `cargo fmt --all`
- [x] `cargo test -p agent query_product_metrics -- --nocapture`
- [x] `cargo test -p agent query_campaign_metrics -- --nocapture`
- [x] `cargo test -p agent autosuggests_product_and_campaign_metrics -- --nocapture`

