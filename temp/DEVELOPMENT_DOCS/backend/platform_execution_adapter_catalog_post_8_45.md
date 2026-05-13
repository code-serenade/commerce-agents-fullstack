# Post 8-45 平台执行 Adapter 能力目录

## 需求摘要

继续按旧产品能力迁移计划开发，后端优先，前端只适配真实后端状态。本轮承接 Post 8-44 的平台执行 readiness 诊断，增加一个只读的“平台执行 adapter 能力目录”。目标是让系统明确暴露：当前哪些平台写动作已纳入审批治理，哪些参数是必填，是否需要审批，当前 adapter 是否能真实执行，以及当前用户的平台连接状态会让该动作处于何种 readiness。

## 业务边界

- 不新增第二套平台执行系统。
- 不新增表，不新增 migration。
- 不真实调用淘宝、京东、拼多多、抖音等平台写接口。
- 不伪造 `ready` 或执行成功。
- 继续复用 `platform_connections` 判断当前用户连接状态。
- 后续真实 adapter 接入时，应填充本目录中的 `adapterStatus/canExecuteNow`，而不是重做 API。

## 数据表设计

本轮不新增表。

复用已有表：

- `platform_connections`：读取当前用户在各平台的连接、启用和配置状态。

## 后端服务设计

在 `PlatformConnectApi` 中新增只读方法：

```text
execution_adapter_catalog(user_id) -> PlatformExecutionAdapterCatalogResponse
```

返回内容：

- 平台：`taobao/jd/pdd/douyin`
- 支持治理的动作：`platform_update_price`、`platform_update_inventory`、`platform_sync_products`
- 每个动作的参数要求、风险等级、是否需要审批、是否支持幂等、执行模式
- 当前 adapter 状态：当前版本固定为 `adapter_not_implemented`
- 当前用户连接状态：`connected/enabled/connectionStatus`
- readiness：缺连接、连接禁用或 adapter 未实现时明确返回 blocked

## API 设计

新增接口：

- 方法：`GET`
- 路径：`/api/app/platform-connect/execution-adapters`
- 鉴权：有效 App 用户 JWT
- 请求参数：无

成功响应 `data`：

```json
{
  "adapters": [
    {
      "platform": "douyin",
      "display": "抖音电商",
      "connected": true,
      "enabled": true,
      "connectionStatus": "configured",
      "actions": [
        {
          "skillName": "platform_update_price",
          "display": "更新商品价格",
          "description": "审批后更新平台商品或 SKU 价格。",
          "riskLevel": "L3",
          "approvalRequired": true,
          "executionMode": "adapter",
          "adapterStatus": "adapter_not_implemented",
          "canExecuteNow": false,
          "readiness": "blocked",
          "readinessMessage": "Platform execution adapter is not implemented yet for price update.",
          "requiredParams": ["platform", "product_id", "new_price"],
          "optionalParams": ["sku_id"],
          "idempotencyKeyFields": ["platform", "product_id", "sku_id", "new_price"]
        }
      ]
    }
  ]
}
```

## 前端适配设计

前端只消费新增 API：

- 在平台连接页增加“平台执行能力”面板。
- 展示平台、连接状态、动作、风险等级、adapter 状态、readiness 文案。
- 不新增执行按钮。
- 不把 `adapter_not_implemented` 展示成可执行。

## 验证计划

后端：

- `cargo fmt`
- `cargo test -p service platform_execution_adapter_catalog -- --nocapture`
- `cargo check -p service -p web-server`
- `cargo clippy -p service -p web-server`

前端：

- `pnpm -C frontend/apps/web rules`
- `pnpm -C frontend/apps/web lint`
- `pnpm -C frontend/apps/web test`
- `pnpm -C frontend/apps/web build`
- `pnpm -C frontend rules`

## 用户可感知变化

用户进入平台连接页后，可以看到每个平台当前哪些高风险执行能力已经被后端纳入治理，以及为什么现在仍不能真实执行。这个面板不会提供假执行入口，只会展示后端真实能力边界。
