# Post 8-46：平台执行 Preflight 预检契约

## 1. 业务理解

旧产品的平台执行类能力不能只停留在“审批后失败”。新产品已经有：

- 平台执行 adapter 能力目录。
- 平台写操作审批 readiness。
- 审批通过后的执行队列失败结果。

本轮补齐“创建审批前”的后端预检入口，让前端、Agent 或后续审批创建流程可以先用同一套后端规则判断：

- 这个动作是不是受支持的平台执行动作。
- 参数是否足够。
- 当前用户是否有对应平台连接。
- 连接是否启用。
- 真实平台 adapter 是否已经实现。

该能力只做只读校验，不创建审批，不创建执行队列，不调用真实平台接口。

## 2. 数据库设计

不新增表，不新增 migration。

复用现有表：

- `app_users`：校验当前 App 用户可用。
- `platform_connections`：读取当前用户对应平台连接和启用状态。

本接口是纯读接口，不写审计日志，避免把普通表单预检刷成业务操作日志。后续若需要审计高风险预检，可单独补日志策略。

## 3. Service 设计

新增 DTO：

- `PlatformExecutionPreflightRequest`
- `PlatformExecutionPreflightResponse`

请求字段：

- `user_id`：来自 JWT，web-server 注入。
- `skill_name`：平台执行动作，例如 `platform_update_price`。
- `args`：动作参数，支持旧审批链路中已经兼容的结构：
  - 顶层字段：`platform`、`product_id`、`productId`、`new_price`、`newPrice`、`quantity` 等。
  - `normalized_inputs` / `normalizedInputs`。
  - `output.normalized_inputs` / `output.normalizedInputs`。

响应字段：

- `status`：`blocked` / `unsupported`。
- `adapterStatus`：`validation_error` / `connection_missing` / `connection_disabled` / `unsupported_action` / `adapter_not_implemented`。
- `canApprove`：当前仍为 `false`，避免用户把审批理解成可执行成功。
- `canExecuteNow`：当前仍为 `false`。
- `willFailIfApproved`：当前为 `true`。
- `platform`。
- `skillName`。
- `message`。
- `details`：归一后的参数。
- `checkedAt`。

## 4. HTTP API 设计

新增：

```text
POST /api/app/platform-connect/execution-preflight
```

鉴权：

- 有效 App 用户 JWT。

请求体：

```json
{
  "skillName": "platform_update_price",
  "args": {
    "platform": "douyin",
    "product_id": "prod-1",
    "new_price": 129.5,
    "sku_id": "sku-9"
  }
}
```

成功响应 `data`：

```json
{
  "status": "blocked",
  "adapterStatus": "adapter_not_implemented",
  "canApprove": false,
  "canExecuteNow": false,
  "willFailIfApproved": true,
  "platform": "douyin",
  "skillName": "platform_update_price",
  "message": "Platform execution adapter is not implemented yet for price update.",
  "details": {
    "productId": "prod-1",
    "skuId": "sku-9",
    "newPrice": 129.5,
    "quantity": null,
    "page": null,
    "pageSize": null
  },
  "checkedAt": "2026-05-13T00:00:00Z"
}
```

## 5. 错误与边界

- `skillName` 为空：请求校验失败。
- `args` 为空：按 `{}` 处理，返回参数缺失类 readiness。
- 非平台动作：返回 `status=unsupported`、`adapterStatus=unsupported_action`。
- 缺平台：返回 `validation_error`。
- 缺必要参数：返回 `validation_error`。
- 缺连接：返回 `connection_missing`。
- 连接禁用：返回 `connection_disabled`。
- 参数和连接满足但 adapter 未实现：返回 `adapter_not_implemented`。

## 6. 开发顺序

1. 在 `platform_connect` service DTO 中增加 preflight 请求/响应。
2. 在 `PlatformConnectApi` 中实现 `execution_preflight`，复用平台连接 repo service。
3. 在 web-server app DTO、handler、route 中挂接新接口。
4. 同步 `API_DOCS/app.md`。
5. 前端先同步 API 类型；若做 UI，只能展示 preflight 结果，不提供真实执行按钮。
6. 增加 service 单测覆盖缺平台、缺连接、adapter 未实现、非平台动作。
7. 运行 cargo 和 pnpm 校验。
