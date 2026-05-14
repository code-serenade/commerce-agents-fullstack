# Post 8-96 Ops ABC-XYZ Product Ref

## 状态

done

## 本次目标

- 把 ABC-XYZ 库存分类 `ops_abc_xyz_classifier` 接入新产品 `product_ref=product:{uuid}` 契约。
- 让用户手动选择商品后，ABC-XYZ / ABC分类 / XYZ分类 / 九宫格库存分类等问法能保留本轮商品引用。
- 让用户本轮文本里提供的 SKU、GMV、历史销量进入真实 ABC-XYZ 分类 skill input。
- 继续保持未选商品时不绑定历史商品或预设商品，也不读取或伪造淘宝、抖音、WMS、ERP、平台库存或实时数仓数据。

## 本次修改

- 后端 agent：`ops_abc_xyz_classifier` schema 新增 `product_ref`。
- 后端 agent：成功输出新增 `商品引用`。
- 后端 worker：新增 ABC-XYZ / ABC分类 / XYZ分类 / 九宫格库存分类自然语言自动补偿。
- 后端 worker：解析本轮显式 SKU 行里的 `GMV` 和 `历史销量`。
- 后端 worker：本轮显式选品后，ABC-XYZ input 带入 `product_ref`。
- 后端 worker：未选商品时不写空 `product_ref`。
- API/SSE：同步 `API_DOCS/sse.md`。
- 迁移文档：同步阶段验收、总计划和 skill 矩阵。

## 用户可感知变化

- 用户手动勾选商品后，再问“给这个商品做 ABC-XYZ 库存分类 / 九宫格库存矩阵 / SKU 分层”时，后端会把本轮选中商品的 `product:<uuid>` 带进分类报告。
- 用户没有手动选商品时，ABC-XYZ 分类仍按普通库存分类问题运行，但不会暗中绑定历史商品或默认商品。
- 用户在问题里写出 `SKU冷萃A GMV12万 历史销量100 102 98 101` 这类明细时，后端会解析成本轮显式 ABC-XYZ 输入；缺 SKU 明细时继续使用确定性演示 SKU 并明确 `calculated_from_defaults`，不能当作平台实时库存数据。

## 验证

- [x] `cargo fmt --all`
- [x] `cargo test -p agent ops_abc_xyz_classifier -- --nocapture`
- [x] `cargo test -p agent autosuggests_abc_xyz_classifier_with_explicit_sku_rows -- --nocapture`
- [x] `cargo test -p agent selected_product_context_feeds_abc_xyz_classifier_ref -- --nocapture`
- [x] `cargo test -p agent abc_xyz_classifier_without_selection_has_no_empty_product_ref -- --nocapture`

## 后续缺口

- 真实淘宝、抖音、WMS、ERP、平台库存和实时数仓 SKU 历史销量仍需要后续 service/adapter/search 注入，不能由本地 native skill 伪造或默认读取。
- 前端后续应把 `skill_result.output.商品引用`、`9格矩阵`、`高价值SKU` 和 `data_readiness=calculated_from_defaults` 展示在真实过程面板与交付物详情中。
