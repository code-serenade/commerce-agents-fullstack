# Post 8-63：店铺指标查询 skill 自然触达补强

状态：done

## 本次目标

- 延续 Post 8-62 的后端能力，让 `query_store_metrics` 不只是在模型手写工具调用时可用。
- 用户用自然语言说“查询店铺指标 / 加载店铺数据 / 店铺指标基线 / GMV、订单、UV”等内容时，后端能自动补偿这个 skill。
- 保持真实数据边界：只解析用户明确提供的指标；没有指标时仍走 deferred，不伪造平台数据。

## 本次修改

- 后端 agent：在 worker autosuggest 层新增 `query_store_metrics` 触达逻辑。
- 后端 agent：自然语言指标可解析为 skill input：`gmv`、`orders`、`visitors`、`clicks`、`ad_spend`、`refund_amount`、`gross_profit`。
- 后端 agent：支持中文金额单位 `万/千/亿`，例如 `GMV18万` 会转成 `180000`。
- 后端 agent：只有用户明确说了抖音、淘宝、天猫、京东、拼多多、快手时才带平台；否则 `platform=all`，避免默认塞淘宝。
- 后端 agent：保留更具体数据 skill 的优先级，例如店铺大盘仍走 `data_dashboard`，竞品仍走 `data_competitor_analysis`。
- 迁移文档和 SSE 文档同步。

## 用户可感知变化

- 用户可以直接说“查询店铺指标：GMV18万，订单1800，UV3.6万”，系统后端会自动调用店铺指标查询 skill。
- 用户可以说“先加载店铺指标作为运营复盘的数据基线”，系统会返回缺数据/待 adapter 的真实状态，而不是沉默或走错普通问答。
- 如果用户没指定平台，系统不会默认当作淘宝；只有明确说“抖音店铺指标”等，才会带入对应平台。
- 看板、竞品、退款、趋势等更具体问题仍进入对应专门 skill，不会被指标查询抢走。

## 当前验证

- [x] 后端设计文档：`temp/DEVELOPMENT_DOCS/backend/query_store_metrics_reachability_post_8_63.md`
- [x] Agent 单测：显式指标自然触达
- [x] Agent 单测：无指标加载意图自然触达
- [x] Agent 单测：显式平台识别
- [x] Agent 单测：不覆盖店铺大盘看板意图

## 后续缺口

- 真实 `store_metrics` service/adapter 仍待实现。
- 前端过程面板后续应把该 skill 展示为数据加载步骤。
- 这轮仍不接官方平台 API，不读取数据库。
