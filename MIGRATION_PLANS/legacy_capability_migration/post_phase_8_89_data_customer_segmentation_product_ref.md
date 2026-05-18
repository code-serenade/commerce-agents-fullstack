# Post 8-89：客户分层 product_ref 契约

## 目标

把旧产品里常用的客户 / 人群 / RFM / 分层能力，按新产品 Rust agent skill 框架继续补强：

- `data_customer_segmentation` 接受 `product_ref=product:{uuid}`。
- 成功输出回显 `商品引用`。
- data worker 的客户分层问法能携带本轮显式选品引用。
- 未选商品时不写空引用，不注入历史商品、默认商品或测试商品。

## 后端变化

- `backend/crates/agent/src/skills/data.rs`
  - `data_customer_segmentation` schema 新增可选 `product_ref`。
  - 成功结果新增 `商品引用` 字段。
  - 保持缺少 RFM 聚合输入时使用 deterministic 默认估算并通过 `data_readiness=calculated_from_defaults` 明示的既有行为。
- `backend/crates/agent/src/agent/worker/runner.rs`
  - 客户 / 人群 / RFM / 分层 autosuggest 改为结构化 input 推断。
  - 显式 `客户总数`、`客单价`、`人均订单数`、`平均购买间隔` 会写入 input。
  - `product_ref` 只来自 `context.selected_summary` 的 `commerce.selected_product_context`。

## 用户可感知变化

用户在前端手动勾选商品后，提问“分析这个商品的客户分层 / 人群分层 / RFM”，后端结果会带上本轮商品引用。

用户没有勾选商品时，同样的问题会走普通客户分层，不会暗中绑定任何商品。

如果用户在问题里给出客户总数、客单价、人均订单数、平均购买间隔等 RFM 聚合指标，后端会把这些指标当作本轮显式输入；如果没有给出，系统会继续使用 deterministic 默认估算，并用 `data_readiness` 明示，不会伪造会员系统或 CRM 数据。

## 边界

本阶段不读取会员系统、订单系统、CRM、DMP 或平台人群包。

真实会员、人群包、复购行为和平台 DMP 数据，需要后续通过 service、adapter 或 search 能力接入。

