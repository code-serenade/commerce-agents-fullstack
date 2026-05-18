# Post 8-95 Ops Ad Fatigue Product Ref

## 状态

done

## 本次目标

- 把广告疲劳检测 `ops_ad_fatigue_detector` 接入新产品 `product_ref=product:{uuid}` 契约。
- 让用户手动选择商品后，广告疲劳 / 素材疲劳 / CTR 衰减 / 曝光频次 / ROAS 健康等问法能保留本轮商品引用。
- 让用户本轮文本里提供的渠道广告指标进入真实广告疲劳检测 skill input。
- 继续保持未选商品时不绑定历史商品或预设商品，也不读取或伪造淘宝、抖音、千川、广告平台或实时数仓数据。

## 本次修改

- 后端 agent：`ops_ad_fatigue_detector` schema 新增 `product_ref`。
- 后端 agent：成功输出新增 `商品引用`。
- 后端 worker：新增广告疲劳自然语言自动补偿。
- 后端 worker：解析本轮显式渠道行里的 `CTR基准`、`近期CTR`、`曝光频次/频次`、`预算` 和 `ROAS`。
- 后端 worker：本轮显式选品后，广告疲劳 input 带入 `product_ref`。
- 后端 worker：未选商品时不写空 `product_ref`。
- API/SSE：同步 `API_DOCS/sse.md`。
- 迁移文档：同步阶段验收、总计划和 skill 矩阵。

## 用户可感知变化

- 用户手动勾选商品后，再问“给这个商品检测广告疲劳 / 素材疲劳 / CTR 衰减 / 曝光频次”时，后端会把本轮选中商品的 `product:<uuid>` 带进广告疲劳检测结果。
- 用户没有手动选商品时，广告疲劳检测仍按普通广告问题运行，但不会暗中绑定历史商品或默认商品。
- 用户在问题里写出不同渠道的 CTR 基准、近期 CTR、频次、预算和 ROAS 时，后端会解析成本轮显式广告疲劳检测输入；缺渠道指标时继续 `missing_user_input` / `deferred`，不能当作平台实时广告数据。

## 验证

- [x] `cargo fmt --all`
- [x] `cargo test -p agent ops_ad_fatigue_detector -- --nocapture`
- [x] `cargo test -p agent autosuggests_ad_fatigue_detector_with_explicit_channel_rows -- --nocapture`
- [x] `cargo test -p agent selected_product_context_feeds_ad_fatigue_detector_ref -- --nocapture`
- [x] `cargo test -p agent ad_fatigue_detector_without_selection_has_no_empty_product_ref -- --nocapture`
- [x] `cargo test -p agent autosuggests_more_specific_ops_skills_by_intent -- --nocapture`

## 后续缺口

- 真实淘宝、抖音、千川、广告平台、CRM、DMP 和实时数仓广告数据仍需要后续 service/adapter/search 注入，不能由本地 native skill 伪造或默认读取。
- 前端后续应把 `skill_result.output.商品引用`、`渠道疲劳分析`、`建议预算分配` 和 `deferred` 状态展示在真实过程面板与交付物详情中。
