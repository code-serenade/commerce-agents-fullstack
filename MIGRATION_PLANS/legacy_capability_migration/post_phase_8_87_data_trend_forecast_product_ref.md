# Post 8-87：通用趋势预测 product_ref 契约

## 目标

把旧产品里最常用的普通趋势 / 预测能力，按新产品 Rust agent skill 框架继续补强：

- `data_trend_forecast` 接受 `product_ref=product:{uuid}`。
- 成功输出回显 `商品引用`。
- data worker 的趋势 / 预测兜底问法能携带本轮显式选品引用。
- 未选商品时不写空引用，不注入历史商品、默认商品或测试商品。

## 后端变化

- `backend/crates/agent/src/skills/data.rs`
  - `data_trend_forecast` schema 新增可选 `product_ref`。
  - 成功结果新增 `商品引用` 字段。
  - 保持少于 2 个时序点时返回 deferred 的既有行为。
- `backend/crates/agent/src/agent/worker/runner.rs`
  - 趋势 / 预测兜底 autosuggest 改为结构化 input 推断。
  - 显式数字序列会写入 `values`。
  - `forecast_periods` 尝试读取“未来 / 预测”后的数字，默认 7。
  - `product_ref` 只来自 `context.selected_summary` 的 `commerce.selected_product_context`。

## 用户可感知变化

用户在前端手动勾选商品后，提问“预测这个商品 GMV / 订单 / UV 未来趋势”，后端结果会带上本轮商品引用。

用户没有勾选商品时，同样的问题会走普通趋势预测，不会暗中绑定任何商品。

如果用户在问题里给出 GMV、订单、UV 等历史数字，后端会把这些数字当作本轮显式输入；如果数字不足，系统会要求补充数据，而不是伪造平台历史数据。

## 边界

本阶段不读取淘宝、抖音、千川、ERP、CRM 或其它外部系统历史趋势。

真实平台历史、行业周期、活动影响和跨平台趋势，需要后续通过 service、adapter 或 search 能力接入。

