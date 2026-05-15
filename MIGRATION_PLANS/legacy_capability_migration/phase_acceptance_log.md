# 阶段验收记录

## Post 8-127：上架/标题/详情文案搜索链路

结论：通过。

验收点：

- 运营岗位上架文案、Web 岗位标题生成和详情文案在普通问法下仍只进入 native skill。
- 用户明确要求最新 / 实时 / 联网 / 搜索 / 公开资料 / 标题规则 / 上架规则 / 违禁词 / 搜索量 / 热搜词 / 高点击 / 详情页趋势 / 买家痛点 / 真实评价时，会返回两个 skill 请求。
- 第一个请求保持 native skill，第二个请求为 `search_platform_policy`、`search_market_info` 或 `search_product_reviews`。
- 标题 / 发布 / 内容规则搜索链路使用 `search_platform_policy(policy_type=content)`。
- 搜索量 / 热搜词 / 标题趋势 / 详情页趋势搜索链路使用 `search_market_info(topic=general)`。
- 买家痛点 / 真实评价搜索链路使用 `search_product_reviews`，痛点 / 顾虑 / 差评语义会写入 `focus=negative`。
- 运营岗位 role catalog 已允许 `search_product_reviews`，不会出现 autosuggest 后运行时拒绝。
- 搜索步骤通过既有 `skill_requested` / `skill_result` 可见，失败或未配置时仍降级，不伪造来源。
- 显式商品上下文只来自本轮选择，未选择商品时不写空引用、不注入默认商品。
- Agent listing / web title-description / role catalog 测试、设计文档、API 文档和迁移归档已同步。

## Post 8-126：创意直播/软文趋势搜索链路

结论：通过。

验收点：

- 创意岗位直播话术 / 产品软文在普通问法下仍只进入 native skill。
- 用户明确要求最新 / 实时 / 联网 / 搜索 / 公开资料 / 直播间玩法 / 热门直播间 / 转化节奏 / 爆款文章 / 公众号热门风格时，会返回两个 skill 请求。
- 第一个请求保持创意 native skill，第二个请求为 `search_trends`。
- `最多4条来源` 等搜索来源数量会写入 `max_results`，不会被当成直播时长。
- 产品软文意图优先于趋势捕捉，包含“爆款文章 / 热门内容”时不会误路由为 `creative_trend_catch`。
- 搜索步骤通过既有 `skill_requested` / `skill_result` 可见，失败或未配置时仍降级，不伪造来源。
- 显式商品上下文只来自本轮选择，未选择商品时不写空引用、不注入默认商品。
- Agent creative / search / role catalog 测试、设计文档、API 文档和迁移归档已同步。

## Post 8-118：Web/SEO 增长搜索链路

结论：通过。

验收点：

- Web 岗位 SEO 优化 / 关键词研究 / 页面转化分析在普通问法下仍只进入 native skill。
- 用户明确要求最新 / 实时 / 联网 / 搜索 / 搜索算法 / 流量规则 / 关键词竞争 / 搜索量 / CPC / 行业转化率基准时，会返回两个 skill 请求。
- 第一个请求保持 Web native skill，第二个请求为 `search_platform_policy`、`search_market_info` 或 `search_industry_benchmarks`。
- SEO 算法 / 流量规则搜索链路使用 `search_platform_policy(policy_type=algorithm)`。
- 关键词竞争 / 搜索量 / CPC 搜索链路使用 `search_market_info`。
- 页面转化率基准搜索链路使用 `search_industry_benchmarks(metric=conversion)`。
- 搜索步骤通过既有 `skill_requested` / `skill_result` 可见，失败或未配置时仍降级，不伪造来源。
- 显式商品上下文只来自本轮选择，未选择商品时不写空引用、不注入默认商品。
- Agent web / search 测试、设计文档、API 文档和迁移归档已同步。

## Post 8-117：创意内容趋势搜索链路

结论：通过。

验收点：

- 创意岗位短视频脚本 / 种草文案 / 内容排期 / 趋势捕捉在普通问法下仍只进入 native skill。
- 用户明确要求最新 / 实时 / 联网 / 搜索 / 爆款 / 热门话题 / 营销节点时，会返回两个 skill 请求。
- 第一个请求保持创意 native skill，第二个请求为 `search_trends` 或 `search_market_info`。
- 内容排期的营销节点搜索链路使用 `search_market_info`，且 creative 角色白名单允许该 skill。
- 搜索步骤通过既有 `skill_requested` / `skill_result` 可见，失败或未配置时仍降级，不伪造来源。
- “最多4条来源”只影响搜索 `max_results`，不污染短视频脚本时长。
- 显式商品上下文只来自本轮选择，未选择商品时不写空引用、不注入默认商品。
- Agent creative / search / role catalog 测试、设计文档、API 文档和迁移归档已同步。

## Post 8-116：数据竞品与价格弹性搜索链路

结论：通过。

验收点：

- 数据岗位竞品分析在用户要求实时 / 最新 / 联网 / 公开资料时会返回两个 skill 请求。
- 第一个请求为 `data_competitor_analysis`，第二个请求为 `search_competitor`。
- 数据岗位价格弹性分析在用户要求市场定价 / 竞品价格区间 / 实时价格时会返回两个 skill 请求。
- 价格弹性链路中的搜索 `focus=price`。
- 普通竞品分析和普通价格弹性分析仍只进入 native skill。
- 显式商品上下文只来自本轮选择，未选择商品时不写空引用、不注入默认商品。
- 搜索结果数量解析已修复中文标点 byte 边界问题。
- Agent data / search 测试、设计文档、API 文档和迁移归档已同步。

## Post 8-115：客服满意度行业基准搜索链路

结论：通过。

验收点：

- 客服岗位 DSR / NPS / 满意度分析在用户要求实时行业基准时会返回两个 skill 请求。
- 第一个请求保持客服 native skill，例如 `service_dsr_improvement` 或 `service_nps_driver_analysis`。
- 第二个请求为 `search_industry_benchmarks`，通过 SSE / skill run 作为独立步骤可见。
- 客服 role catalog 已允许 `search_industry_benchmarks`。
- 搜索基准 metric 已支持 `dsr`，NPS 仍走 `nps`。
- 显式商品上下文只来自本轮选择，未选择商品时不写空引用、不注入默认商品。
- Agent service / search 测试、设计文档、API 文档和迁移归档已同步。

## Post 8-114：外部搜索来源时效过滤

结论：通过。

验收点：

- App 外部搜索 API 会过滤超出请求时间窗口的可解析发布时间来源。
- Agent `search_*` skill 会过滤超出 `query.days` 的可解析发布时间来源。
- 过滤后仍有可信来源时保持 live，且 provenance / freshness 记录 stale 过滤数量。
- 过滤后没有可信来源时返回 degraded/deferred，不创建假搜索证据。
- Agent 搜索测试、Service 外部搜索测试、设计文档、API 文档和迁移归档已同步。

## Post 8-113：搜索自然语言结果数量解析

结论：通过。

验收点：

- worker autosuggest 能从中文搜索结果数量表达中提取 `max_results`。
- worker autosuggest 能从英文 `top N sources/results` 表达中提取 `max_results`。
- 六个搜索类 skill input builder 都会复用该数量解析。
- 结果数量会 clamp 到 `1..=12`。
- `最近30天` 等时间窗口不会被误识别为来源数量。
- Agent 搜索测试、设计文档、API 文档和迁移归档已同步。

## Post 8-112：搜索 Skill 结果数量输入契约

结论：通过。

验收点：

- 六个搜索类 skill 均声明可选 `max_results` 输入。
- Runtime 同时兼容 `max_results` 和 `maxResults`。
- 显式输入优先于 adapter 默认结果数。
- 结果数会 clamp 到 `1..=12`。
- `output.query.max_results` 与 provider 请求、live 来源截断、provenance、evidence 和搜索证据 artifact 保持一致。
- Agent 搜索测试、设计文档、API 文档和迁移归档已同步。

## Post 8-111：外部搜索结果数量约束

结论：通过。

验收点：

- App 外部搜索 API 会按本次 `maxResults` 截断 provider 返回后的可信来源。
- Agent `search_*` skill 会按 `output.query.max_results` 截断 provider 返回后的可信来源。
- `sources` / `data.items`、`provenance`、`evidence` 使用同一批截断后的来源，数量一致。
- provider 返回多余结果不会进入搜索证据 artifact 或后续上下文包。
- provider 未配置、失败或无可信 URL 来源时仍降级，不伪造实时来源。
- 后端测试、设计文档、API 文档和迁移归档已同步。

## Post 8-110：会话附件浏览器文本抽取状态对齐

结论：通过。

验收点：

- Agent Chat 上传附件后会调用已有后端抽取登记接口。
- 浏览器可读文本文件会登记 `status=extracted`，正文进入后端可信 metadata。
- 空文本会登记 `status=failed`，非文本文件会登记 `status=unsupported`。
- 抽取状态登记失败不会把上传本身显示为失败；只是不把附件标记为可读资料。
- 不做 OCR、PDF、Office、图片或 object storage 后台解析，不伪造正文。
- 前端测试、设计文档、API 文档和迁移归档已同步。

## Post 8-109：外部搜索来源归一化增强

结论：通过。

验收点：

- App 侧 `/api/app/external-search/search` 和 Agent `search_*` skill 使用一致的来源归一化语义。
- 支持更多常见 provider 来源数组结构，包括 `data.webPages.value`、`organic_results`、`results`、`sources` 和 `value`。
- 支持更多常见来源字段，包括 `headline`、`href`、`description`、`text`、`excerpt`、`datePublished`。
- 缺少可追溯 URL 或无法形成 domain 的弱来源不会进入 live evidence。
- provider 过滤后无可信来源时仍走 degraded/deferred，不伪造实时来源，不创建搜索证据 artifact。
- API 文档、后端设计文档和迁移归档已同步。

## Post 8-108：前端显式资料引用接入

结论：通过。

验收点：

- Agent Chat 输入区新增“资料”选择入口，入口来自当前会话真实已上传文件。
- 用户手动勾选资料后，前端发送 `conversation_file:{fileId}` 到 `metadata.context.contextRefs`。
- 商品引用和资料引用在前端统一去重合并，资料不会反向设置商品。
- 未勾选资料时不自动注入历史附件、商品素材、历史交付物或测试商品。
- 可读文件和仅元数据文件在界面中明确区分。
- 前端设计文档、迁移归档和 skill 矩阵已同步。

# 阶段验收记录

## Post 8-107：显式工作区来源上下文引用

结论：通过。

验收点：

- `contextRefs` 支持显式引用 `conversation_file:{uuid}` / `file:{uuid}`。
- `contextRefs` 支持显式引用 `product_asset:{uuid}` / `asset:{uuid}`。
- `contextRefs` 支持显式引用 `product_material:{uuid}` / `material:{uuid}`。
- 既有 `workspace_artifact:{uuid}` / `artifact:{uuid}` 继续可用。
- 显式来源进入 workspace context summary / pack 前会校验当前用户、active 状态和所属资源。
- 显式商品资产 / 商品素材只作为资料来源，不会反向设置本轮选中商品。
- API 文档和迁移归档已同步。

## Post 8-106：显式交付物上下文引用接入 Agent SSE

结论：通过。

验收点：

- `contextRefs` 支持显式引用 `workspace_artifact:{uuid}` / `artifact:{uuid}`。
- 显式 artifact 进入 workspace context summary / pack 前会校验当前用户、当前 workspace 和 active 状态。
- Agent SSE 默认仍只聚合当前 run artifact；历史 artifact 必须显式选择。
- `search_evidence` artifact 进入 pack 时继续使用可读摘要，避免原始搜索 JSON 直接污染上下文。
- API 文档和迁移归档已同步。

## Post 8-105：搜索证据进入上下文内容包的可读摘要

状态：done

验收项：

- [x] 后端设计文档已写入 `temp/DEVELOPMENT_DOCS/backend/`。
- [x] Service：`workspace_context_pack` 能识别 `search_evidence` artifact。
- [x] Service：搜索证据进入内容包时使用 `contentKind=search_evidence_summary`。
- [x] Service：摘要包含 query、显式商品上下文、evidence summary、source count、confidence、domains 和 Top sources。
- [x] Service：JSON 解析失败时回退到 artifact preview。
- [x] Service：不从 query 文本、历史或工作区摘要推断商品。
- [x] API：`API_DOCS/app.md` 与 `API_DOCS/sse.md` 已同步。
- [x] 前端：本轮不新增假入口。

当前说明：

- 本阶段解决的是“搜索证据可沉淀，但后续上下文复用时完整 JSON 太重、太吵”的问题。
- 本阶段只整理已存在的真实 live 搜索证据，不触发新搜索，不访问外部 provider。
- 本阶段让 Agent 连续追问更容易消费搜索证据，同时保留来源和降级边界。

## Post 8-104：搜索证据 Artifact 商品上下文沉淀

状态：done

验收项：

- [x] 后端设计文档已写入 `temp/DEVELOPMENT_DOCS/backend/`。
- [x] Service：`search_evidence` artifact 从 `output.query` 读取 `product_ref/product_name/product_category`。
- [x] Service：商品上下文存在时写入 `metadata.productContext.productRef/productName/productCategory`。
- [x] Service：商品上下文存在时冗余写入顶层 `metadata.productRef/productName/productCategory`。
- [x] Service：`preview` 增加 `Product:` 行，便于前端和用户区分指定商品搜索证据。
- [x] Service：未显式选商品时不写空商品字段，不从 query、历史或工作区摘要推断商品。
- [x] Service：仍只为真实 live 搜索来源创建 `search_evidence`，degraded/deferred 不创建假证据。
- [x] API：`API_DOCS/app.md` 与 `API_DOCS/sse.md` 已同步。
- [x] 前端：本轮不新增假入口。

当前说明：

- 本阶段解决的是“搜索 skill 已知道本轮选中商品，但搜索证据 artifact 刷新后丢失商品绑定关系”的问题。
- 本阶段只沉淀 `search_*` skill 已输出的显式商品上下文；不查询商品库、不跨用户读取商品资料、不从文本猜默认商品。
- 本阶段不改变外部搜索 provider 边界；未配置、失败或无来源时仍明确降级，不伪造联网结果。

## Post 8-103：搜索 skill 显式选品上下文

状态：done

验收项：

- [x] 后端设计文档已写入 `temp/DEVELOPMENT_DOCS/backend/`。
- [x] Worker autosuggest：`search_*` 自动补偿可读取 `Task` 中已校验的 `context.selected_summary`。
- [x] Worker autosuggest：识别“这个商品 / 本品 / 当前商品 / 所选商品 / selected product”等显式选品指代。
- [x] Worker autosuggest：本轮显式选品时，搜索 input 携带 `product_ref`、`product_name`、`product_category`。
- [x] Worker autosuggest：本轮显式选品且文本未指定平台时，搜索 platform 可使用选中商品平台。
- [x] Worker autosuggest：未选商品时不写空商品上下文字段，也不从历史或预设商品推断默认商品。
- [x] Agent search skill：输出 `query` 对象回显可选商品上下文。
- [x] API：`API_DOCS/sse.md` 已同步搜索 product context 契约。
- [x] 前端：本轮不新增假入口。

当前说明：

- 本阶段解决的是“外部搜索类 skill 能触发，但不知道用户本轮手动选择的是哪个商品”的问题。
- 本阶段只消费后端 service/web-server 已校验后注入的显式选品 metadata；agent crate 不直连数据库、不跨用户读取商品资料。
- 本阶段不改变真实搜索 provider 调用边界；未配置、失败或无来源时仍明确降级，不伪造联网结果。

## Post 8-102：运营执行计划 product_ref 契约

状态：done

验收项：

- [x] 后端设计文档已写入 `temp/DEVELOPMENT_DOCS/backend/`。
- [x] Agent：`ops_execution_plan` 成功输出回显 `商品引用`。
- [x] Worker autosuggest：运营岗位兜底执行计划使用结构化 input。
- [x] Worker autosuggest：本轮显式选品时执行计划 input 携带 `product_ref`。
- [x] Worker autosuggest：未选商品时不写空 `product_ref`。
- [x] API：`API_DOCS/sse.md` 已同步执行计划 product_ref 契约。
- [x] 前端：本轮不新增假入口。

当前说明：

- 本阶段解决的是“运营执行计划已可 native 执行，但商品引用回显和未选商品空引用控制不完整”的问题。
- 本阶段只使用用户本轮显式输入的商品选择和运营目标；未选商品时不会从历史摘要、预设商品或测试商品补默认引用。
- 本阶段不读取或伪造淘宝、抖音、千川、ERP、WMS、供应链或实时平台数据。

## Post 8-101：定价策略 product_ref 契约

状态：done

验收项：

- [x] 后端设计文档已写入 `temp/DEVELOPMENT_DOCS/backend/`。
- [x] Agent：`ops_pricing_strategy` schema 支持 `product_ref`。
- [x] Agent：成功定价策略输出回显 `商品引用`。
- [x] Worker autosuggest：定价 / 价格 / 售价 / 成本问法触发 `ops_pricing_strategy` 时使用结构化 input。
- [x] Worker autosuggest：本轮显式选品时定价策略 input 携带 `product_ref`。
- [x] Worker autosuggest：未选商品时不写空 `product_ref`。
- [x] Worker autosuggest：成本和竞品价格优先读取本轮显式标签。
- [x] API：`API_DOCS/sse.md` 已同步定价策略 product_ref 契约。
- [x] 前端：本轮不新增假入口。

当前说明：

- 本阶段解决的是“定价策略已可 native 执行，但商品引用和参数解析不够稳”的问题。
- 本阶段只使用用户本轮显式输入的商品选择、成本和竞品价格；未选商品时不会从历史摘要、预设商品或测试商品补默认引用。
- 本阶段不读取或伪造淘宝、抖音、千川、ERP、WMS、供应链或实时竞品价格数据。

## Post 8-100：库存规划 product_ref 契约

状态：done

验收项：

- [x] 后端设计文档已写入 `temp/DEVELOPMENT_DOCS/backend/`。
- [x] Agent：`ops_inventory_planning` schema 支持 `product_ref`。
- [x] Agent：成功库存规划输出回显 `商品引用`。
- [x] Worker autosuggest：库存 / 补货 / 备货问法触发 `ops_inventory_planning` 时使用结构化 input。
- [x] Worker autosuggest：本轮显式选品时库存规划 input 携带 `product_ref`。
- [x] Worker autosuggest：未选商品时不写空 `product_ref`。
- [x] Worker autosuggest：日均销量和供货周期优先读取本轮显式标签。
- [x] API：`API_DOCS/sse.md` 已同步库存规划 product_ref 契约。
- [x] 前端：本轮不新增假入口。

当前说明：

- 本阶段解决的是“库存规划已可 native 执行，但商品引用和参数解析不够稳”的问题。
- 本阶段只使用用户本轮显式输入的商品选择、日均销量和供货周期；未选商品时不会从历史摘要、预设商品或测试商品补默认引用。
- 本阶段不读取或伪造淘宝、抖音、千川、ERP、WMS、供应链或实时库存数据。

## Post 8-99：选品规划 product_ref 契约

状态：done

验收项：

- [x] 后端设计文档已写入 `temp/DEVELOPMENT_DOCS/backend/`。
- [x] Agent：`ops_assortment_planning` schema 支持 `product_ref`。
- [x] Agent：成功选品规划输出回显 `商品引用`。
- [x] Worker autosuggest：货盘 / SKU 组合 / 选品组合 / 组合规划问法触发 `ops_assortment_planning` 时使用结构化 input。
- [x] Worker autosuggest：本轮显式选品时选品规划 input 携带 `product_ref`。
- [x] Worker autosuggest：未选商品时不写空 `product_ref`。
- [x] Worker autosuggest：预算解析优先读取预算标签，避免把 `618`、`双11` 当预算。
- [x] API：`API_DOCS/sse.md` 已同步选品规划 product_ref 契约。
- [x] 前端：本轮不新增假入口。

当前说明：

- 本阶段解决的是“选品规划已可 native 执行，但商品引用和预算解析不够稳”的问题。
- 本阶段只使用用户本轮显式输入的商品选择、类目和预算；未选商品时不会从历史摘要、预设商品或测试商品补默认引用。
- 本阶段不读取或伪造淘宝、抖音、千川、ERP、WMS、供应链、平台热卖或实时库存数据。

## Post 8-98：渠道策略 product_ref 契约

状态：done

验收项：

- [x] 后端设计文档已写入 `temp/DEVELOPMENT_DOCS/backend/`。
- [x] Agent：`ops_channel_strategy` schema 支持 `product_ref`。
- [x] Agent：成功渠道策略输出回显 `商品引用`。
- [x] Worker autosuggest：渠道 / 投放渠道 / 流量渠道 / 渠道分配 / 渠道策略问法触发 `ops_channel_strategy` 时使用结构化 input。
- [x] Worker autosuggest：本轮显式选品时渠道策略 input 携带 `product_ref`。
- [x] Worker autosuggest：未选商品时不写空 `product_ref`。
- [x] Worker autosuggest：预算解析优先读取预算标签，避免把 `618`、`双11` 当预算。
- [x] API：`API_DOCS/sse.md` 已同步渠道策略 product_ref 契约。
- [x] 前端：本轮不新增假入口。

当前说明：

- 本阶段解决的是“渠道策略已可 native 执行，但商品引用和预算解析不够稳”的问题。
- 本阶段只使用用户本轮显式输入的商品选择、类目和预算；未选商品时不会从历史摘要、预设商品或测试商品补默认引用。
- 本阶段不读取或伪造淘宝、抖音、千川、平台渠道、广告账户或实时投放数据。

## Post 8-97：促销策划 product_ref 契约

状态：done

验收项：

- [x] 后端设计文档已写入 `temp/DEVELOPMENT_DOCS/backend/`。
- [x] Agent：`ops_promo_planning` schema 支持 `product_ref`。
- [x] Agent：成功促销策划输出回显 `商品引用`。
- [x] Worker autosuggest：促销 / 活动 / 大促 / 618 / 双11 问法触发 `ops_promo_planning` 时使用结构化 input。
- [x] Worker autosuggest：本轮显式选品时促销策划 input 携带 `product_ref`。
- [x] Worker autosuggest：未选商品时不写空 `product_ref`。
- [x] Worker autosuggest：预算解析优先读取预算标签，避免把 `618`、`双11` 当预算。
- [x] API：`API_DOCS/sse.md` 已同步促销策划 product_ref 契约。
- [x] 前端：本轮不新增假入口。

当前说明：

- 本阶段解决的是“促销策划已可 native 执行，但商品引用和预算解析不够稳”的问题。
- 本阶段只使用用户本轮显式输入的商品选择与预算；未选商品时不会从历史摘要、预设商品或测试商品补默认引用。
- 本阶段不读取或伪造淘宝、抖音、千川、平台活动或实时投放数据。

## Post 8-96：ABC-XYZ 库存分类 product_ref 契约

状态：done

验收项：

- [x] 后端设计文档已写入 `temp/DEVELOPMENT_DOCS/backend/`。
- [x] Agent：`ops_abc_xyz_classifier` schema 支持 `product_ref`。
- [x] Agent：成功 ABC-XYZ 库存分类输出回显 `商品引用`。
- [x] Worker autosuggest：ABC-XYZ / ABC分类 / XYZ分类 / 九宫格库存分类 / SKU分层问法触发 `ops_abc_xyz_classifier` 时使用结构化 input。
- [x] Worker autosuggest：本轮显式选品时 ABC-XYZ input 携带 `product_ref`。
- [x] Worker autosuggest：本轮显式 SKU GMV 和历史销量会进入 ABC-XYZ input。
- [x] Worker autosuggest：未选商品时不写空 `product_ref`。
- [x] API：`API_DOCS/sse.md` 已同步 ABC-XYZ product_ref 契约。
- [x] 前端：本轮不新增假入口。

当前说明：

- 本阶段解决的是“ABC-XYZ 已可 native 执行，但自然语言触达和商品引用没有打通”的问题。
- 本阶段只使用用户本轮显式输入的 SKU / GMV / 历史销量；缺 SKU 明细时仍使用确定性演示数据并明示 `calculated_from_defaults`。
- 本阶段不读取或伪造淘宝、抖音、WMS、ERP、平台库存或实时数仓数据。

## Post 8-95：广告疲劳检测 product_ref 契约

状态：done

验收项：

- [x] 后端设计文档已写入 `temp/DEVELOPMENT_DOCS/backend/`。
- [x] Agent：`ops_ad_fatigue_detector` schema 支持 `product_ref`。
- [x] Agent：成功广告疲劳检测输出回显 `商品引用`。
- [x] Worker autosuggest：广告疲劳 / 素材疲劳 / CTR 衰减 / 点击率衰减 / 曝光频次 / ROAS 健康问法触发 `ops_ad_fatigue_detector` 时使用结构化 input。
- [x] Worker autosuggest：本轮显式选品时广告疲劳 input 携带 `product_ref`。
- [x] Worker autosuggest：本轮显式渠道 CTR 基准、近期 CTR、频次、预算和 ROAS 会进入广告疲劳 input。
- [x] Worker autosuggest：未选商品时不写空 `product_ref`。
- [x] API：`API_DOCS/sse.md` 已同步广告疲劳 product_ref 契约。
- [x] 前端：本轮不新增假入口。

当前说明：

- 本阶段解决的是“广告疲劳检测已有 native skill，但自然语言触达和商品引用没有打通”的问题。
- 本阶段只使用用户本轮显式输入的广告指标，不读取或伪造淘宝、抖音、千川、广告平台或实时数仓数据。
- 缺少渠道指标时仍返回 `deferred / missing_user_input`，不会创建假广告疲劳交付物。

## Post 8-94：库存优化 product_ref 契约

状态：done

验收项：

- [x] 后端设计文档已写入 `temp/DEVELOPMENT_DOCS/backend/`。
- [x] Agent：`ops_inventory_optimizer` schema 支持 `product_ref`。
- [x] Agent：成功库存优化输出回显 `商品引用`。
- [x] Worker autosuggest：库存优化 / 库存健康 / 安全库存 / 再订货点 / EOQ / ROP 问法触发 `ops_inventory_optimizer` 时使用结构化 input。
- [x] Worker autosuggest：本轮显式选品时库存优化 input 携带 `product_ref`。
- [x] Worker autosuggest：本轮显式 SKU 月销量、成本、库存数量、供货周期和服务水平会进入库存优化 input。
- [x] Worker autosuggest：未选商品时不写空 `product_ref`。
- [x] Worker autosuggest：普通补货点问题仍触发 `ops_inventory_planning`。
- [x] API：`API_DOCS/sse.md` 已同步库存优化 product_ref 契约。
- [x] 前端：本轮不新增假入口。

当前说明：

- 本阶段解决的是“库存优化无法自然触发、也无法关联本轮选中商品”的问题。
- 本阶段只使用本轮显式 SKU 输入或继续明确 `missing_user_input` / `deferred` 状态。
- 本阶段不调用淘宝、抖音、千川、CRM、ERP、WMS、DMP、会员系统或实时数仓数据，也不把历史商品当作默认商品。

## Post 8-93：智能定价 product_ref 契约

状态：done

验收项：

- [x] 后端设计文档已写入 `temp/DEVELOPMENT_DOCS/backend/`。
- [x] Agent：`ops_smart_pricing` schema 支持 `product_ref`。
- [x] Agent：成功智能定价输出回显 `商品引用`。
- [x] Worker autosuggest：智能定价 / 动态定价 / 调价建议 / 价格优化问法触发 `ops_smart_pricing` 时使用结构化 input。
- [x] Worker autosuggest：本轮显式选品时智能定价 input 携带 `product_ref`。
- [x] Worker autosuggest：本轮显式成本、当前售价、月销量、竞品价、库存天数和价格弹性会进入智能定价 input。
- [x] Worker autosuggest：未选商品时不写空 `product_ref`。
- [x] API：`API_DOCS/sse.md` 已同步智能定价 product_ref 契约。
- [x] 前端：本轮不新增假入口。

当前说明：

- 本阶段解决的是“智能定价无法自然触发、也无法关联本轮选中商品”的问题。
- 本阶段只使用本轮显式价格输入或继续明确 validation / calculated_from_defaults 状态。
- 本阶段不调用淘宝、抖音、千川、CRM、ERP、WMS、DMP、会员系统或实时数仓数据，也不把历史商品当作默认商品。

## Post 8-92：综合业务诊断 product_ref 契约

状态：done

验收项：

- [x] 后端设计文档已写入 `temp/DEVELOPMENT_DOCS/backend/`。
- [x] Agent：`data_comprehensive_diagnosis` schema 支持 `product_ref`。
- [x] Agent：成功综合诊断输出回显 `商品引用`。
- [x] Worker autosuggest：综合诊断 / 全面诊断 / 业务诊断 / 经营体检问法触发 `data_comprehensive_diagnosis` 时使用结构化 input。
- [x] Worker autosuggest：本轮显式选品时综合诊断 input 携带 `product_ref`。
- [x] Worker autosuggest：本轮显式经营指标、渠道指标、退款项和数字序列会进入综合诊断 input。
- [x] Worker autosuggest：未选商品时不写空 `product_ref`。
- [x] API：`API_DOCS/sse.md` 已同步综合诊断 product_ref 契约。
- [x] 前端：本轮不新增假入口。

当前说明：

- 本阶段解决的是“综合诊断无法自然触发、也无法关联本轮选中商品”的问题。
- 本阶段只使用本轮显式业务输入或继续明确 missing/deferred 状态。
- 本阶段不调用淘宝、抖音、千川、CRM、ERP、WMS、DMP、会员系统或实时数仓数据，也不把历史商品当作默认商品。

## Post 8-91：数据看板 product_ref 契约

状态：done

验收项：

- [x] 后端设计文档已写入 `temp/DEVELOPMENT_DOCS/backend/`。
- [x] Agent：`data_dashboard` schema 支持 `product_ref`。
- [x] Agent：成功经营看板输出回显 `商品引用`。
- [x] Worker autosuggest：dashboard / 看板 / 经营概览 / 经营指标 / 店铺大盘问法触发 `data_dashboard` 时使用结构化 input。
- [x] Worker autosuggest：本轮显式选品时 dashboard input 携带 `product_ref`。
- [x] Worker autosuggest：本轮显式经营指标会进入 dashboard input。
- [x] Worker autosuggest：未选商品时不写空 `product_ref`。
- [x] API：`API_DOCS/sse.md` 已同步数据看板 product_ref 契约。
- [x] 前端：本轮不新增假入口。

当前说明：

- 本阶段解决的是“商品级经营看板无法关联本轮选中商品”的问题。
- 本阶段只使用本轮显式经营指标或继续明确 sample / store_metrics_required 状态。
- 本阶段不调用淘宝、抖音、千川、CRM、ERP、WMS 或实时店铺数据，也不把历史商品当作默认商品。

## Post 8-90：A/B 测试 product_ref 契约

状态：done

验收项：

- [x] 后端设计文档已写入 `temp/DEVELOPMENT_DOCS/backend/`。
- [x] Agent：`data_ab_test_analyzer` schema 支持 `product_ref`。
- [x] Agent：成功 A/B 测试输出回显 `商品引用`。
- [x] Worker autosuggest：A/B / AB测试 / 对照组 / 实验组 / 推全问法触发 `data_ab_test_analyzer`。
- [x] Worker autosuggest：本轮显式选品时 A/B 测试 input 携带 `product_ref`。
- [x] Worker autosuggest：未选商品时不写空 `product_ref`。
- [x] API：`API_DOCS/sse.md` 已同步 A/B 测试 product_ref 契约。
- [x] 前端：本轮不新增假入口。

当前说明：

- 本阶段解决的是“商品级 A/B 测试报告无法关联本轮选中商品”的问题。
- 本阶段只使用本轮显式对照组/实验组访客和转化输入。
- 本阶段不调用实验平台、淘宝、抖音、千川、CRM、DMP 或平台实时实验数据，也不把历史商品当作默认商品。

## Post 8-89：客户分层 product_ref 契约

状态：done

验收项：

- [x] 后端设计文档已写入 `temp/DEVELOPMENT_DOCS/backend/`。
- [x] Agent：`data_customer_segmentation` schema 支持 `product_ref`。
- [x] Agent：成功客户分层输出回显 `商品引用`。
- [x] Worker autosuggest：客户 / 人群 / RFM / 分层问法触发 `data_customer_segmentation` 时使用结构化 input。
- [x] Worker autosuggest：本轮显式选品时客户分层 input 携带 `product_ref`。
- [x] Worker autosuggest：未选商品时不写空 `product_ref`。
- [x] API：`API_DOCS/sse.md` 已同步客户分层 product_ref 契约。
- [x] 前端：本轮不新增假入口。

当前说明：

- 本阶段解决的是“商品级客户 / 人群 / RFM 分层结果无法关联本轮选中商品”的问题。
- 本阶段只使用本轮显式 RFM 聚合输入或 deterministic 默认估算，并用 `data_readiness` 明示。
- 本阶段不调用会员系统、订单系统、CRM、DMP 或平台实时人群数据，也不把历史商品当作默认商品。

## Post 8-88：异常诊断 product_ref 契约

状态：done

验收项：

- [x] 后端设计文档已写入 `temp/DEVELOPMENT_DOCS/backend/`。
- [x] Agent：`data_anomaly_diagnosis` schema 支持 `product_ref`。
- [x] Agent：成功异常诊断输出回显 `商品引用`。
- [x] Worker autosuggest：异常 / 波动 / 下滑 / 暴涨 / 暴跌问法触发 `data_anomaly_diagnosis` 时使用结构化 input。
- [x] Worker autosuggest：本轮显式选品时异常诊断 input 携带 `product_ref`。
- [x] Worker autosuggest：未选商品时不写空 `product_ref`。
- [x] API：`API_DOCS/sse.md` 已同步异常诊断 product_ref 契约。
- [x] 前端：本轮不新增假入口。

当前说明：

- 本阶段解决的是“商品级异常诊断结果无法关联本轮选中商品”的问题。
- 本阶段只使用本轮显式历史序列；少于 3 个历史数据点时继续 deferred。
- 本阶段不调用平台历史销售、实时告警或活动外部数据，也不把历史商品当作默认商品。

## Post 8-87：通用趋势预测 product_ref 契约

状态：done

验收项：

- [x] 后端设计文档已写入 `temp/DEVELOPMENT_DOCS/backend/`。
- [x] Agent：`data_trend_forecast` schema 支持 `product_ref`。
- [x] Agent：成功趋势预测输出回显 `商品引用`。
- [x] Worker autosuggest：趋势 / 预测问法触发 `data_trend_forecast` 时使用结构化 input。
- [x] Worker autosuggest：本轮显式选品时趋势预测 input 携带 `product_ref`。
- [x] Worker autosuggest：未选商品时不写空 `product_ref`。
- [x] API：`API_DOCS/sse.md` 已同步趋势预测 product_ref 契约。
- [x] 前端：本轮不新增假入口。

当前说明：

- 本阶段解决的是“商品级普通趋势预测结果无法关联本轮选中商品”的问题。
- 本阶段只使用本轮显式历史序列；少于 2 个历史数据点时继续 deferred。
- 本阶段不调用平台历史销售、行业周期或活动外部数据，也不把历史商品当作默认商品。

## Post 8-86：多周期趋势 product_ref 契约

状态：done

验收项：

- [x] 后端设计文档已写入 `temp/DEVELOPMENT_DOCS/backend/`。
- [x] Agent：`data_multi_period_trend` schema 支持 `product_ref`。
- [x] Agent：成功多周期趋势输出回显 `商品引用`。
- [x] Worker autosuggest：`多周期趋势` / `多周期环比` / `多周期对比` / `环比趋势` / `同比环比` / `移动平均` / `趋势动能` 问法触发 `data_multi_period_trend`。
- [x] Worker autosuggest：本轮显式选品时多周期趋势 input 携带 `product_ref`。
- [x] Worker autosuggest：未选商品时不写空 `product_ref`。
- [x] API：`API_DOCS/sse.md` 已同步多周期趋势 product_ref 契约。
- [x] 前端：本轮不新增假入口。

当前说明：

- 本阶段解决的是“商品级多周期趋势 / 环比 / 移动平均结果无法关联本轮选中商品”的问题。
- 本阶段只使用本轮显式历史序列；少于 4 个历史数据点时继续 deferred。
- 本阶段不调用平台历史销售、行业周期或活动外部数据，也不把历史商品当作默认商品。

## Post 8-85：季节性分解 product_ref 契约

状态：done

验收项：

- [x] 后端设计文档已写入 `temp/DEVELOPMENT_DOCS/backend/`。
- [x] Agent：`data_seasonal_decompose` schema 支持 `product_ref`。
- [x] Agent：成功季节性分解输出回显 `商品引用`。
- [x] Worker autosuggest：`季节性` / `周期性` / `周期分解` / `季节分解` / `周内波动` / `周期波动` 问法触发 `data_seasonal_decompose`。
- [x] Worker autosuggest：本轮显式选品时季节性分解 input 携带 `product_ref`。
- [x] Worker autosuggest：未选商品时不写空 `product_ref`。
- [x] API：`API_DOCS/sse.md` 已同步季节性分解 product_ref 契约。
- [x] 前端：本轮不新增假入口。

当前说明：

- 本阶段解决的是“商品级季节性 / 周期性波动结果无法关联本轮选中商品”的问题。
- 本阶段只使用本轮显式历史序列；少于 `period * 2` 个数据点时继续 deferred。
- 本阶段不调用平台历史销售、活动日历或行业外部数据，也不把历史商品当作默认商品。

## Post 8-84：队列留存 product_ref 契约

状态：done

验收项：

- [x] 后端设计文档已写入 `temp/DEVELOPMENT_DOCS/backend/`。
- [x] Agent：`data_cohort_analysis` schema 支持 `product_ref`。
- [x] Agent：成功队列留存输出回显 `商品引用`。
- [x] Worker autosuggest：`队列留存` / `同期群` / `留存分析` / `复购留存` / `新客留存` / `流失节点` 问法触发 `data_cohort_analysis`。
- [x] Worker autosuggest：本轮显式选品时队列留存 input 携带 `product_ref`。
- [x] Worker autosuggest：未选商品时不写空 `product_ref`。
- [x] API：`API_DOCS/sse.md` 已同步队列留存 product_ref 契约。
- [x] 前端：本轮不新增假入口。

当前说明：

- 本阶段解决的是“商品级队列留存 / 复购留存结果无法关联本轮选中商品”的问题。
- 本阶段只使用本轮显式新客基线或 deterministic 默认估算，并用 `data_readiness` 明示。
- 本阶段不调用会员系统、订单系统、CRM 或平台实时数据，也不把历史商品当作默认商品。

## Post 8-83：需求预测 product_ref 契约

状态：done

验收项：

- [x] 后端设计文档已写入 `temp/DEVELOPMENT_DOCS/backend/`。
- [x] Agent：`data_demand_forecast` schema 支持 `product_ref`。
- [x] Agent：成功需求预测输出回显 `商品引用`。
- [x] Worker autosuggest：`需求预测` / `销量预测` / `订单预测` / `备货预测` / `补货预测` / `库存预测` 问法触发 `data_demand_forecast`。
- [x] Worker autosuggest：本轮显式选品时需求预测 input 携带 `product_ref`。
- [x] Worker autosuggest：未选商品时不写空 `product_ref`。
- [x] API：`API_DOCS/sse.md` 已同步需求预测 product_ref 契约。
- [x] 前端：本轮不新增假入口。

当前说明：

- 本阶段解决的是“商品级需求预测 / 备货预测结果无法关联本轮选中商品”的问题。
- 本阶段只使用本轮显式历史序列；少于 3 个历史数据点时继续 deferred。
- 本阶段不调用外部平台、WMS 或 ERP，也不执行真实补货。

## Post 8-82：价格弹性 product_ref 契约

状态：done

验收项：

- [x] 后端设计文档已写入 `temp/DEVELOPMENT_DOCS/backend/`。
- [x] Agent：`data_price_elasticity` schema 支持 `product_ref`。
- [x] Agent：成功价格弹性输出回显 `商品引用`。
- [x] Worker autosuggest：`价格弹性` / `价格敏感` / `需求弹性` / `调价测试` / `涨价影响` / `降价影响` 问法触发 `data_price_elasticity`。
- [x] Worker autosuggest：本轮显式选品时价格弹性 input 携带 `product_ref`。
- [x] Worker autosuggest：未选商品时不写空 `product_ref`。
- [x] API：`API_DOCS/sse.md` 已同步价格弹性 product_ref 契约。
- [x] 前端：本轮不新增假入口。

当前说明：

- 本阶段解决的是“商品级价格弹性 / 调价影响结果无法关联本轮选中商品”的问题。
- 本阶段只使用本轮显式价格-销量点、成本和当前售价；缺少价格点时继续明示默认估算。
- 本阶段不调用外部平台价格系统，也不执行真实调价。

## Post 8-81：LTV 客户价值 product_ref 契约

状态：done

验收项：

- [x] 后端设计文档已写入 `temp/DEVELOPMENT_DOCS/backend/`。
- [x] Agent：`data_ltv_calculator` schema 支持 `product_ref`。
- [x] Agent：成功 LTV 输出回显 `商品引用`。
- [x] Worker autosuggest：`LTV` / `客户价值` / `生命周期价值` / `获客成本回收` 问法触发 `data_ltv_calculator`。
- [x] Worker autosuggest：本轮显式选品时 LTV input 携带 `product_ref`。
- [x] Worker autosuggest：未选商品时不写空 `product_ref`。
- [x] API：`API_DOCS/sse.md` 已同步 LTV product_ref 契约。
- [x] 前端：本轮不新增假入口。

当前说明：

- 本阶段解决的是“商品级 LTV / 客户价值结果无法关联本轮选中商品”的问题。
- 本阶段只使用本轮显式 CAC、客单价、复购频次、毛利率、留存率等输入；缺少参数时继续明示默认估算。
- 本阶段不调用外部平台或会员系统 API，也不绑定历史商品或默认商品。

## Post 8-80：渠道归因 product_ref 契约

状态：done

验收项：

- [x] 后端设计文档已写入 `temp/DEVELOPMENT_DOCS/backend/`。
- [x] Agent：`data_attribution_analysis` schema 支持 `product_ref`。
- [x] Agent：成功渠道归因输出回显 `商品引用`。
- [x] Worker autosuggest：`渠道归因` / `多触点归因` / `归因模型` 问法触发 `data_attribution_analysis`。
- [x] Worker autosuggest：本轮显式选品时渠道归因 input 携带 `product_ref`。
- [x] Worker autosuggest：未选商品时不写空 `product_ref`。
- [x] API：`API_DOCS/sse.md` 已同步渠道归因 product_ref 契约。
- [x] 前端：本轮不新增假入口。

当前说明：

- 本阶段解决的是“商品级渠道归因结果无法关联本轮选中商品”的问题。
- 本阶段把渠道 ROI 和多模型渠道归因的自然语言触达边界分开。
- 本阶段只使用本轮显式渠道数据，不调用外部平台 API。
- 缺少多渠道输入时仍返回 deferred，不创建假归因报告。

## Post 8-79：退款拆解 product_ref 契约

状态：done

验收项：

- [x] 后端设计文档已写入 `temp/DEVELOPMENT_DOCS/backend/`。
- [x] Agent：`data_refund_decomposition` schema 支持 `product_ref`。
- [x] Agent：成功退款拆解输出回显 `商品引用`。
- [x] Worker autosuggest：本轮显式选品时退款拆解 input 携带 `product_ref`。
- [x] Worker autosuggest：未选商品时不写空 `product_ref`。
- [x] API：`API_DOCS/sse.md` 已同步退款拆解 product_ref 契约。
- [x] 前端：本轮不新增假入口。

当前说明：

- 本阶段解决的是“商品级退款拆解结果无法关联本轮选中商品”的问题。
- 本阶段只使用本轮显式退款项，不调用外部平台 API。
- 缺少退款项数据时仍返回 deferred，不创建假报告，也不绑定历史商品或默认商品。

## Post 8-78：渠道 ROI product_ref 契约

状态：done

验收项：

- [x] 后端设计文档已写入 `temp/DEVELOPMENT_DOCS/backend/`。
- [x] Agent：`data_channel_roi` schema 支持 `product_ref`。
- [x] Agent：成功渠道 ROI 输出回显 `商品引用`。
- [x] Worker autosuggest：本轮显式选品时渠道 ROI input 携带 `product_ref`。
- [x] Worker autosuggest：未选商品时不写空 `product_ref`。
- [x] API：`API_DOCS/sse.md` 已同步渠道 ROI product_ref 契约。
- [x] 前端：本轮不新增假入口。

当前说明：

- 本阶段解决的是“商品级渠道 ROI 结果无法关联本轮选中商品”的问题。
- 本阶段只使用本轮显式渠道 GMV、广告费、订单、UV 等指标，不调用外部平台 API。
- 缺少渠道数据时仍返回 deferred，不创建假报告，也不绑定历史商品或默认商品。

## Post 8-77：漏斗分析 product_ref 契约

状态：done

验收项：

- [x] 后端设计文档已写入 `temp/DEVELOPMENT_DOCS/backend/`。
- [x] Agent：`data_funnel_analysis` schema 支持 `product_ref`。
- [x] Agent：成功漏斗分析输出回显 `商品引用`。
- [x] Worker autosuggest：本轮显式选品时漏斗分析 input 携带 `product_ref`。
- [x] Worker autosuggest：未选商品时不写空 `product_ref`。
- [x] API：`API_DOCS/sse.md` 已同步漏斗分析 product_ref 契约。
- [x] 前端：本轮不新增假入口。

当前说明：

- 本阶段解决的是“商品级漏斗分析结果无法关联本轮选中商品”的问题。
- 本阶段只使用本轮显式漏斗指标，不调用外部平台 API。
- 缺少曝光、点击等漏斗数据时仍返回 deferred，不创建假报告。

## Post 8-76：竞品分析 product_ref 契约

状态：done

验收项：

- [x] 后端设计文档已写入 `temp/DEVELOPMENT_DOCS/backend/`。
- [x] Agent：`data_competitor_analysis` schema 支持 `product_ref`。
- [x] Agent：竞品分析输出回显 `商品引用`。
- [x] Worker autosuggest：本轮显式选品时竞品分析 input 携带 `product_ref` 和选中商品名。
- [x] Worker autosuggest：未选商品时不写空 `product_ref`。
- [x] API：`API_DOCS/sse.md` 已同步竞品分析 product_ref 契约。
- [x] 前端：本轮不新增假入口。

当前说明：

- 本阶段解决的是“本地竞品分析结果无法关联本轮选中商品”的问题。
- 本阶段不调用外部搜索或官方平台 API。
- 真实竞品公开资料仍由 `search_competitor` 或后续 adapter 负责。

## Post 8-75：商品指标查询 product_ref 契约

状态：done

验收项：

- [x] 后端设计文档已写入 `temp/DEVELOPMENT_DOCS/backend/`。
- [x] Agent：`query_product_metrics` schema 支持 `product_ref`。
- [x] Agent：成功输出和 deferred 输出都会在 `query.product_ref` 回显显式商品引用或 null。
- [x] Worker autosuggest：本轮显式选品时商品指标查询 input 携带 `product_ref` 和商品名。
- [x] Worker autosuggest：未选商品时不写空 `product_ref`，不从历史摘要或默认商品推断。
- [x] API：`API_DOCS/sse.md` 已同步商品指标 product_ref 契约。
- [x] 前端：本轮不新增假入口。

当前说明：

- 本阶段解决的是“商品指标 skill 已有显式指标能力，但没有接入新产品商品引用契约”的问题。
- 本阶段只消费用户本轮显式商品引用和显式指标，不接淘宝、抖音、千川官方 API。
- 没有指标时仍返回 deferred，不创建假商品指标报告。

## Post 8-69：平台接口健康检查自然触达

状态：done

验收项：

- [x] 后端设计文档已写入 `temp/DEVELOPMENT_DOCS/backend/`。
- [x] Agent：工程 worker 可从平台 API 连通性 / 接口健康问法触达 `engineering_system_check`。
- [x] Agent：可解析目标平台，并在真实测试连接意图下传入 `test_live=true`。
- [x] Runtime：真实测试连接返回 `pending_approval`。
- [x] Runtime：metadata 标记为 `execution_mode=adapter`、`risk_level=high`、`approval_required=true`。
- [x] Role skill catalog：工程岗位允许使用 `engineering_system_check`。
- [x] API：`API_DOCS/sse.md` 已同步自然触达和审批边界。
- [x] 前端：本轮不新增假入口。

当前说明：

- 本阶段解决的是“旧产品平台健康检查能力在新产品里不容易自然触达”的问题。
- 本阶段不读取平台连接数据库，不调用淘宝、抖音、千川等官方 API。
- 真实连通性测试仍必须走 adapter 和审批闭环。

## Post 8-68：渠道 ROI / 退款拆解 native 契约修正

状态：done

验收项：

- [x] 后端设计文档已写入 `temp/DEVELOPMENT_DOCS/backend/`。
- [x] Agent：`data_channel_roi` 在显式 `channels[]` 输入下返回 success。
- [x] Agent：`data_refund_decomposition` 在显式 `items[]` 输入下返回 success。
- [x] Agent runtime：两个 skill 在成功执行时标记为 `execution_mode=native`、`risk_level=low`。
- [x] Worker autosuggest：渠道 ROI 问法可解析本轮显式渠道指标。
- [x] Worker autosuggest：退款帕累托 / 退款拆解问法可解析本轮显式退款项。
- [x] API：`API_DOCS/sse.md` 已同步 native / deferred 边界。
- [x] 前端：本轮不新增假入口。

当前说明：

- 本阶段解决的是“渠道 ROI / 退款拆解已有 Rust 执行，但 runtime 契约仍像 deferred/adapter 能力”的问题。
- 本阶段只使用本轮用户显式输入的数据，不接官方平台 API，也不读取默认商品或历史商品。
- 缺少渠道或退款项数据时仍返回 deferred，不创建假报告。

## Post 8-67：店铺指标报告 artifact 沉淀

状态：done

验收项：

- [x] 后端设计文档已写入 `temp/DEVELOPMENT_DOCS/backend/`。
- [x] Service：`query_store_metrics` 成功输出可沉淀为 `data_store_metrics_report`。
- [x] Service：店铺指标 deferred 输出不会创建 artifact。
- [x] API：`API_DOCS/app.md` 已同步新 artifact 类型。
- [x] API：`API_DOCS/sse.md` 已同步 `workspace_artifact_created` 边界。
- [x] 前端：本轮不新增假入口。

当前说明：

- 本阶段解决的是“店铺指标查询能回答但不能沉淀为工作区交付物”的问题。
- 本阶段不接官方平台 API，也不从数据库读取默认店铺实时指标。
- 只有本轮显式指标触发的成功输出才会生成报告。

## Post 8-66：商品 / 活动指标报告 artifact 沉淀

状态：done

验收项：

- [x] 后端设计文档已写入 `temp/DEVELOPMENT_DOCS/backend/`。
- [x] Service：`query_product_metrics` 成功输出可沉淀为 `data_product_metrics_report`。
- [x] Service：`query_campaign_metrics` 成功输出可沉淀为 `data_campaign_metrics_report`。
- [x] Service：商品指标 deferred 输出不会创建 artifact。
- [x] Service：活动指标 deferred 输出不会创建 artifact。
- [x] API：`API_DOCS/app.md` 已同步新 artifact 类型。
- [x] API：`API_DOCS/sse.md` 已同步 `workspace_artifact_created` 边界。
- [x] 前端：本轮不新增假入口。

当前说明：

- 本阶段解决的是“商品 / 活动指标分析能回答但不能沉淀为工作区交付物”的问题。
- 本阶段不接官方平台 API，也不从数据库读取默认商品或默认活动。
- 只有本轮显式指标触发的成功输出才会生成报告。

## Post 8-65：商品 / 活动指标查询 native skill

状态：done

验收项：

- [x] 后端设计文档已写入 `temp/DEVELOPMENT_DOCS/backend/`。
- [x] Agent：新增 `query_product_metrics`。
- [x] Agent：新增 `query_campaign_metrics`。
- [x] Agent：显式指标输入时返回 `status=success`、`adapter_status=manual_input`。
- [x] Agent：缺指标输入时返回 `status=deferred`、`product_metrics_required` / `campaign_metrics_required`。
- [x] Agent：商品指标输出包含 totals、derived_metrics、week_over_week、risks、next_actions。
- [x] Agent：活动指标输出包含预算使用率、CTR、转化率、ROAS、ROI、退款率、风险和动作建议。
- [x] Agent：自然语言可触达商品指标和活动指标查询。
- [x] API：`API_DOCS/sse.md` 已同步新 skill 契约。
- [x] 前端：本轮不新增假入口。

当前说明：

- 本阶段解决的是“旧产品指标仓能力只覆盖店铺级，不覆盖商品/活动级”的问题。
- 本阶段只消费本轮明确提供的指标，不读取数据库，不调用官方平台 API。
- 商品指标不会默认读取任何预设商品或历史商品，保持显式商品通路原则。

## Post 8-64：搜索类 skill 自然触达补强

状态：done

验收项：

- [x] 后端设计文档已写入 `temp/DEVELOPMENT_DOCS/backend/`。
- [x] Agent：用户明确搜索/联网/最新/公开来源意图时，autosuggest 可生成对应 `search_*` skill。
- [x] Agent：趋势、竞品、市场行情、商品评价、平台政策、行业基准 6 类搜索均有自然触达覆盖。
- [x] Agent：搜索输入包含 query/platform/focus/topic/policy_type/metric/days 等结构化字段。
- [x] Agent：保留 `_needs_fresh_data` / `bypass_cache` 语义。
- [x] Agent：普通 native 分析意图不会被搜索 skill 抢走。
- [x] 角色目录：被 autosuggest 的搜索 skill 在对应角色白名单内。
- [x] API：`API_DOCS/sse.md` 已同步自然触达说明。
- [x] 前端：本轮不新增假入口。

当前说明：

- 本阶段解决的是“已有搜索 adapter 契约但用户自然语言不容易触发”的问题。
- 本阶段不接淘宝、抖音、千川官方接口；provider 未配置时仍返回真实 deferred/degraded。

## Post 8-63：店铺指标查询 skill 自然触达补强

状态：done

验收项：

- [x] 后端设计文档已写入 `temp/DEVELOPMENT_DOCS/backend/`。
- [x] Agent：用户明确查询/加载店铺指标时，autosuggest 可生成 `query_store_metrics`。
- [x] Agent：文本中的 GMV、订单、UV、点击、广告费、退款、毛利可进入 skill input。
- [x] Agent：支持 `万/千/亿` 单位解析。
- [x] Agent：未明确平台时为 `platform=all`，不默认淘宝。
- [x] Agent：店铺大盘/看板等更具体意图仍走 `data_dashboard`。
- [x] API：`API_DOCS/sse.md` 已同步自然触达说明。
- [x] 前端：本轮不新增假入口。

当前说明：

- 本阶段解决的是“能力已经实现但自然语言不容易触发”的问题。
- 本阶段只做后端路由/触达补强，不读取真实平台和数据库。

## Post 8-62：店铺指标查询 skill native 显式输入补强

状态：done

验收项：

- [x] 后端设计文档已写入 `temp/DEVELOPMENT_DOCS/backend/`。
- [x] Agent：`query_store_metrics` 不再固定返回 deferred。
- [x] Agent：有显式指标时返回 `status=success`、`adapter_status=manual_input`、`data_readiness=explicit_metrics`。
- [x] Agent：输出 `totals`、`derived_metrics`、`week_over_week`、`anomalies` 和 `platforms`。
- [x] Agent：无显式指标时返回 `status=deferred`、`adapter_status=store_metrics_required`。
- [x] Agent：多平台输入支持按 `platform` 过滤。
- [x] API：`API_DOCS/sse.md` 已同步该 skill 的 `skill_result.output` 契约。
- [x] 前端：本轮不新增假入口；后续过程面板应按 `adapter_status/data_readiness` 展示真实状态。
- [x] 验证：已补显式指标、缺指标和多平台过滤单测。

当前说明：

- 本阶段解决的是“旧产品店铺指标查询能力在新产品里只有占位”的问题。
- 本阶段只消费本轮明确提供的指标，不读取数据库，不调用官方平台 API。
- 后续真实 store_metrics 读取必须通过已鉴权 service/adapter 注入 Agent。

## Post 8-61：技能链规划 skill native 补强

状态：done

验收项：

- [x] 后端设计文档已写入 `temp/DEVELOPMENT_DOCS/backend/`。
- [x] Agent：`coordination_skill_chain_planner` 返回 `status=success` 和结构化 `推荐执行链`。
- [x] Agent：每个步骤包含阶段、负责角色、执行模式、风险等级、前置依赖、可用状态和说明。
- [x] Agent：旧产品 `data_query_store_metrics` 已规范为新产品真实 `query_store_metrics`。
- [x] Agent：输出 `前置依赖说明`、`前置依赖缺口`、`不可用技能`、`审批提示` 和 `adapter提示`。
- [x] Agent：`skills_available` 只用于标记可用性，不伪造执行。
- [x] API：`API_DOCS/sse.md` 已同步该 skill 的 `skill_result.output` 契约。
- [x] 前端：本轮不新增假入口；后续可用真实 `skill_result.output` 接过程面板。
- [x] 验证：`cargo test -p agent coordination_skill_chain -- --nocapture` 通过。

当前说明：

- 本阶段解决的是“旧产品技能链规划只迁了名字，过程信息不足”的问题。
- 本阶段只规划，不自动批量执行，不调用外部 Agent，不执行平台写操作。
- 后续 Kimi Agent 风格右侧过程面板应直接消费这些结构化字段。

## Post 8-60：外部搜索证据契约补强

状态：done

验收项：

- [x] 后端：Agent `search_*` skill 输出新增 `query.bypass_cache`。
- [x] 后端：Agent `search_*` skill 支持 `bypass_cache`、`bypassCache`、`_needs_fresh_data`。
- [x] 后端：provider 请求会转发 `bypass_cache`。
- [x] 后端：live/deferred/degraded 搜索输出均包含 `evidence`。
- [x] 后端：live 搜索来源会去重并重新生成 rank。
- [x] Service：`ExternalSearchResponse` 新增 `maxResults`、`bypassCache` 和 `evidence`。
- [x] Service：`search_evidence` artifact metadata 保留 `evidence`。
- [x] API：`API_DOCS/app.md` 已同步外部搜索响应结构。
- [x] API：`API_DOCS/sse.md` 已同步 Agent 搜索输出和搜索证据 artifact 说明。
- [x] 文档：后端设计文档、迁移矩阵、开发记录和本轮归档已同步。
- [x] 验证：`cargo fmt --all`
- [x] 验证：`cargo test -p agent search -- --nocapture`
- [x] 验证：`cargo test -p service external_search -- --nocapture`

当前说明：

- 本轮补强的是通用外部搜索证据契约，不是淘宝、抖音、千川官方 API 对接。
- 没有可信来源时仍保持 degraded/deferred，不创建假 `search_evidence`。
- 前端后续应消费 `evidence` 构建来源面板，不要在前端伪造来源。

## Post 8-59：综合业务诊断 skill native 编排迁移

状态：done

验收项：

- [x] 后端：`data_comprehensive_diagnosis` 不再是 deferred 宏占位，已改为 Agent native orchestrator skill。
- [x] 后端：可编排经营看板、漏斗、异常、客户分层、退款、需求预测、渠道归因和 LTV 分项。
- [x] 后端：输出整体健康评分、综合诊断报告、分项诊断、核心发现、风险预警、增长机会、7 天行动计划和缺失数据。
- [x] 后端：整体无有效用户数据时返回 `status=deferred`。
- [x] 后端：不读取、不伪造淘宝/抖音/千川/数仓实时数据。
- [x] Service：成功输出可沉淀为 `data_comprehensive_diagnosis_report` workspace artifact。
- [x] API：`API_DOCS/app.md` 已同步 artifact 类型。
- [x] API：`API_DOCS/sse.md` 已同步 `workspace_artifact_created` 支持名单和数据说明。
- [x] 文档：后端设计文档、迁移矩阵、开发记录和本轮归档已同步。
- [x] 验证：`cargo fmt --all`
- [x] 验证：`cargo fmt --all --check`
- [x] 验证：`cargo test -p agent --lib data_comprehensive_diagnosis -- --nocapture`
- [x] 验证：`cargo test -p service content_artifact_builds_from_data_comprehensive_skill_output -- --nocapture`
- [x] 验证：`cargo check -p agent -p service`
- [x] 验证：`cargo clippy -p agent -p service`

## Post 8-58：渠道归因分析 skill native 迁移

状态：done

验收项：

- [x] 后端：`data_attribution_analysis` 不再是 deferred 宏占位，已改为 Agent native skill。
- [x] 后端：支持 `channels` 输入，每个渠道可包含 GMV、UV、转化率和广告花费。
- [x] 后端：输出末次归因、线性归因、时间衰减归因、位置归因。
- [x] 后端：缺渠道、少于两个有效渠道或总 GMV 为 0 时返回 `status=deferred`。
- [x] 后端：不读取、不伪造淘宝/抖音/千川/数仓实时数据。
- [x] Service：成功输出可沉淀为 `data_attribution_report` workspace artifact。
- [x] API：`API_DOCS/app.md` 已同步 artifact 类型。
- [x] API：`API_DOCS/sse.md` 已同步 `workspace_artifact_created` 支持名单和数据说明。
- [x] 文档：后端设计文档、迁移矩阵、开发记录和本轮归档已同步。
- [x] 验证：`cargo fmt --all`
- [x] 验证：`cargo fmt --all --check`
- [x] 验证：`cargo test -p agent --lib data_attribution_analysis -- --nocapture`
- [x] 验证：`cargo test -p service content_artifact_builds_from_data_advanced_skill_output -- --nocapture`
- [x] 验证：`cargo check -p agent -p service`
- [x] 验证：`cargo clippy -p agent -p service`

## Post 8-57：高级财务 skill 输出契约与 artifact 沉淀

状态：done

验收项：

- [x] 后端：高级财务/经营分析 skill 已补齐统一输出契约。
- [x] 后端：`accounting_break_even_calc` 成功时返回 `status=success`、`adapter_status=manual_input` 和 `summary`。
- [x] 后端：`accounting_compliance_check` 缺少 `monthly_revenue` 时返回 `status=deferred`，不创建假合规报告。
- [x] 后端：`accounting_pl_statement` 缺少 `gmv` 时返回 `status=deferred`，成功时明示默认参数使用情况。
- [x] 后端：`accounting_budget_vs_actual` 缺少 `actual_gmv` 时返回 `status=deferred`。
- [x] 后端：`accounting_gmv_waterfall` 缺少 `gmv` 时返回 `status=deferred`，成功时可沉淀报告。
- [x] 后端：`accounting_scenario_analysis` 缺少 `base_gmv` 时返回 `status=deferred`。
- [x] 后端：`accounting_cash_flow_forecast` 和 `accounting_financial_narrative` 成功时明示 `data_readiness`。
- [x] Service：高级财务成功输出可沉淀为 workspace markdown artifact。
- [x] API：`API_DOCS/app.md` 已同步 artifact 类型。
- [x] API：`API_DOCS/sse.md` 已同步 `workspace_artifact_created` 支持名单和财务说明。
- [x] 文档：后端设计文档、迁移矩阵、开发记录和本轮归档已同步。
- [x] 验证：`cargo fmt --all`
- [x] 验证：`cargo fmt --all --check`
- [x] 验证：`cargo test -p agent accounting_ -- --nocapture`
- [x] 验证：`cargo test -p service content_artifact_builds_from_accounting_skill_output -- --nocapture`
- [x] 验证：`cargo check -p agent -p service`
- [x] 验证：`cargo clippy -p agent -p service`
- [x] 验证：`git diff --check`

## Post 8-56：客服商品查询 skill 读取显式商品上下文

状态：done

验收项：

- [x] 后端：`service_query_product` 不再是 deferred 宏占位，已改为 Agent native skill。
- [x] 后端：有本轮显式商品上下文时返回 `status=success`。
- [x] 后端：输出包含 `adapter_status=context_metadata`、`data_readiness=explicit_product_context`。
- [x] 后端：无商品上下文时返回 `status=deferred`，不伪造商品信息。
- [x] 后端：商品 ID/名称筛选不命中时返回 `status=deferred`，不改用默认商品。
- [x] 后端：兼容旧产品整数商品 ID 与新产品字符串商品 ID。
- [x] 架构：不在 `agent` crate 直连数据库，不新增 HTTP API，不新增 SSE 事件类型。
- [x] API：`API_DOCS/sse.md` 已补充 `service_query_product` 的 `skill_result` 契约。
- [x] 文档：迁移矩阵、开发记录和本轮归档已同步。
- [x] 验证：`cargo fmt --all`
- [x] 验证：`cargo test -p agent service_query_product`
- [x] 验证：`cargo test -p agent executor_runs_service_query_product_with_context`
- [x] 验证：`cargo check -p agent`
- [x] 验证：`cargo clippy -p agent`
- [x] 验证：`git diff --check`

## Post 8-55：外部搜索提升为第 2 优先级后的 RAG/SSE 可用性契约

状态：done

验收项：

- [x] 迁移总计划已把外部搜索 / 实时信息增强提升为第 2 优先级。
- [x] 官方平台直接 API 集成仍明确暂缓，不在本轮接淘宝、抖音、千川等平台。
- [x] `BuildAgentRetrievalContextRequest` 增加 `external_search_available`。
- [x] Agent SSE 从真实 `[external_search]` 配置读取可用性。
- [x] `rag_context_loaded.externalSearchAvailable` 和 `rag.external_search_available` 不再固定为 `false`。
- [x] 本地 RAG 未命中时，降级原因区分外部搜索未配置和已配置两种状态。
- [x] 未选择商品时仍不注入默认商品知识，保持显式商品上下文原则。
- [x] API 文档同步 `API_DOCS/sse.md`。
- [x] `cargo fmt`
- [x] `cargo test -p service agent_context`
- [x] `cargo test -p web-server retrieval_metadata`
- [x] `cargo check -p service -p web-server`
- [x] `cargo clippy -p service -p web-server`
- [x] `git diff --check`

当前说明：

- 本轮是后端契约增强，不做前端假来源面板。
- 外部搜索已进入第二优先级，但仍坚持“真实来源才展示”：只有同一轮 `search_*` skill 返回 live 来源后，才会有搜索证据。
- 官方平台写/读 adapter 仍暂缓，避免影响当前内部问答和 skill 生态迁移主线。

## Post 8-54：问答商品上下文显式选择与工作台遮挡修正

状态：done

验收项：

- [x] SSE 问答链路未选择商品时，不再通过关键词隐式检索并注入商品上下文。
- [x] SSE 问答链路不再把工作区绑定商品当作本轮默认商品。
- [x] 普通工作区 context summary / context pack API 保留原有工作区商品回退能力。
- [x] API 文档明确“未指定商品”和“显式选择商品”是两条不同链路。
- [x] 前端商品选择入口显示“未指定商品”，并可读取 `active/listing/draft` 商品。
- [x] 左上角折叠入口去重。
- [x] 右侧虚拟员工工作区不再以浮层遮挡正文。
- [x] 过程卡片显示在用户提问和助手正文之间。
- [x] 过程卡片运行中展开，结束后默认折叠，用户可手动展开。
- [x] `cargo fmt`
- [x] `cargo check`
- [x] `cargo clippy`
- [x] `cargo test -p service agent_context`
- [x] `cargo test -p service workspace::tests::context`
- [x] `pnpm -C frontend/apps/web lint`
- [x] `pnpm -C frontend/apps/web rules`
- [x] `pnpm -C frontend/apps/web test`
- [x] `pnpm -C frontend/apps/web build`
- [x] `pnpm -C frontend rules`
- [x] `git diff --check`

当前说明：

- 本轮重点修正用户已经指出的问答链路和界面遮挡问题。
- 商品上下文严格改为用户手动选择后才进入回答；未选择商品时按普通电商问题处理。
- 前端仍基于同事给的 `SidebarShell`、现有 agent-chat 页面组件和 app 页面装配实现，没有新增第二套 UI 框架。
- Codex 内置浏览器插件本轮因 Windows 会话凭据错误无法截图接管，已打开 Edge 到本地产品页面供人工试用。

## Post 8-53：问答工作台折叠收纳与演示清爽化

状态：done

验收项：

- [x] 电商多智能体页左侧导航默认收起，但仍使用同事给的 `SidebarShell` 折叠机制。
- [x] 右侧虚拟员工工作区默认收起为浮动入口，可点击展开。
- [x] 输入区策略、商品上下文、模型、岗位和 playbook 入口默认收进“任务设置”，展开后仍可操作。
- [x] 主问答区保留真实 SSE/Agent 过程展示，不删除已接后端能力。
- [x] 不新增后端 API，不伪造未完成平台 adapter 或外部搜索能力。
- [x] `pnpm -C frontend/apps/web lint`
- [x] `pnpm -C frontend/apps/web rules`
- [x] `pnpm -C frontend/apps/web test`
- [x] `pnpm -C frontend/apps/web build`
- [x] `pnpm -C frontend/packages/app-kit rules`
- [x] `pnpm -C frontend/packages/app-kit typecheck`

当前说明：

- 本轮解决的是汇报前端可用性和信息密度问题。
- 核心原则是“默认清爽，按需展开”：左侧导航、右侧过程和高级任务设置都可以展开，但不再默认全部铺开。
- 本轮只做前端体验收纳和共享 shell 的小配置能力，不改变后端能力边界。

## Post 8-52：演示可用前端对齐

状态：done

验收项：

- [x] 问答页默认展示右侧虚拟员工工作区。
- [x] 右侧虚拟员工工作区可折叠并可重新展开。
- [x] 审批页展示后端执行结果，不把 `adapter_not_implemented` 伪装成成功。
- [x] 平台连接页展示 adapter 必填参数、幂等字段和可执行边界。
- [x] 本轮只修改 `apps/web` 页面装配和语言文案，不新增第二套 UI 体系。
- [x] `pnpm -C frontend/apps/web rules`
- [x] `pnpm -C frontend/apps/web lint`
- [x] `pnpm -C frontend/apps/web test`
- [x] `pnpm -C frontend/apps/web build`

当前说明：

- 本轮是为了今天公司汇报做的前端可用性对齐。
- 已完成后端能力在前端更清楚地展示：问答工作台、平台执行 readiness、adapter request/outcome、审批执行结果。
- 后端未完成的真实平台 adapter 仍明确显示为不可执行或未接入，不做假成功。

## 总门禁

每个阶段必须先通过后端门禁，再进入前端适配。

后端门禁：

- [ ] 后端能力边界确认完成。
- [ ] 涉及 API、DB、SSE、权限、错误码或 LLM 调用时，后端设计文档完成。
- [ ] Rust agent/service/web-server 实现完成。
- [ ] `API_DOCS/` 已同步。
- [ ] `cargo fmt`、`cargo check`、`cargo clippy`、相关测试或 smoke 已通过。
- [ ] 真实 SSE/HTTP 请求验证完成。

前端适配门禁：

- [ ] 对应后端 API/SSE 契约已稳定。
- [ ] 前端只消费真实后端数据和事件。
- [ ] 未接入的后端能力不做假流程展示。
- [ ] `pnpm rules` 和相关前端验证已通过。

## Phase 0：迁移台账与能力核对

状态：pending

验收项：

- [ ] 旧 46 skill 全部映射完成。
- [ ] 新 93 staged spec 核对完成。
- [ ] 每个 skill 标记 `native_ready` / `adapter_required` / `approval_required` / `missing`。
- [ ] 角色矩阵核对完成。

## Phase 1：问答策略与路由增强

状态：done

验收项：

- [x] 后端：execution/learning/collab 策略进入 request metadata。
- [x] 后端：Commander 能按策略影响路由。
- [x] 后端：SSE 输出真实 route decision / strategy applied。
- [x] 简单问答走轻流程。
- [x] 复杂任务触发多角色。
- [x] 前端：策略切换只展示后端真实行为差异。

## Phase 2：Skill 执行与 SSE 可视化

状态：done

验收项：

- [x] 后端：`skill_requested` 事件。
- [x] 后端：`skill_result` 事件。
- [x] 后端：失败、deferred、approval_required 状态结构化。
- [x] 后端：至少 6 个代表 skill 可真实执行或明确 deferred。
- [x] 前端：运行详情卡和时间线接真实 skill 事件。

## Phase 3：产品知识与上下文绑定

状态：done

验收项：

- [x] 后端：SSE request 携带 context refs。
- [x] 后端：读取真实产品知识。
- [x] 后端：权限、空数据和超长上下文降级规则明确。
- [x] 回答引用真实产品信息。
- [x] 前端：可选择产品上下文并展示引用来源。

## Phase 4：工作区沉淀与交付物

状态：done

验收项：

- [x] 后端：Agent run 关联 workspace run。
- [x] 后端：Worker report 可沉淀任务。
- [x] 后端：artifact 创建、查询、权限闭环。
- [x] 前端：问答右侧面板可展示后端 `workspace_artifact_created` 交付物事件。
- [x] 前端：完整工作区页面可刷新后查看 artifact 列表和详情。

当前说明：

- 已完成 workspace/run/task/artifact 后端闭环。
- Agent SSE 正常完成时会创建 `final_report` 交付物，并发出 `workspace_artifact_created`。
- 完整工作区页面已接入后端 artifact 查询和详情接口，刷新后可查看交付物列表、预览正文、来源、格式、存储、运行 ID 和校验状态。

## Phase 5：质量、信任、学习闭环

状态：done

验收项：

- [x] 后端：质量评分结构可用。
- [x] 后端：用户反馈可保存。
- [x] 后端：trust/learning 可影响后续回答。
- [x] 前端：质量评分展示真实数据。
- [x] 前端：用户反馈入口已展示并可提交。

当前说明：

- 已完成质量检查、信任分、学习记录三张表和后端闭环。
- Agent SSE 正常完成时会自动质量检查，并发出 `quality_checked`。
- 前端问答页右侧过程区已展示质量分、信任分和学习记录数量。
- 前端问答页右侧工作台已接入真实运行反馈入口，提交后写入后端学习记录。

## Phase 6：审批、告警、平台写操作

状态：done

验收项：

- [x] 后端：平台写操作默认 pending approval。
- [x] 后端：审批记录可查。
- [x] 后端：告警可推送。
- [x] 前端：审批和告警只展示后端真实记录。

当前说明：

- Agent SSE 收到 `skill_result.status=pending_approval` 或 `approval_required=true` 时，会自动创建或复用工具审批记录。
- 同步写入 `tool_approval.create` 审计日志，并创建或复用 `agent_approval_required` 告警。
- 前端问答页右侧时间线和工作区卡片已消费真实 `approval_required` 事件。
- 本阶段只完成审批记录、审计和告警，不执行平台写操作。

## Phase 7：搜索/RAG/外部数据

状态：done

验收项：

- [x] 后端：搜索结果有来源。
- [x] 后端：产品知识可检索。
- [x] 后端：外部不可用可降级。
- [x] 前端：来源和降级原因可见。

当前说明：

- 已完成本地 RAG 检索，来源来自当前用户自己的 active 商品和 active 商品知识。
- Agent SSE 会发出 `rag_context_loaded`，并把检索摘要注入 agent metadata。
- 外部搜索适配器未接入时不伪造结果，前端展示明确降级原因。

## Phase 8：完整商业化体验

状态：done

验收项：

- [x] 新品冷启动场景全通。
- [x] 投放复盘场景全通。
- [x] 客诉危机场景全通。
- [x] 选品定价场景全通。
- [x] 素材优化场景全通。
- [x] 日报复盘场景全通。

当前说明：

- 后端已提供 6 个商业场景 playbook，并走 App 用户鉴权。
- 前端问答输入区已改为读取后端 playbook；点击后自动填入任务并切换推荐策略。
- 本阶段完成的是商业场景入口和问答链路适配，不新增用户自定义 playbook 数据库能力。

## Phase 9：登录态恢复与旧产品内置能力包治理同步

状态：done

验收项：

- [x] 后端：旧产品内置 domain/role/skill manifest 可同步到当前用户能力包治理目录。
- [x] 后端：派生 capability 记录可用于能力矩阵和依赖校验。
- [x] 后端：批量同步使用事务，重复同步不重复创建。
- [x] API：同步接口写入 `API_DOCS/app.md`。
- [x] 前端：能力包治理页提供真实同步入口。
- [x] 前端：登录态恢复后刷新受保护页面不再误跳登录。

当前说明：

- 本阶段同步的是治理目录，不动态加载 Agent runtime catalog。
- 旧产品 role manifest 的旧 skill 函数名保存为 metadata，不作为新治理依赖，避免校验误报。
- 真实 HTTP smoke 已确认同步 52 条记录，`builtin.ops` 和 `role.ops-strategist` 校验通过。

## Phase 10：Agent 代表 Skill 自动补偿触发

状态：done

验收项：

- [x] 后端：模型显式 `skill_requests` 优先，不被自动补偿覆盖。
- [x] 后端：模型没有输出 `skill_requests` 时，可按岗位和任务自动补偿代表 skill。
- [x] 后端：自动补偿仍通过角色白名单和 `SkillExecutorSet`，不绕过权限。
- [x] 后端：默认业务问题不会被过宽的 chat bypass 吃掉，明显岗位任务会进入 task graph。
- [x] 后端：stream 级别真实发出 `skill_requested` / `skill_result`。
- [x] API：`API_DOCS/sse.md` 已说明自动补偿触发语义。
- [x] 前端：不新增假展示，沿用真实 SSE 事件展示 skill 调用。

当前说明：

- 已完成 ops/data/accounting/service/creative/web/design/engineering 的第一层代表 skill 自动补偿。
- 已修复默认漏斗问题只走 chat 的断点；真实云端 Ollama smoke 已确认默认漏斗问题进入 data+ops 任务图并发出 skill 事件。
- 该阶段提升的是“能稳定触发旧 skill 生态入口”，不是把所有旧 skill 都完整接入外部搜索、平台执行、内容生成或导出产物。
- 高风险平台写操作仍不会自动执行；需要审批的 skill 继续进入 `pending_approval`。

## Phase 11：问答内核调度说明与轻问答兼容

状态：done

验收项：

- [x] 后端：`chat` 角色轻问答不强制 `findings/recommendations`。
- [x] 后端：短确认、能力说明、参与角色说明不再因复杂质量规则误判失败。
- [x] 后端：运营、数据、内容虚拟员工分工问题可稳定进入 `ops -> data/creative` 任务图。
- [x] 后端：中文提示词约束减少英文混入。
- [x] 前端：不新增接口，沿用现有 SSE 和任务图展示。

当前说明：

- 本阶段修复的是问答内核和后端任务图调度，不新增数据库表。
- `chat` 角色仍需要有摘要内容，但不再被复杂业务岗位的质量条款误伤。
- 角色分工类问题可由配置任务图稳定编排运营、数据、内容创意三个岗位。

## Phase 12：创意内容类 Skill Native 迁移

状态：done

验收项：

- [x] 后端：短视频脚本、种草文案、IP 定位、内容排期、趋势捕捉、直播脚本、产品软文均可 native success。
- [x] 后端：creative 岗位自动补偿能按用户意图命中不同内容类 skill。
- [x] 后端：结果通过既有 `SkillExecutorSet`、角色白名单和 SSE skill 事件链路输出。
- [x] 前端：不新增假展示，沿用现有 `skill_requested` / `skill_result` / 最终回答。

当前说明：

- 本阶段把旧产品创意岗位的内容生成能力迁到新产品 Rust Agent runtime，当前以本地结构化文本产物为主。
- 结果包括脚本结构、种草正文草稿、IP 人设、内容日历、趋势方向、直播阶段话术、产品软文结构。
- 外部实时趋势、真实商品详情自动加载、图片/视频/DOCX 文件生成仍是后续阶段。

## Post 8-5：显式岗位选择优先级与前端运行配置稳定性

状态：done

验收项：

- [x] 后端：`manual/single + primaryRole/selectedRoles` 中包含非 `chat` 岗位时，短答旁路不会吞掉显式岗位选择。
- [x] 后端：没有显式非 `chat` 岗位选择的短答请求仍可走 `chat_bypass`，保持轻问答体验。
- [x] 后端：`route_decided` 能在回归测试中返回 `task_graph`、主岗位和支持岗位。
- [x] 前端：多岗模式下主岗位和参与岗位分开控制，发送给后端的运行选择更稳定。
- [x] API：`API_DOCS/sse.md` 已说明显式岗位选择优先于 `chat_bypass`。
- [x] 真实 HTTP/Edge smoke：业务漏斗问题按 `data + ops` 进入任务图并触发 native skill，页面真实请求体包含主岗位 `data` 和参与岗位 `data/ops`。

当前说明：

- 本阶段是问答调度契约补强，不新增数据库表，也不新增 HTTP/SSE 字段。
- 前端只适配后端运行契约，不伪造 skill、任务图或岗位执行结果。
- 本阶段不包含外部搜索、附件、导出报告或平台执行 adapter 迁移。

## Post 8-6：设计与 Web 增长 Skill Native 迁移

状态：done

验收项：

- [x] 后端：`design_main_image`、`design_detail_page`、`design_color_scheme`、`design_material_spec`、`design_campaign_poster` 返回 native success。
- [x] 后端：`web_seo_optimize`、`web_title_generator`、`web_page_conversion`、`web_store_design`、`web_keyword_research`、`web_product_description` 返回 native success。
- [x] 后端：design/web 岗位自动补偿能按用户意图命中不同设计和增长 skill。
- [x] 后端：结果通过既有 `SkillExecutorSet`、角色白名单和 SSE skill 事件链路输出。
- [x] 验证：agent skill 执行测试通过。
- [x] 验证：autosuggest 规则测试通过。
- [x] 验证：`cargo test -p agent` 通过。
- [x] 验证：`cargo check -p agent` 通过。
- [x] 验证：`cargo clippy -p agent` 通过。

当前说明：

- 本阶段把旧产品设计/Web 岗位的内容方案类能力迁到新产品 Rust Agent runtime，当前以本地结构化文本产物为主。
- 结果包括设计 brief、平台规格、配色 HEX、SEO 提排路径、关键词矩阵、标题候选、详情文案和页面转化 AB 测试清单。
- 图片/长图文件生成、实时搜索量、CPC、店铺真实指标和素材文件化仍是后续阶段。

## Post 8-7：客服处理 Skill Native 迁移

状态：done

验收项：

- [x] 后端：`service_ticket_handler` 返回 native success，输出工单优先级、SLA、处理策略、客服话术和升级条件。
- [x] 后端：`service_faq_playbook` 返回 native success，输出 FAQ 简短版、详细版和处理指引。
- [x] 后端：`service_escalation_flow` 返回 native success，输出严重度、升级路径、主管话术、赔偿建议和复核模板。
- [x] 后端：`service_dsr_improvement` 不再返回 deferred dependencies，输出 native DSR 改善方案。
- [x] 后端：service 岗位自动补偿能按用户意图命中工单、FAQ、升级、退换货、DSR、NPS等客服 skill。
- [x] 后端：结果通过既有 `SkillExecutorSet`、角色白名单和 SSE skill 事件链路输出。
- [x] 前端：不新增假展示，沿用现有 `skill_requested` / `skill_result` / 最终回答。
- [x] 验证：agent skill 执行测试通过。
- [x] 验证：autosuggest 规则测试通过。
- [x] 验证：`cargo test -p agent` 通过。
- [x] 验证：`cargo check -p agent` 通过。
- [x] 验证：`cargo clippy -p agent` 通过。
- [x] 验证：`cargo check` 通过。
- [x] 验证：`cargo clippy` 通过。

当前说明：

- 本阶段把旧产品客服岗位的工单、FAQ和升级处理能力迁到新产品 Rust Agent runtime，当前以本地结构化 SOP 和话术产物为主。
- 结果包括优先级、SLA、客服话术、升级条件、FAQ答案、处理指引、主管接管话术、赔偿建议和 DSR 改善动作。
- 实时平台政策、店铺真实售后规则、商品画像自动读取和客服 SOP 文档导出仍是后续阶段。

## Post 8-8：外部搜索 Adapter 契约化

状态：done

验收项：

- [x] 后端：`search_trends` 返回统一 `adapter_required` 降级契约，不伪造 sources。
- [x] 后端：`search_competitor` 返回统一 `adapter_required` 降级契约，不伪造 sources。
- [x] 后端：`search_market_info` 返回统一 `adapter_required` 降级契约，不伪造 sources。
- [x] 后端：`search_product_reviews` 返回统一 `adapter_required` 降级契约，不伪造 sources。
- [x] 后端：`search_platform_policy` 返回统一 `adapter_required` 降级契约，不伪造 sources。
- [x] 后端：`search_industry_benchmarks` 返回统一 `adapter_required` 降级契约，不伪造 sources。
- [x] 后端：六个 search skill 均保留 `status=deferred`，runtime metadata 仍包含 search/LLM adapter requirements。
- [x] 后端：输出包含 `query`、`data.items=[]`、`sources=[]`、`provenance.source_count=0`、`confidence=0.0`、`degraded=true`。
- [x] 前端：本阶段不新增假展示，继续沿用真实 `skill_result` 状态。
- [x] 验证：搜索契约测试通过。

当前说明：

- 本阶段没有接入真实互联网搜索，不声明 provider，不返回 `_search_engine`。
- 本阶段解决的是新产品后端搜索能力的稳定输出契约，给后续真实 search provider/API/SSE 展示铺路。
- 外部实时搜索、来源展示、置信度展示和前端搜索来源面板仍是后续阶段。

## Post 8-9：Rust 外部搜索 Adapter API

状态：done

验收项：

- [x] 后端：新增 `GET /api/app/external-search/status`，返回 adapter 启用和配置状态。
- [x] 后端：新增 `GET /api/app/external-search/search`，查询参数包含 `q/topic/days/maxResults/bypassCache`。
- [x] 后端：未配置 adapter 时返回 `status=degraded`，不伪造来源。
- [x] 后端：provider payload 可归一化为 `data.items`、`provenance`、`confidence`。
- [x] 后端：当前 App 用户鉴权与 enabled 校验沿用现有 `app_users`。
- [x] API：`API_DOCS/app.md` 已同步新增接口。
- [x] 配置：`services-example.toml` 新增 `[external_search]` 示例并默认关闭。
- [x] 前端：本阶段不新增假展示。

当前说明：

- 本阶段完成的是 Rust web/service 层的搜索 adapter API 入口，不是 Agent skill 的 live 调用。
- 当前 `services.toml` 未配置 `[external_search]` 时，产品启动不受影响，接口会返回明确 degraded。
- 下一阶段可以把 Agent `search_*` skill 接入该 adapter 契约，或先做前端来源展示。

## Post 8-10：Agent 搜索 Skill 接入外部搜索 Adapter

状态：done

验收项：

- [x] 后端：Agent runtime 可接收 `[external_search]` 配置。
- [x] 后端：worker skill executor 把搜索配置传给 `search_*` skill。
- [x] 后端：配置完整时，搜索 skill 会请求 provider，并把 provider payload 归一化为 `sources`、`data.items`、`provenance`、`confidence`。
- [x] 后端：provider 未配置、请求失败或无来源时返回降级状态，不伪造来源。
- [x] 后端：搜索类 skill 在角色提示目录中标记为 `adapter`，不再只是静态 `deferred`。
- [x] API/SSE：不新增路由，`API_DOCS/sse.md` 已同步搜索类 `skill_result` 语义。
- [x] 前端：本阶段不改前端，不新增假搜索展示。

## Post 8-11：Agent 搜索证据沉淀为 Workspace Artifact

状态：done

验收项：

- [x] Agent：`skill_result` 事件增加可选结构化 `output` 字段。
- [x] 后端 service：新增 `record_agent_search_evidence`，复用现有 `workspace_artifacts` 表写入 `artifactType=search_evidence`。
- [x] 后端 service：只接受 `search_* + status=success + executionMode=adapter + output.status=live + sources 非空` 的真实搜索结果。
- [x] Web-server：SSE 收到 live 搜索结果后自动写入 `search_evidence`，并发送 `workspace_artifact_created`。
- [x] Web-server：搜索证据写入失败时返回 `workspace_run_persistence_error`，不截断原始回答。
- [x] API/SSE：`API_DOCS/sse.md` 和 `API_DOCS/app.md` 已同步 `search_evidence` artifact 契约。
- [x] 前端：本阶段不改前端，不新增假来源展示。
- [x] 验证：`cargo test -p service search_evidence_artifact` 通过。
- [x] 验证：`cargo test -p web-server skill_result_output_is_camelized_for_sse_consumers` 通过。
- [x] 验证：`cargo test -p agent` 通过。
- [x] 验证：`cargo check -p agent/service/web-server` 通过。
- [x] 验证：`cargo check`、`cargo clippy`、`git diff --check` 通过。

当前说明：

- 本阶段解决的是“真实搜索结果可沉淀、可追溯、可被工作区复用”。
- 如果 `[external_search]` 未配置或 provider 降级，不会创建搜索证据交付物。
- 前端后续可基于 `workspace_artifact_created` 或 artifact 查询接口，把 `search_evidence` 展示为来源面板。

## Post 8-12：附件与素材上下文摘要 API

状态：done

验收项：

- [x] 后端：新增 `WorkspaceApi::context_summary`。
- [x] 后端：上下文摘要聚合会话附件、商品资产、商品素材、工作区 artifact。
- [x] 后端：返回 `ready/partial/empty` 就绪状态、计数、来源 refs、缺失项、警告和 `agentContext`。
- [x] Web-server：新增 `GET /api/app/workspaces/{workspaceId}/context-summary`。
- [x] Web-server：接口走 JWT 鉴权，并复用工作区、会话、商品、运行记录归属校验。
- [x] API：`API_DOCS/app.md` 已同步新增接口。
- [x] 迁移清单：`附件摘要状态 API` 已标记完成。
- [x] 前端：本阶段不改前端，不新增假展示。

当前说明：

- 本阶段不读取文件内容，不做 OCR/文档解析，不伪造附件摘要。
- 返回的 `agentContext.refs` 是后续 Agent 上下文注入和前端来源面板的稳定桥接字段。

## Post 8-13：工作区上下文摘要接入 Agent 问答主链路

状态：done

验收项：

- [x] 后端：SSE `metadata.context` 支持 `conversationId`。
- [x] 后端：Agent SSE 绑定工作区运行记录后调用 `WorkspaceApi::context_summary`。
- [x] 后端：工作区上下文摘要写入 `workspaceContext.*` metadata，并追加到 `context.refs` / `context.summary`。
- [x] SSE：新增 `workspace_context_loaded` 事件。
- [x] API：`API_DOCS/sse.md`、`API_DOCS/app.md` 已同步。
- [x] 前端：问答发送时传入当前 `conversationId`。
- [x] 前端：右侧过程卡片和时间线识别 `workspace_context_loaded`。

当前说明：

- 本阶段仍不读取附件正文，不做 OCR，不伪造素材或报告摘要。
- `readiness=empty` 会作为真实空状态展示，不代表后端已读取文件内容。
- 验证已通过：`cargo fmt`、`cargo test -p web-server sse`、`cargo check`、`cargo clippy`、`pnpm -C apps/web rules/lint/test/build`、`pnpm rules`、`git diff --check`。

## Post 8-14：工作区上下文内容包接入 Agent 问答链路

状态：done

验收项：

- [x] 后端：新增 `WorkspaceApi::context_pack`。
- [x] 后端：内容包只读取数据库已有文本字段，不读 object storage，不做 OCR。
- [x] 后端：会话附件没有抽取文本时返回 `metadata_only`，不伪造正文。
- [x] Web-server：新增 `GET /api/app/workspaces/{workspaceId}/context-pack`。
- [x] SSE：新增 `workspace_context_pack_loaded`。
- [x] Agent metadata：新增 `workspaceContextPack.*`，并把可读文本追加到 `context.summary`。
- [x] API：`API_DOCS/app.md` 和 `API_DOCS/sse.md` 已同步。
- [x] 前端：同步事件类型、时间线和右侧上下文卡片。

当前说明：

- 本阶段解决的是“已有可读文本进入 Agent 问答链路”。
- PDF/图片/视频等附件正文抽取仍未实现，需要后续独立 adapter。
- 前端不会展示假读取状态，只展示后端返回的 `contentKind`、可读数量和警告。

## Post 8-15：工作区交付物导出能力

状态：done

验收项：

- [x] 后端：新增 `WorkspaceApi::export_artifact`。
- [x] 后端：导出接口复用工作区和 artifact 归属校验，只允许当前用户导出自己的 active 交付物。
- [x] 后端：支持 `markdown`、`text`、`json`，不支持的格式返回 `WORKSPACE_ARTIFACT_REQUEST_INVALID`。
- [x] 后端：只导出 `workspace_artifacts.content` 或 `preview`，不读取 object storage，不伪造文件内容。
- [x] Web-server：新增 `GET /api/app/workspaces/{workspaceId}/artifacts/{artifactId}/export`。
- [x] API：`API_DOCS/app.md` 已同步。
- [x] 前端：工作区交付物详情接入后端导出接口。
- [x] 前端：用户可复制或下载后端导出的交付物内容。

当前说明：

- 本阶段解决的是旧产品“报告/原文导出”在新产品里的后端真实落点。
- 对象存储文件下载、附件解析、OCR 和批量导出仍是后续独立能力。

## Post 8-16：会话附件抽取文本登记 API

状态：done

验收项：

- [x] 后端设计文档已写入 `temp/DEVELOPMENT_DOCS/backend/`。
- [x] Repo：新增 `ConversationFileService::update_metadata`，只操作 `conversation_files` 单表。
- [x] Service：新增 `ConversationApi::update_file_extraction`。
- [x] Service：校验当前用户、附件归属、附件 active 状态、会话归属和会话未删除。
- [x] Service：抽取文本 trim 后不能为空，最多 100000 字符。
- [x] Service：写入 `metadata.extractedText`、`metadata.textContent` 和 `metadata.extraction.*`。
- [x] Web-server：新增 `PUT /api/app/conversation-files/{fileId}/extraction`。
- [x] API：`API_DOCS/app.md` 已同步。
- [x] 前端：API adapter 类型已同步，但本阶段不新增页面入口。

当前说明：

- 本阶段解决的是“附件可读正文如何进入新产品上下文体系”的后端入口。
- 本阶段不实现 OCR、PDF/Office 解析、图片识别或 object storage 读取。
- 后续解析器、OCR worker 或附件管理 UI 都应通过该 API 登记抽取结果。

## Post 8-43：会话附件解析状态契约

状态：done

验收项：

- [x] 后端设计文档已写入 `temp/DEVELOPMENT_DOCS/backend/`。
- [x] Service：新注册会话附件默认写入 `metadata.extractionStatus=pending`。
- [x] Service：`ConversationApi::update_file_extraction` 支持 `pending/extracted/failed/unsupported`。
- [x] Service：旧调用不传 `status` 时按 `extracted` 兼容；旧值 `ready/success` 归一为 `extracted`。
- [x] Service：`status=extracted` 必须有非空 `text`；其它状态会清除旧可读文本，避免 Agent 读取过期正文。
- [x] Context pack：只在 `extractionStatus=extracted` 时读取附件正文。
- [x] Context pack：返回 `extraction_pending`、`extraction_failed`、`extraction_unsupported` 作为真实不可读状态。
- [x] API：`API_DOCS/app.md` 和 `API_DOCS/sse.md` 已同步。
- [x] 前端：`apps/web` API adapter 类型已同步，但本阶段不新增页面入口。

当前说明：

- 本阶段解决的是“附件解析状态和可读边界”。
- 本阶段不实现 OCR、PDF/Office 解析、图片识别、object storage 读取或后台 parser worker。
- 后续解析 adapter 应通过同一个接口登记成功/失败/不支持状态。

## Post 8-44：平台执行 Readiness 诊断契约

状态：done

验收项：

- [x] 后端设计文档已写入 `temp/DEVELOPMENT_DOCS/backend/`。
- [x] Service：平台写类审批创建时写入 `result.executionReadiness`。
- [x] Service：readiness 覆盖 `validation_error`、`connection_missing`、`connection_disabled`、`unsupported_action`、`adapter_not_implemented`。
- [x] Service：当前真实平台 adapter 未实现时不会返回 `ready`。
- [x] Service：审批通过后的执行规划优先识别未知平台动作为 `unsupported_action`。
- [x] API：`API_DOCS/app.md` 和 `API_DOCS/sse.md` 已同步。
- [x] 前端：审批页展示 readiness 的 `adapterStatus/message`。
- [x] 前端：工作区执行建议展示真实执行结果的 `adapterStatus/message`。
- [x] 前端：未新增假执行按钮或假成功状态。

当前说明：

- 本阶段解决的是“平台写操作审批前后都能看到真实执行失败原因”。
- 本阶段不实现淘宝/JD/PDD/抖音等真实平台写 adapter。
- 后续接入真实 adapter 时，应继续复用当前审批、执行队列和审计闭环。

## Post 8-45：平台执行 Adapter 能力目录

状态：done

验收项：

- [x] 后端设计文档已写入 `temp/DEVELOPMENT_DOCS/backend/`。
- [x] Service：新增 `PlatformConnectApi::execution_adapter_catalog`。
- [x] Service：能力目录复用当前用户 `platform_connections`，不新增数据库表。
- [x] Service：目录覆盖 `platform_update_price`、`platform_update_inventory`、`platform_sync_products`。
- [x] Service：每个动作返回风险等级、审批要求、参数、幂等字段、adapter 状态、readiness 和可执行状态。
- [x] Service：当前真实平台 adapter 未实现时 `canExecuteNow=false`，不会返回假成功。
- [x] Web-server：新增 `GET /api/app/platform-connect/execution-adapters`。
- [x] API：`API_DOCS/app.md` 已同步。
- [x] 前端：平台连接页展示只读平台执行 adapter 能力目录。
- [x] 前端：未新增假执行按钮或假成功状态。

当前说明：

- 本阶段解决的是“用户先看清平台写能力目录和当前不可执行原因”。
- 本阶段不实现淘宝/JD/PDD/抖音等真实平台写 adapter。
- 后续接入真实 adapter 时，应继续复用该目录、审批、执行队列和审计闭环。

## Post 8-46：平台执行 Preflight 预检契约

状态：done

验收项：

- [x] 后端设计文档已写入 `temp/DEVELOPMENT_DOCS/backend/`。
- [x] Service：新增 `PlatformConnectApi::execution_preflight`。
- [x] Service：preflight 复用当前用户 `platform_connections`，不新增数据库表。
- [x] Service：参数解析兼容顶层字段、`normalized_inputs`、`normalizedInputs`、`output.normalized_inputs`、`output.normalizedInputs`。
- [x] Service：预检覆盖 `validation_error`、`connection_missing`、`connection_disabled`、`unsupported_action`、`adapter_not_implemented`。
- [x] Service：当前真实平台 adapter 未实现时 `canExecuteNow=false`，不会返回假成功。
- [x] Web-server：新增 `POST /api/app/platform-connect/execution-preflight`。
- [x] API：`API_DOCS/app.md` 已同步。
- [x] 前端：`apps/web` API adapter 类型和请求函数已同步。
- [x] 前端：未新增假执行按钮或假成功状态。

当前说明：

- 本阶段解决的是“创建平台写审批前先做真实预检”。
- 本阶段不创建审批、不创建执行队列、不调用淘宝/JD/PDD/抖音等真实平台写 adapter。
- 后续审批表单、Agent 执行建议和真实平台 adapter 接入应复用该 preflight 契约。

## Post 8-47：平台执行共享规则内核

状态：done

验收项：

- [x] 后端设计文档已写入 `temp/DEVELOPMENT_DOCS/backend/`。
- [x] Service：新增内部共享模块 `api::platform_execution`。
- [x] Service：平台执行参数解析兼容顶层字段、`normalized_inputs`、`normalizedInputs`、`output.normalized_inputs`、`output.normalizedInputs`。
- [x] Service：平台执行 readiness、参数校验、支持动作判断和 adapter 未实现文案由共享模块提供。
- [x] Service：`approval_audit` 复用共享模块生成审批 readiness 和执行队列失败详情。
- [x] Service：`platform_connect` 复用共享模块生成 preflight 结果。
- [x] API：外部响应结构不变，`API_DOCS/app.md` 已补实现说明。
- [x] 前端：本轮不需要改前端，未新增假执行按钮或假成功状态。

当前说明：

- 本阶段解决的是“同一平台执行动作在预检、审批和执行失败详情里使用同一套后端规则”。
- 本阶段不新增 HTTP API，不调用真实平台写 adapter，不改变 `canExecuteNow=false` 的当前边界。
- 后续接入真实淘宝/JD/PDD/抖音 adapter 时，应先扩展该共享规则，再接审批和执行队列闭环。

## Post 8-48：平台执行动作规格注册表

状态：done

验收项：

- [x] 后端设计文档已写入 `temp/DEVELOPMENT_DOCS/backend/`。
- [x] Service：`api::platform_execution` 新增 `PlatformExecutionActionSpec`。
- [x] Service：共享动作规格覆盖 `platform_update_price`、`platform_update_inventory`、`platform_sync_products`。
- [x] Service：动作规格包含展示名、描述、required/optional 参数、幂等字段和 adapter 未实现文案。
- [x] Service：`supported_platform_action`、validation 和 adapter message 复用共享动作规格。
- [x] Service：`platform_connect` 的 adapter catalog 复用共享动作规格生成 DTO。
- [x] API：外部响应结构不变，`API_DOCS/app.md` 已补实现说明。
- [x] 前端：本轮不需要改前端，未新增假执行按钮或假成功状态。

当前说明：

- 本阶段解决的是“平台执行动作定义分散在 catalog 和 preflight/审批规则之间”的问题。
- 本阶段不新增 HTTP API，不调用真实平台写 adapter，不改变 `canExecuteNow=false` 的当前边界。
- 后续接入真实平台 adapter 时，应继续扩展该动作规格注册表，再实现实际 adapter 调用。

## Post 8-49：平台执行 Adapter 请求信封

状态：done

验收项：

- [x] 后端设计文档已写入 `temp/DEVELOPMENT_DOCS/backend/`。
- [x] Service：`api::platform_execution` 新增 `PlatformExecutionAdapterRequest`。
- [x] Service：支持平台执行动作可生成规范化 `adapterRequest`。
- [x] Service：`adapterRequest` 包含 `skillName`、`platform`、`executionMode`、`adapterStatus`、`idempotencyKey`、`idempotencyKeyFields` 和 `args`。
- [x] Service：幂等键基于共享动作规格的 `idempotency_key_fields` 生成。
- [x] Service：`approval_audit` 的平台执行失败结果写入 `adapterRequest`。
- [x] Service：unsupported 动作或缺平台时 `adapterRequest=null`，不伪造可执行请求。
- [x] API：外部路由不变，`API_DOCS/app.md` 已补结果字段说明。
- [x] 前端：本轮不需要改前端，未新增假执行按钮或假成功状态。

当前说明：

- 本阶段解决的是“真实 adapter 接入前缺少统一请求信封和幂等键”的问题。
- 本阶段不调用真实平台写 adapter，不改变 `adapter_not_implemented` 的当前边界。
- 后续接入真实平台 adapter 时，应直接消费该 request envelope。

## Post 8-50：平台执行 Adapter Outcome 契约

状态：done

验收项：

- [x] 后端设计文档已写入 `temp/DEVELOPMENT_DOCS/backend/`。
- [x] Service：`api::platform_execution` 新增 `PlatformExecutionAdapterOutcome`。
- [x] Service：新增通用 `platform_execution_adapter_outcome()`。
- [x] Service：新增 `platform_execution_unimplemented_outcome()` 承载当前 `adapter_not_implemented` 结果。
- [x] Service：outcome 包含 `status`、`adapterStatus`、`message`、`adapterRequest`、`providerResult` 和 `retryable`。
- [x] Service：当前真实 adapter 未实现时 `providerResult=null`、`retryable=false`。
- [x] Service：`approval_audit` 的平台执行失败结果改为从 outcome 写入。
- [x] API：外部路由不变，`API_DOCS/app.md` 已补结果字段说明。
- [x] 前端：本轮不需要改前端，未新增假执行按钮或假成功状态。

当前说明：

- 本阶段解决的是“真实 adapter 接入前缺少统一 outcome 契约”的问题。
- 本阶段不调用真实平台写 adapter，不改变 `adapter_not_implemented` 的当前边界。
- 后续接入真实平台 adapter 时，应让 adapter 返回 outcome，再写回 execution queue、tool approval 和 audit。

## Post 8-71：商品 UUID 引用进入 Skill 契约

状态：done

验收项：

- [x] 后端设计文档已写入 `temp/DEVELOPMENT_DOCS/backend/`。
- [x] 新增 `product_ref` / `product_refs` 作为新产品 UUID 商品引用契约。
- [x] 保留旧 `product_id` integer 兼容字段。
- [x] worker 显式选品后自动补偿 `product_ref/product_refs`。
- [x] worker 未选商品时清理 `null` 和空数组，不留下空引用字段。
- [x] creative 商品型 skill schema 和 data readiness 支持 `product_ref/product_refs`。
- [x] web 商品型 skill schema 和 data readiness 支持 `product_ref`。
- [x] design 商品型 skill schema 和 data readiness 支持 `product_ref`。
- [x] ops 上架文案与执行计划 schema 支持 `product_ref`。
- [x] API 文档已同步。

当前说明：

- 本阶段解决“新产品 UUID 商品不能安全进入旧整数 product_id skill 字段”的契约问题。
- 本阶段不新增数据库、不新增 HTTP API、不让 agent crate 直连数据库。
- 后续应逐步把 data / service / platform 相关 skill 的旧 `product_id` 迁到同一 `product_ref` 契约。

## Post 8-72：客服商品查询 product_ref 契约

状态：done

验收项：

- [x] 后端设计文档已写入 `temp/DEVELOPMENT_DOCS/backend/`。
- [x] `service_query_product` schema 新增 `product_ref`。
- [x] `service_query_product` 优先按 `product_ref=product:{uuid}` 匹配本轮显式商品上下文。
- [x] 旧 `product_id` 和 `product_name` 筛选继续兼容。
- [x] 客服岗位自然语言“查询商品 / 商品信息 / 商品资料 / 引用商品”可触发 `service_query_product`。
- [x] 显式选品后 autosuggest input 会带入 `product_ref` 和商品名。
- [x] 未选商品时 autosuggest input 不留下空 `product_ref/product_name`。
- [x] API 文档已同步。

当前说明：

- 本阶段解决的是客服和售后场景无法使用新产品商品引用精确查询显式选品上下文的问题。
- 本阶段不新增数据库、不新增 HTTP API、不让 agent crate 直连数据库。
- `service_query_product` 仍只读取 web-server/service 已校验后注入的 Agent metadata，不会默认读取历史商品或预设商品。

## Post 8-73：客服支持类 Skill 商品引用契约补齐

状态：done

验收项：

- [x] 后端设计文档已写入 `temp/DEVELOPMENT_DOCS/backend/`。
- [x] `service_ticket_handler` schema 和输出支持 `product_ref`。
- [x] `service_faq_playbook` schema 和输出支持 `product_ref`。
- [x] `service_escalation_flow` schema 和输出支持 `product_ref`。
- [x] `service_return_handler` schema 和输出支持 `product_ref`。
- [x] worker 显式选品后自动给客服支持类 skill 补偿 `product_ref`。
- [x] worker 未选商品时不留下空 `product_ref`。
- [x] 旧 `product_id` 兼容字段保留。
- [x] API 文档已同步。

当前说明：

- 本阶段解决客服支持类 skill 无法记录新产品 UUID 商品引用的问题。
- 本阶段不新增数据库、不新增 HTTP API、不让 agent crate 直连数据库。
- `product_ref` 仅表示本轮用户显式选择的商品关联，不代表已查询实时平台商品库。

## Post 8-74：营销活动草稿 product_ref 契约

状态：done

验收项：

- [x] 后端设计文档已写入 `temp/DEVELOPMENT_DOCS/backend/`。
- [x] `coordination_create_campaign` schema 新增 `product_ref`。
- [x] 输出新增 `campaign_creation.product_ref` 和 `normalized_inputs.product_ref`。
- [x] 未显式传入时可从 `context.refs/context.product_ids` 推断 `product_ref`。
- [x] 旧 `product_id` 继续保留并可从 `product_ref` 派生，兼容现有 campaign draft 持久化链路。
- [x] 不触碰平台改价、改库存、同步商品等审批型平台写操作。
- [x] API 文档已同步。

当前说明：

- 本阶段解决营销活动草稿缺少新产品商品引用字段的问题。
- 本阶段不新增数据库、不新增 HTTP API、不让 agent crate 直连数据库。
- 本阶段只处理草稿沉淀，不执行平台动作、不创建审批。

## Post 8-70：显式选品上下文进入 Skill 输入

状态：done

验收项：

- [x] 后端设计文档已写入 `temp/DEVELOPMENT_DOCS/backend/`。
- [x] Agent worker 只从 `context.selected_summary` 解析本轮显式选中商品。
- [x] 商品行解析支持商品 ID、名称、平台、类目、品牌和描述。
- [x] 商品知识行解析支持按 product ID 把 `content` 写入卖点。
- [x] creative 商品型 skill 自动补偿 input 优先使用显式选品。
- [x] ops 商品型 skill 自动补偿 input 优先使用显式选品。
- [x] web 商品型 skill 自动补偿 input 优先使用显式选品。
- [x] design 商品型 skill 自动补偿 input 优先使用显式选品。
- [x] 未选择商品时，不从 `conversation.history_summary` 注入历史商品。
- [x] API 文档已明确 `context.selected_summary` 是商品型 skill input 的唯一选品来源。

当前说明：

- 本阶段解决“用户手动选品后 skill input 没有稳定使用选中商品”和“未选商品时可能误用历史/测试商品”的问题。
- 本阶段不新增 API、不新增数据库表、不改前端。
- 当前旧 skill 的 `product_id` 字段仍偏 integer 语义，本阶段暂不写 UUID，后续需要统一商品引用 schema。

## Post 8-51：平台执行 Adapter 调度边界

状态：done

验收项：

- [x] 后端设计文档已写入 `temp/DEVELOPMENT_DOCS/backend/`。
- [x] Service：`api::platform_execution` 新增 `platform_execution_dispatch_outcome()`。
- [x] Service：dispatch 复用既有 `platform_execution_readiness_status()`。
- [x] Service：dispatch 复用既有 `PlatformExecutionAdapterOutcome`。
- [x] Service：dispatch 覆盖 `validation_error`、`connection_missing`、`connection_disabled`、`unsupported_action` 和 `adapter_not_implemented`。
- [x] Service：`approval_audit` 的审批后平台执行规划改为调用统一 dispatch outcome。
- [x] Service：`approval_audit` 保留事务、execution queue、tool approval 和 audit 写回职责。
- [x] API：外部路由和响应字段保持兼容，`API_DOCS/app.md` 已补内部 dispatch 说明。
- [x] 前端：本轮不需要改前端，未新增假执行按钮或假成功状态。

当前说明：

- 本阶段解决的是“审批后平台执行调度边界仍散在审批模块里”的问题。
- 本阶段仍不调用真实平台写 adapter，不改变 `adapter_not_implemented` 的当前边界。
- 后续接入真实平台 adapter 时，应挂在该 dispatch 边界之后，并继续返回 `PlatformExecutionAdapterOutcome`。

## Post 8-17：竞品分析读取 API

状态：done

验收项：

- [x] 后端设计文档已写入 `temp/DEVELOPMENT_DOCS/backend/`。
- [x] Service：新增 `ProductCompetitorApi::competitor_analysis`。
- [x] Service：复用已有商品、竞品、竞品变化记录表，不新增 migration。
- [x] Service：按当前用户和商品归属校验权限，已归档商品不可分析。
- [x] Service：返回竞品数量、价格覆盖、评分覆盖、销量字段覆盖和近期变化数量。
- [x] Service：返回价格带、竞品均价、商品价格相对竞品均价的位置。
- [x] Service：返回平台分布、Top 竞品、近期变化、refs、signals、warnings。
- [x] Service：返回 `agentContext`，供后续 Agent 问答和 skill 复用。
- [x] Web-server：新增 `GET /api/app/products/{productId}/competitors/analysis`。
- [x] API：`API_DOCS/app.md` 已同步。
- [x] 前端：API adapter 类型已同步，但本阶段不新增页面入口。

当前说明：

- 本阶段解决的是“竞品散数据可被统一分析读取”。
- 本阶段不调用 LLM，不触发外部搜索，不自动创建 artifact。
- 后续可继续做自动搜索竞品、立即监控竞品、分析结果沉淀和前端展示。

## Post 8-18：Campaign draft 创建 API

状态：done

验收项：

- [x] 后端设计文档已写入 `temp/DEVELOPMENT_DOCS/backend/`。
- [x] Service：新增 `CampaignApi::create_campaign_draft`。
- [x] Service：复用已有 `campaigns` 表，不新增 migration。
- [x] Service：支持商品、工作区、交付物来源归属校验。
- [x] Service：创建活动时强制 `status = draft`。
- [x] Service：把 source、planning、refs、workspace、artifact 写入 `metadata`。
- [x] Service：把 draft 和 planning 写入 `output`。
- [x] Service：把后续执行提示和审批要求写入 `action_metadata`。
- [x] Web-server：新增 `POST /api/app/campaigns/draft`。
- [x] API：`API_DOCS/app.md` 已同步。
- [x] 前端：API adapter 类型已同步，但本阶段不新增页面入口。

当前说明：

- 本阶段解决的是“Agent/工作区输出能保存为营销活动草稿”。
- 本阶段不调用 LLM，不发布平台活动，不绕过后续审批。
- 后续可继续做前端交付物入口、审批流和平台执行 adapter。

## Post 8-19：产品生命周期状态 API

状态：done

验收项：

- [x] 后端设计文档已写入 `temp/DEVELOPMENT_DOCS/backend/`。
- [x] Repo：扩展 `ProductStatus` 为 `draft`、`listing`、`active`、`archived`。
- [x] Repo：创建商品支持写入指定生命周期状态，不传仍默认 `active`。
- [x] Service：新增生命周期状态目录、允许流转和 Agent 上下文。
- [x] Service：新增商品生命周期查询，已归档商品可查询但没有下一步动作。
- [x] Service：新增商品生命周期流转，校验 `draft -> listing -> active -> archived` 及 `listing -> draft`。
- [x] Service：非法状态或非法流转返回 `PRODUCT_REQUEST_INVALID`。
- [x] Service：流转成功后写入 `metadata.lifecycle.currentStatus`、`lastTransition` 和最近历史。
- [x] Web-server：新增 `GET /api/app/products/lifecycle/options`。
- [x] Web-server：新增 `GET /api/app/products/{productId}/lifecycle`。
- [x] Web-server：新增 `PATCH /api/app/products/{productId}/lifecycle`。
- [x] API：`API_DOCS/app.md` 已同步。
- [x] 前端：API adapter 类型已同步，但本阶段不新增页面入口。

当前说明：

- 本阶段解决的是“商品生命周期成为后端真实状态机”。
- 本阶段不自动执行平台上架、下架或发布动作。
- 后续可继续做商品页生命周期 UI、Agent 阶段感知和平台执行 adapter。

## Post 8-20：Skill run 记录查询 API

状态：done

验收项：

- [x] 后端设计文档已写入 `temp/DEVELOPMENT_DOCS/backend/`。
- [x] Migration：新增 `agent_skill_runs` 表，包含 workspace/run/session/worker/task/request/skill/status/output/risk/approval/time 字段。
- [x] Repo：新增 `AgentSkillRunService`，支持创建、按 request 查询、分页列表、计数和更新。
- [x] Service：新增 skill run 查询接口，并复用当前用户、workspace、run 归属校验。
- [x] Service：新增 `record_agent_skill_requested`，把 `skill_requested` 写为 `requested` 记录。
- [x] Service：新增 `record_agent_skill_result`，按 `requestId` 更新结果；缺少请求记录时补写结果记录。
- [x] Web-server：SSE 收到 `SkillRequested` / `SkillResult` 时自动持久化，不截断原始回答流。
- [x] Web-server：新增 `GET /api/app/workspaces/{workspaceId}/runs/{runId}/skill-runs`。
- [x] API：`API_DOCS/app.md` 已同步。
- [x] 前端：API adapter 类型已同步，但本阶段不新增页面入口。

当前说明：

- 本阶段解决的是“skill 调用历史可查询、可审计”。
- 本阶段不从 LLM 文本反推 skill input，`input` 预留为空对象，避免伪造参数。
- 搜索证据 artifact、审批记录和告警仍按既有链路沉淀；后续前端可用 `requestId/skillId/runId` 做关联展示。

## Post 8-119：运营岗位实时搜索链路

状态：done

验收项：

- [x] 后端设计文档已写入 `temp/DEVELOPMENT_DOCS/backend/`。
- [x] Agent worker 新增 ops 岗位 native + search 双 skill 编排。
- [x] 普通运营定价、促销、渠道、货盘规划任务仍只触发 native skill。
- [x] 智能定价 / 定价策略在用户要求实时竞品价、市场定价或竞品价格区间时追加 `search_competitor(focus=price)`。
- [x] 促销策划在用户要求平台活动规则、大促规则或报名规则时追加 `search_platform_policy(policy_type=promotion)`。
- [x] 促销策划在用户要求热门玩法或营销节点时追加 `search_market_info(topic=news)`。
- [x] 渠道策略在用户要求行业 ROI 基准时追加 `search_industry_benchmarks(metric=roi)`。
- [x] 渠道策略在用户要求渠道趋势、流量趋势或获客成本时追加 `search_market_info(topic=general)`。
- [x] 货盘 / 选品规划在用户要求热卖趋势、品类趋势或选品机会时追加 `search_trends`。
- [x] 未显式选择商品时，不从历史摘要或默认演示商品注入 `product_ref/product_name/product_category`。
- [x] API/SSE 文档已同步。

当前说明：

- 本阶段解决的是“运营岗位实时资料诉求可见化”，不是直接接通淘宝、抖音或其它官方平台。
- 搜索 provider 未配置、失败或无可信来源时仍按搜索 skill 降级，不伪造 live 结果。
- 前端后续应把该双 skill 链路展示为过程卡片和来源面板。

## Post 8-120：客服评价/口碑搜索链路

状态：done

验收项：

- [x] 后端设计文档已写入 `temp/DEVELOPMENT_DOCS/backend/`。
- [x] Agent worker 新增客服岗位 native + `search_product_reviews` 双 skill 编排。
- [x] 普通客服情绪分析、工单处理、退换货处理、满意度驱动因素分析仍只触发 native skill。
- [x] 用户明确要求真实评价 / 商品评价 / 买家评价 / 口碑 / 差评 / 吐槽 / 真实反馈 / 联网 / 搜索 / 最新时，才追加 `search_product_reviews`。
- [x] 差评、负面、投诉、吐槽、不满意会映射为 `focus=negative`。
- [x] 好评、正面、满意会映射为 `focus=positive`；其它评价诉求使用 `focus=overall`。
- [x] 用户说“最近30天、最多5条来源”时，时间窗口和来源数量会分别落到 `days=30`、`max_results=5`。
- [x] 未显式选择商品时，不从历史摘要或默认演示商品注入 `product_ref/product_name/product_category`。
- [x] API/SSE 文档已同步。

当前说明：

- 本阶段解决的是“客服岗位真实评价/口碑诉求可见化”，不是直接接通淘宝、抖音或其它官方评价 API。
- 搜索 provider 未配置、失败或无可信来源时仍按搜索 skill 降级，不伪造 live 评价。
- 前端后续应把该双 skill 链路展示为过程卡片和来源面板。

## Post 8-121：财务健康/财务基准搜索链路

状态：done

验收项：

- [x] 后端设计文档已写入 `temp/DEVELOPMENT_DOCS/backend/`。
- [x] Agent worker 新增财务岗位 native + `search_market_info(topic=finance)` 双 skill 编排。
- [x] 财务健康诊断、P&L、GMV利润瀑布、预算执行分析可由自然语言触发对应 native skill。
- [x] 用户明确要求行业财务基准 / 财务对标 / 毛利率基准 / 净利率基准 / 广告 ROI 基准 / 联网 / 搜索 / 最新时，才追加财务搜索。
- [x] 纯搜索类“查最新电商行业财务基准”不会误触发预算编制或财务诊断。
- [x] 财务基准搜索固定写入 `topic=finance`，默认 `days=60`，并支持解析 `max_results`。
- [x] 未显式选择商品时，不从历史摘要或默认演示商品注入 `product_ref/product_name/product_category`。
- [x] API/SSE 文档已同步。

当前说明：

- 本阶段解决的是“财务诊断行业基准诉求可见化”，不是直接接通财报数据库、淘宝、抖音或第三方财务数据 API。
- 搜索 provider 未配置、失败或无可信来源时仍按搜索 skill 降级，不伪造 live 财务基准。
- 前端后续应把该双 skill 链路展示为过程卡片和来源面板。

## Post 8-122：设计岗位视觉趋势搜索链路

状态：done

验收项：

- [x] 后端设计文档已写入 `temp/DEVELOPMENT_DOCS/backend/`。
- [x] Agent worker 新增设计岗位 native + `search_trends` 双 skill 编排。
- [x] 普通主图、详情页、配色、活动海报、素材规范设计任务仍只触发 native skill。
- [x] 用户明确要求视觉趋势 / 设计趋势 / 爆款主图 / 高点击率 / 爆款海报 / 竞品视觉 / 最新 / 实时 / 联网 / 搜索 / 公开资料时，才追加视觉趋势搜索。
- [x] 纯搜索类“查最新爆款主图视觉趋势”不会误触发设计 brief。
- [x] 主图/详情页趋势搜索默认 `days=60`，活动海报默认 `days=30`，并支持解析 `max_results`。
- [x] 未显式选择商品时，不从历史摘要或默认演示商品注入 `product_ref/product_name/product_category`。
- [x] API/SSE 文档已同步。

当前说明：

- 本阶段解决的是“设计岗位视觉趋势诉求可见化”，不是直接生成图片文件，也不是直接接通淘宝、抖音或第三方素材平台。
- 搜索 provider 未配置、失败或无可信来源时仍按搜索 skill 降级，不伪造 live 设计趋势。
- 前端后续应把该双 skill 链路展示为过程卡片和来源面板。

## Post 8-123：工程岗位外部资料搜索链路

状态：done

验收项：

- [x] 后端设计文档已写入 `temp/DEVELOPMENT_DOCS/backend/`。
- [x] Agent worker 新增工程岗位 native + `search_market_info(topic=general)` 双 skill 编排。
- [x] 普通 Bug 排查、技术选型、性能优化、架构评估仍只触发 native skill。
- [x] 用户明确要求联网 / 搜索 / 公开资料 / 已知解决方案 / 最佳实践 / 最新对比 / 性能 benchmark 时，才追加外部资料搜索。
- [x] Bug 排查搜索 query 包含错误类型、技术栈、解决方案、排查和最佳实践。
- [x] 技术选型搜索 query 包含选型领域、技术选型、对比、性能和 benchmark。
- [x] 工程搜索默认 `days=90`，并支持解析 `max_results`。
- [x] API/SSE 文档已同步。

当前说明：

- 本阶段解决的是“工程岗位外部资料诉求可见化”，不是直接读取代码仓库、日志、APM 或平台 API。
- 搜索 provider 未配置、失败或无可信来源时仍按搜索 skill 降级，不伪造 live benchmark 或社区案例。
- 前端后续应把该双 skill 链路展示为过程卡片和来源面板。

## Post 8-124：数据岗位外部趋势 / 基准搜索链路

状态：done

验收项：

- [x] 后端设计文档已写入 `temp/DEVELOPMENT_DOCS/backend/`。
- [x] Agent worker 新增数据岗位 native + search 双 skill 编排。
- [x] 普通渠道 ROI、归因、退款、趋势、需求预测、多周期、季节性、异常、综合诊断和看板任务仍只触发 native skill。
- [x] 用户明确要求最新 / 实时 / 联网 / 公开资料 / 行业基准 / 市场趋势时，才追加外部搜索。
- [x] 渠道 ROI / 归因外部基准追加 `search_industry_benchmarks(metric=roi)`。
- [x] 退款外部基准追加 `search_industry_benchmarks(metric=refund)`。
- [x] 趋势预测 / 需求预测 / 多周期趋势 / 季节性分解追加 `search_trends`。
- [x] 异常诊断追加 `search_market_info(topic=news)`，综合诊断 / 看板追加 `search_market_info(topic=general)`。
- [x] `max_results` 和 `days` 分别从来源数量和明确时间窗口解析，不把“最多4条来源”误当作 `days=4`。
- [x] 未显式选择商品时，不从历史摘要、预设商品或默认演示商品注入 `product_ref/product_name/product_category`。
- [x] API/SSE 文档已同步。

当前说明：

- 本阶段解决的是“数据岗位外部趋势 / 基准诉求可见化”，不是直接接通淘宝、抖音、千川或第三方行业数据库。
- 搜索 provider 未配置、失败或无可信来源时仍按搜索 skill 降级，不伪造 live 行业基准、市场趋势或公开报告。
- 前端后续应把该双 skill 链路展示为过程卡片和来源面板。

## Post 8-125：客服平台规则搜索链路

状态：done

验收项：

- [x] 后端设计文档已写入 `temp/DEVELOPMENT_DOCS/backend/`。
- [x] Agent worker 新增客服岗位 native + `search_platform_policy(policy_type=general)` 双 skill 编排。
- [x] 普通工单处理、FAQ话术、升级处理、退换货处理仍只触发 native skill。
- [x] 用户明确要求平台规则 / 售后规则 / 退换货规则 / 赔付标准 / 运费规则 / 投诉规则 / 平台介入规则 / 七天无理由 / 联网 / 最新时，才追加平台规则搜索。
- [x] 平台规则搜索复用已有 `search_platform_policy`，不新增第二套平台规则 adapter。
- [x] “最多4条来源”等来源数量限制会写入 `max_results`。
- [x] 未显式选择商品时，不从历史摘要、预设商品或默认演示商品注入 `product_ref/product_name/product_category`。
- [x] API/SSE 文档已同步。

当前说明：

- 本阶段解决的是“客服平台规则诉求可见化”，不是直接接通淘宝、抖音、京东或其它官方售后规则 API。
- 搜索 provider 未配置、失败或无可信来源时仍按搜索 skill 降级，不伪造 live 平台政策、赔付标准或售后规则。
- 前端后续应把该双 skill 链路展示为过程卡片和来源面板。
