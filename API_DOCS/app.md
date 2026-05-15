# App API

接口前缀：`/api/app`

说明：本文件只记录普通客户端使用的接口。后台管理 App 用户、App 角色、App 角色授权的接口属于后台管理端，统一记录在 `API_DOCS/admin.md`。

## App 权限码

当前初始化脚本 `scripts/init_app_permissions.sh` 会写入以下 App 权限：

| App 权限码 | 类型 | 用途 |
| --- | --- | --- |
| `profile` | `group` | 个人中心 |
| `profile:view` | `action` | 查看个人资料 |
| `profile:update` | `action` | 更新个人资料 |

## 1. 注册 App 用户

- 方法：`POST`
- 路径：`/api/app/register`
- 鉴权：有效 Auth JWT

请求体：

```json
{
  "displayId": "u_10001",
  "displayName": "App User",
  "remark": "optional note"
}
```

成功响应 `data`：`AppUserResponse`。

补充说明：

- `userId` 由服务端从 JWT 当前用户中读取，前端不得传入或伪造
- 首次注册会创建 `enabled` 状态 App 用户
- 首次注册会自动绑定默认 App 角色 `free`
- 如果默认 App 角色 `free` 不存在，服务端会自动创建；初始化脚本 `scripts/init_app_permissions.sh` 也会写入该角色
- 同一个 `userId` 重复注册时返回已有用户信息
- 该接口用于登录后的业务 App 用户初始化；前端应先查询 `/api/app/me/permissions`，仅当当前 Auth 用户尚未初始化为 App 用户时再调用本接口

## 2. 查询当前 App 用户权限

- 方法：`GET`
- 路径：`/api/app/me/permissions`
- 鉴权：有效 App 用户 JWT

成功响应 `data`：

```json
{
  "userId": "1b1f4e1d-5b4f-4d25-ae07-520f587f8d13",
  "roleCodes": [
    "user"
  ],
  "permissionCodes": [
    "profile",
    "profile:view",
    "profile:update"
  ]
}
```

补充说明：

- 当前用户必须存在于 `app_users`，且状态为 `enabled`
- 返回权限码来自该用户绑定的 App 角色
- 如果当前 Auth 用户尚未初始化为 App 用户，返回 `404`，错误码为 `-12000`
- 如果当前 App 用户已被禁用，返回 `403`，错误码为 `-12001`

## 2.1 查询电商智能体场景 Playbook

- 方法：`GET`
- 路径：`/api/app/agent/playbooks`
- 鉴权：有效 App 用户 JWT

成功响应 `data`：

```json
{
  "playbooks": [
    {
      "id": "new_product_launch",
      "title": "新品冷启动",
      "description": "从商品事实、首批人群、内容种草、投放测试和复盘节奏生成冷启动方案。",
      "category": "launch",
      "recommendedStrategy": "collab",
      "prompt": "请以电商多智能体虚拟员工方式...",
      "requiredContext": [
        "product_knowledge",
        "product_material",
        "campaign",
        "platform_metrics"
      ],
      "expectedOutputs": [
        "定位诊断",
        "首批动作清单",
        "预算与止损线",
        "复盘指标"
      ],
      "roles": [
        "product",
        "data",
        "creative",
        "ads",
        "operations"
      ]
    }
  ]
}
```

当前内置场景：

- `new_product_launch`：新品冷启动，推荐 `collab`
- `ad_review`：投放复盘，推荐 `collab`
- `customer_complaint_crisis`：客诉危机，推荐 `execution`
- `selection_pricing`：选品定价，推荐 `learning`
- `creative_optimization`：素材优化，推荐 `collab`
- `daily_business_review`：日报复盘，推荐 `execution`

补充说明：

- 当前用户必须存在于 `app_users`，且状态为 `enabled`
- 该接口是问答页商业场景入口的后端契约，前端不应再硬编码替代样例
- 本阶段 playbook 为 service 层内置目录，暂不支持用户自定义
- `recommendedStrategy` 对齐问答策略：`execution`、`learning`、`collab`

## 2.2 查询 Agent 运行选项

- 方法：`GET`
- 路径：`/api/app/agent/runtime-options`
- 鉴权：有效 App 用户 JWT

成功响应 `data`：

```json
{
  "defaultModelName": "ollama-gpt-oss-20b",
  "models": [
    {
      "name": "ollama-gpt-oss-20b",
      "model": "gpt-oss:20b",
      "isDefault": true
    }
  ],
  "strategies": [
    {
      "id": "collab",
      "label": "Agent 集群",
      "description": "调度多个电商虚拟员工分工处理并汇总。",
      "isDefault": true
    }
  ],
  "responseModes": [
    {
      "id": "execution",
      "label": "执行导向",
      "description": "先输出业务答案和动作清单。",
      "isDefault": true
    }
  ],
  "learningLevels": [
    {
      "id": "higher_vocational",
      "label": "实操",
      "description": "按电商岗位实操标准解释。",
      "isDefault": true
    }
  ],
  "collaborationModes": [
    {
      "id": "manual",
      "label": "指定多岗",
      "description": "用户指定多个岗位协同执行。",
      "isDefault": true
    }
  ],
  "roles": [
    {
      "id": "role.ops",
      "name": "运营",
      "runtimeRole": "ops",
      "capabilities": ["campaign_planning", "execution"],
      "isDefault": false
    }
  ]
}
```

字段说明：

- `models` 来自 `backend/config/services.toml` 的 `[[llm]]` 配置，只返回 `name`、`model`、`isDefault`，不返回 `base_url` 或 `api_key`
- `defaultModelName` 是后端实际默认 `llm.name`
- `strategies`、`responseModes`、`learningLevels`、`collaborationModes` 是后端认可的运行枚举，前端不应自行硬编码替代
- `roles` 来自 Agent `RoleCatalog::configured()`，`runtimeRole` 用于 SSE `metadata.strategy.primaryRole` 和 `selectedRoles`

补充说明：

- 当前用户必须存在于 `app_users`，且状态为 `enabled`
- 该接口只暴露问答工作台可选择的运行契约，不会动态修改模型配置或角色目录
- 前端发送 SSE 时应把用户选择映射到 `metadata.model.name` 与 `metadata.strategy.*`

## 2.3 查询外部搜索 Adapter 状态

- 方法：`GET`
- 路径：`/api/app/external-search/status`
- 鉴权：有效 App 用户 JWT

成功响应 `data`：

```json
{
  "adapter": "external_search",
  "enabled": false,
  "configured": false,
  "provider": null,
  "requiredConfig": "external_search.enabled=true, base_url and search_endpoint are required",
  "status": "degraded"
}
```

字段说明：

- `enabled` 来自 `backend/config/services.toml` 的 `[external_search].enabled`
- `configured=true` 需要 `enabled=true` 且配置了 `base_url`、`search_endpoint`
- `provider` 仅在配置完整时返回 provider 名称
- `status=configured` 表示后端具备向外部搜索 provider 发起请求的配置；`status=degraded` 表示当前不会查询外部搜索

补充说明：

- 当前用户必须存在于 `app_users`，且状态为 `enabled`
- 该接口只暴露搜索 adapter 可用状态，不返回 `api_key`
- 未配置时前端必须展示“外部搜索未接入”，不能伪造来源

## 2.4 查询外部搜索

- 方法：`GET`
- 路径：`/api/app/external-search/search`
- 鉴权：有效 App 用户 JWT

查询参数：

| 参数 | 必填 | 说明 |
| --- | --- | --- |
| `q` | 是 | 查询词，1 到 500 字符 |
| `topic` | 否 | `general`、`news`、`finance`，默认 `general` |
| `days` | 否 | 最近 N 天，1 到 365，默认 30 |
| `maxResults` | 否 | 最大结果数，1 到 12，默认 6 |
| `bypassCache` | 否 | 是否要求 provider 绕过缓存，默认 false |

未配置 adapter 时成功响应 `data`：

```json
{
  "adapter": "external_search",
  "key": "search",
  "status": "degraded",
  "query": "咖啡 趋势",
  "engine": null,
  "requestedAt": "2026-05-10T00:00:00Z",
  "topic": "general",
  "freshnessDays": 30,
  "maxResults": 6,
  "bypassCache": false,
  "confidence": 0.0,
  "provenance": {
    "sourceCount": 0,
    "domainCount": 0,
    "domains": [],
    "publishedCount": 0,
    "latestPublishedAt": null
  },
  "data": {
    "items": []
  },
  "evidence": {
    "summary": "没有可采信实时来源；当前结果只能作为降级状态，不可当作联网证据。",
    "sourceTitles": [],
    "sourceDomains": [],
    "llmContext": "（本次没有可采信实时来源。回答时必须说明外部搜索未返回来源，不能引用不存在的联网数据。）"
  },
  "notice": "未获取到可靠公开来源，当前不能伪造实时搜索结果。",
  "degraded": true,
  "degradedReason": "External search adapter is not configured; no live sources were queried."
}
```

配置 adapter 且 provider 返回来源时成功响应 `data`：

```json
{
  "adapter": "external_search",
  "key": "search",
  "status": "live",
  "query": "咖啡 趋势",
  "engine": "custom",
  "requestedAt": "2026-05-10T00:00:00Z",
  "topic": "news",
  "freshnessDays": 14,
  "maxResults": 6,
  "bypassCache": true,
  "confidence": 0.72,
  "provenance": {
    "sourceCount": 2,
    "domainCount": 2,
    "domains": ["example.com", "report.example.cn"],
    "publishedCount": 1,
    "latestPublishedAt": "2026-05-01T00:00:00Z"
  },
  "data": {
    "items": [
      {
        "rank": 1,
        "title": "咖啡消费趋势",
        "snippet": "冷萃咖啡增长",
        "url": "https://www.example.com/a",
        "domain": "example.com",
        "publishedAt": "2026-05-01T00:00:00Z"
      }
    ]
  },
  "evidence": {
    "summary": "共 2 条实时来源，覆盖 2 个域名；回答应优先引用这些来源，并注明时效性和置信度。",
    "sourceTitles": ["咖啡消费趋势", "咖啡行业报告"],
    "sourceDomains": ["example.com", "report.example.cn"],
    "llmContext": "（共2条实时来源，优先从中提取具体数字、结论和出处；若来源不足，明确说明数据不足。）"
  },
  "notice": null,
  "degraded": false,
  "degradedReason": null
}
```

补充说明：

- 当前用户必须存在于 `app_users`，且状态为 `enabled`
- provider HTTP 失败、返回结构无法解析或没有可归一化来源时，接口返回 200 + `status=degraded`
- 该接口是 Rust 后端外部搜索 adapter 的 App 侧入口；Agent `search_*` skill 已接入同一份 `[external_search]` 配置，并复用相同字段语义
- `maxResults` 和 `bypassCache` 是后端实际使用的搜索控制参数回显；`bypassCache=true` 会转发给 provider，但 provider 是否真的绕过缓存由 provider 自身决定
- `evidence` 是后端归一化后的搜索证据摘要：`sourceTitles`、`sourceDomains` 用于前端来源面板，`llmContext` 用于 Agent/后端复用来源文本
- provider 返回重复 URL 或重复标题时，后端会去重并重新生成 `rank`
- 当前不会在前端伪造搜索结果；只有 `status=live` 且 `data.items` 非空时，前端才可展示为实时来源

## 3. 查询我的会话列表

- 方法：`GET`
- 路径：`/api/app/conversations`
- 鉴权：有效 App 用户 JWT

成功响应 `data`：`ConversationResponse[]`。

## 4. 创建会话

- 方法：`POST`
- 路径：`/api/app/conversations`
- 鉴权：有效 App 用户 JWT

请求体：

```json
{
  "title": "New chat",
  "firstMessage": "hello",
  "agentId": "default-agent"
}
```

成功响应 `data`：`ConversationDetailResponse`。

## 5. 查询会话详情

- 方法：`GET`
- 路径：`/api/app/conversations/{conversationId}`
- 鉴权：有效 App 用户 JWT

成功响应 `data`：`ConversationDetailResponse`，包含 `conversation`、`messages`、`files`。

## 6. 更新会话标题

- 方法：`PATCH`
- 路径：`/api/app/conversations/{conversationId}`
- 鉴权：有效 App 用户 JWT

请求体：

```json
{
  "title": "新的对话名称"
}
```

成功响应 `data`：`ConversationResponse`。

补充说明：

- 只能更新当前用户自己的会话
- `title` 最长 255 个字符，空字符串会按 `null` 处理

## 7. 删除会话

- 方法：`DELETE`
- 路径：`/api/app/conversations/{conversationId}`
- 鉴权：有效 App 用户 JWT

成功响应 `data`：`null`。

补充说明：

- 只能删除当前用户自己的会话
- 当前实现为软删除，删除后不会再出现在会话列表中

## 8. 添加会话消息

- 方法：`POST`
- 路径：`/api/app/conversations/{conversationId}/messages`
- 鉴权：有效 App 用户 JWT

请求体：

```json
{
  "role": "user",
  "contentText": "hello",
  "contentJson": null
}
```

成功响应 `data`：`ConversationMessageResponse`。

补充说明：

- `role` 只能是 `user`、`assistant`、`system`、`tool`
- `contentText` 和 `contentJson` 至少需要一个有值

## 9. 获取会话附件上传签名

- 方法：`POST`
- 路径：`/api/app/conversations/{conversationId}/files/presign-upload`
- 鉴权：有效 App 用户 JWT

请求体：

```json
{
  "extension": "png",
  "contentType": "image/png"
}
```

成功响应 `data`：

```json
{
  "method": "PUT",
  "uploadUrl": "http://s3.example/private/chat/...",
  "bucket": "private",
  "objectKey": "chat/userScope/conversationId/file.png",
  "contentType": "image/png",
  "expiresSecs": 600
}
```

补充说明：

- 聊天附件固定使用 private bucket
- 前端使用 `uploadUrl` 上传成功后，再调用注册附件接口写入业务表

## 10. 注册会话附件

- 方法：`POST`
- 路径：`/api/app/conversations/{conversationId}/files`
- 鉴权：有效 App 用户 JWT

请求体：

```json
{
  "messageId": null,
  "objectKey": "chat/userScope/conversationId/file.png",
  "fileName": "file.png",
  "mimeType": "image/png",
  "fileSize": 12345,
  "sha256": "optional"
}
```

成功响应 `data`：`ConversationFileResponse`。

`ConversationFileResponse`：

```json
{
  "id": "file-uuid",
  "conversationId": "conversation-uuid",
  "messageId": null,
  "bucket": "private",
  "objectKey": "chat/userScope/conversationId/file.png",
  "fileName": "file.png",
  "mimeType": "image/png",
  "fileSize": 12345,
  "sha256": null,
  "status": "active",
  "metadata": {},
  "createdAt": "2026-05-06T00:00:00Z"
}
```

补充说明：

- `bucket` 不由前端传入，服务端固定写入 `private`
- `objectKey` 必须来自当前用户当前会话的上传签名路径

## 11. 登记会话附件抽取状态 / 文本

- 方法：`PUT`
- 路径：`/api/app/conversation-files/{fileId}/extraction`
- 鉴权：有效 App 用户 JWT

请求体：

```json
{
  "status": "extracted",
  "text": "附件抽取出的正文内容，可被 Agent 上下文读取",
  "contentKind": "plain_text",
  "source": "trusted_api",
  "parser": "external_adapter",
  "language": "zh-CN",
  "failureReason": null,
  "metadata": {
    "pageCount": 3
  }
}
```

字段说明：

| 字段 | 类型 | 必填 | 说明 |
| --- | --- | --- | --- |
| `status` | `pending \| extracted \| failed \| unsupported` | 否 | 不传时兼容旧调用，按 `extracted` 处理；旧值 `ready/success` 会归一为 `extracted` |
| `text` | `string` | `status=extracted` 时必填 | 可信抽取正文，trim 后不能为空，最多 100000 字符 |
| `contentKind` | `string` | 否 | 默认 `plain_text`，用于标记正文类型 |
| `source` | `string` | 否 | 默认 `trusted_api`；人工登记可传 `manual` |
| `parser` | `string` | 否 | 默认 `external_adapter`；`source=manual` 时默认 `manual` |
| `language` | `string` | 否 | 语言标记，例如 `zh-CN` |
| `failureReason` | `string` | 否 | `failed/unsupported` 时记录失败或不支持原因 |
| `metadata` | `object` | 否 | 解析器补充信息，必须是 JSON object |

成功响应 `data`：`ConversationFileResponse`，其中 `metadata` 会包含：

```json
{
  "extractionStatus": "extracted",
  "extractedText": "附件抽取出的正文内容，可被 Agent 上下文读取",
  "textContent": "附件抽取出的正文内容，可被 Agent 上下文读取",
  "extraction": {
    "status": "extracted",
    "contentKind": "plain_text",
    "source": "trusted_api",
    "parser": "external_adapter",
    "language": "zh-CN",
    "text": "附件抽取出的正文内容，可被 Agent 上下文读取",
    "textChars": 23,
    "updatedAt": "2026-05-10T00:00:00Z",
    "metadata": {
      "pageCount": 3
    }
  }
}
```

失败或不支持状态示例：

```json
{
  "status": "failed",
  "source": "adapter",
  "parser": "pdf",
  "failureReason": "encrypted pdf",
  "metadata": {
    "attempt": 1
  }
}
```

该状态会清除旧的 `metadata.extractedText` / `metadata.textContent`，避免 Agent 继续读取过期正文。

补充说明：

- 本接口只登记可信抽取状态和文本，不负责 OCR、PDF 解析、图片识别或 object storage 文件读取
- 新上传会话附件会初始化为 `metadata.extractionStatus=pending`
- 只有 `status=extracted` 的附件正文会被工作区 `context-pack` 读取
- `status=pending/failed/unsupported` 不会进入 Agent 可读上下文
- `metadata` 必须是 JSON object
- 写入后的 `metadata.extractedText` 会被工作区 `context-pack` 读取，并进入 Agent 问答上下文

## 12. 获取会话附件下载签名

- 方法：`GET`
- 路径：`/api/app/conversation-files/{fileId}/access`
- 鉴权：有效 App 用户 JWT

成功响应 `data`：

```json
{
  "file": {
    "id": "file-uuid",
    "conversationId": "conversation-uuid",
    "messageId": null,
    "bucket": "private",
    "objectKey": "chat/userScope/conversationId/file.png",
    "fileName": "file.png",
    "mimeType": "image/png",
    "fileSize": 12345,
    "sha256": null,
    "status": "active",
    "metadata": {},
    "createdAt": "2026-05-06T00:00:00Z"
  },
  "method": "GET",
  "downloadUrl": "http://s3.example/private/chat/...",
  "expiresSecs": 600
}
```

## 12. 查询我的商品列表

- 方法：`GET`
- 路径：`/api/app/products`
- 鉴权：有效 App 用户 JWT

查询参数：

| 参数 | 类型 | 必填 | 说明 |
| --- | --- | --- | --- |
| `status` | `draft \| listing \| active \| archived` | 否 | 不传时默认查询 `active` |
| `keyword` | `string` | 否 | 匹配商品名称、平台、平台商品 ID、类目、品牌 |

成功响应 `data`：`ProductResponse[]`。

示例：

```json
[
  {
    "id": "product-uuid",
    "userId": "user-uuid",
    "name": "五一咖啡组合",
    "platform": "douyin",
    "externalId": "123",
    "category": "coffee",
    "brand": "Atlas",
    "status": "active",
    "description": "节日活动主推组合",
    "price": "99.00",
    "currency": "CNY",
    "metadata": {},
    "createdAt": "2026-05-07T00:00:00Z",
    "updatedAt": "2026-05-07T00:00:00Z"
  }
]
```

## 13. 创建商品

- 方法：`POST`
- 路径：`/api/app/products`
- 鉴权：有效 App 用户 JWT

请求体：

```json
{
  "name": "五一咖啡组合",
  "platform": "douyin",
  "externalId": "123",
  "category": "coffee",
  "brand": "Atlas",
  "status": "draft",
  "description": "节日活动主推组合",
  "price": "99.00",
  "currency": "CNY",
  "metadata": {}
}
```

成功响应 `data`：`ProductResponse`。

补充说明：

- `name` 必填，最长 255 个字符
- `status` 可选：`draft`、`listing`、`active`、`archived`；不传时默认 `active`，用于兼容已有商品页
- `price` 当前按字符串保存，避免前端浮点精度问题；后续如需财务级金额计算，可单独迁移为金额规范
- `metadata` 不传时默认 `{}`，用于承接旧产品里的扩展业务字段

## 14. 查询商品详情

- 方法：`GET`
- 路径：`/api/app/products/{productId}`
- 鉴权：有效 App 用户 JWT

成功响应 `data`：

```json
{
  "product": {
    "id": "product-uuid",
    "userId": "user-uuid",
    "name": "五一咖啡组合",
    "platform": "douyin",
    "externalId": "123",
    "category": "coffee",
    "brand": "Atlas",
    "status": "active",
    "description": "节日活动主推组合",
    "price": "99.00",
    "currency": "CNY",
    "metadata": {},
    "createdAt": "2026-05-07T00:00:00Z",
    "updatedAt": "2026-05-07T00:00:00Z"
  }
}
```

## 15. 更新商品

- 方法：`PATCH`
- 路径：`/api/app/products/{productId}`
- 鉴权：有效 App 用户 JWT

请求体：

```json
{
  "name": "五一咖啡组合升级版",
  "platform": "douyin",
  "externalId": "123",
  "category": "coffee",
  "brand": "Atlas",
  "description": "更新后的商品说明",
  "price": "109.00",
  "currency": "CNY",
  "metadata": {}
}
```

成功响应 `data`：`ProductResponse`。

补充说明：

- 只能更新当前用户自己的商品
- 省略字段表示保持原值
- 字符串字段传空字符串时会按清空处理
- 已归档商品不能更新

## 16. 归档商品

- 方法：`DELETE`
- 路径：`/api/app/products/{productId}`
- 鉴权：有效 App 用户 JWT

成功响应 `data`：`null`。

补充说明：

- 当前实现为软删除，设置 `status = archived`
- 默认商品列表不会返回已归档商品

## 17. 查询商品看板指标

- 方法：`GET`
- 路径：`/api/app/products/dashboard`
- 鉴权：有效 App 用户 JWT

成功响应 `data`：

```json
{
  "total": 12,
  "draft": 1,
  "listing": 1,
  "active": 10,
  "archived": 2
}
```

## 17.1 查询商品生命周期选项

- 方法：`GET`
- 路径：`/api/app/products/lifecycle/options`
- 鉴权：有效 App 用户 JWT

成功响应 `data`：

```json
{
  "statuses": [
    {
      "status": "draft",
      "label": "选品中",
      "description": "商品仍在选品、定位或资料准备阶段。",
      "terminal": false
    },
    {
      "status": "listing",
      "label": "上架中",
      "description": "商品正在准备上架资料、素材、价格和渠道配置。",
      "terminal": false
    },
    {
      "status": "active",
      "label": "运营中",
      "description": "商品已进入日常运营、投放、内容和复盘阶段。",
      "terminal": false
    },
    {
      "status": "archived",
      "label": "已归档",
      "description": "商品已归档，不再参与新的运营动作。",
      "terminal": true
    }
  ],
  "transitions": [
    {
      "from": "draft",
      "to": "listing",
      "label": "进入上架准备",
      "requiresReason": false
    },
    {
      "from": "listing",
      "to": "active",
      "label": "进入运营",
      "requiresReason": false
    },
    {
      "from": "listing",
      "to": "draft",
      "label": "退回选品",
      "requiresReason": true
    },
    {
      "from": "active",
      "to": "archived",
      "label": "归档商品",
      "requiresReason": true
    }
  ],
  "agentContext": {
    "type": "product_lifecycle_options",
    "statuses": ["draft", "listing", "active", "archived"],
    "terminalStatuses": ["archived"]
  }
}
```

补充说明：

- 当前用户必须存在于 `app_users`，且状态为 `enabled`
- 前端商品生命周期展示和可操作按钮应以本接口返回为准

## 17.2 查询商品生命周期

- 方法：`GET`
- 路径：`/api/app/products/{productId}/lifecycle`
- 鉴权：有效 App 用户 JWT

成功响应 `data`：

```json
{
  "product": {
    "id": "product-uuid",
    "userId": "user-uuid",
    "name": "五一咖啡组合",
    "platform": "douyin",
    "externalId": "123",
    "category": "coffee",
    "brand": "Atlas",
    "status": "listing",
    "description": "节日活动主推组合",
    "price": "99.00",
    "currency": "CNY",
    "metadata": {},
    "createdAt": "2026-05-07T00:00:00Z",
    "updatedAt": "2026-05-07T00:00:00Z"
  },
  "currentStatus": "listing",
  "allowedNextStatuses": ["active", "draft"],
  "transitions": [
    {
      "from": "listing",
      "to": "active",
      "label": "进入运营",
      "requiresReason": false
    },
    {
      "from": "listing",
      "to": "draft",
      "label": "退回选品",
      "requiresReason": true
    }
  ],
  "warnings": [],
  "agentContext": {
    "type": "product_lifecycle",
    "productId": "product-uuid",
    "currentStatus": "listing",
    "allowedNextStatuses": ["active", "draft"],
    "terminal": false
  }
}
```

补充说明：

- 只能查询当前用户自己的商品
- 已归档商品可查询生命周期，但 `allowedNextStatuses` 为空

## 17.3 流转商品生命周期

- 方法：`PATCH`
- 路径：`/api/app/products/{productId}/lifecycle`
- 鉴权：有效 App 用户 JWT

请求体：

```json
{
  "targetStatus": "active",
  "reason": "上架资料和首批素材已完成",
  "metadata": {
    "source": "operator"
  }
}
```

成功响应 `data`：`ProductLifecycleResponse`，结构同“查询商品生命周期”。

允许流转：

- `draft -> listing`
- `listing -> active`
- `listing -> draft`
- `active -> archived`

补充说明：

- 非法目标状态或非法流转返回 `400`，错误码为 `PRODUCT_REQUEST_INVALID`
- `archived` 是终态，不能再流转
- 流转成功后，后端会在 `product.metadata.lifecycle` 写入 `currentStatus`、`lastTransition` 和最近流转 `history`
- 本接口只负责新产品内部生命周期状态，不会自动执行平台上架、下架或发布动作

## 18. 查询商品知识列表

- 方法：`GET`
- 路径：`/api/app/products/{productId}/knowledge`
- 鉴权：有效 App 用户 JWT

查询参数：

| 参数 | 类型 | 必填 | 说明 |
| --- | --- | --- | --- |
| `status` | `active \| archived` | 否 | 不传时默认查询 `active` |
| `contentType` | `string` | 否 | 例如 `text`、`spec`、`selling_point`、`competitive`、`faq` |
| `keyword` | `string` | 否 | 匹配标题、内容、来源类型、来源 ID |

成功响应 `data`：`ProductKnowledgeResponse[]`。

示例：

```json
[
  {
    "id": "knowledge-uuid",
    "userId": "user-uuid",
    "productId": "product-uuid",
    "title": "核心卖点",
    "contentType": "selling_point",
    "content": "轻量、防晒、通勤户外双场景。",
    "sourceType": "manual",
    "sourceId": null,
    "confidence": 0.8,
    "status": "active",
    "metadata": {},
    "createdAt": "2026-05-07T00:00:00Z",
    "updatedAt": "2026-05-07T00:00:00Z"
  }
]
```

补充说明：

- 只能查询当前用户自己的商品知识
- 商品已归档时返回商品不存在
- 默认按 `confidence`、`updatedAt` 倒序返回

## 19. 创建商品知识

- 方法：`POST`
- 路径：`/api/app/products/{productId}/knowledge`
- 鉴权：有效 App 用户 JWT

请求体：

```json
{
  "title": "核心卖点",
  "contentType": "selling_point",
  "content": "轻量、防晒、通勤户外双场景。",
  "sourceType": "manual",
  "sourceId": null,
  "confidence": 0.8,
  "metadata": {}
}
```

成功响应 `data`：`ProductKnowledgeResponse`。

补充说明：

- `content` 必填，最长 20000 个字符
- `contentType` 不传时默认 `text`
- `sourceType` 不传时默认 `manual`
- `confidence` 不传时默认 `0.8`，有效范围为 `0..1`
- `metadata` 不传时默认 `{}`

## 20. 更新商品知识

- 方法：`PATCH`
- 路径：`/api/app/products/{productId}/knowledge/{knowledgeId}`
- 鉴权：有效 App 用户 JWT

请求体：

```json
{
  "title": "更新后的卖点",
  "contentType": "selling_point",
  "content": "更新后的知识内容",
  "sourceType": "manual",
  "sourceId": null,
  "confidence": 0.9,
  "metadata": {}
}
```

成功响应 `data`：`ProductKnowledgeResponse`。

补充说明：

- 只能更新当前用户自己商品下的知识条目
- 省略字段表示保持原值
- `content` 传空字符串会返回请求非法
- 已归档商品或已归档知识不能更新

## 21. 归档商品知识

- 方法：`DELETE`
- 路径：`/api/app/products/{productId}/knowledge/{knowledgeId}`
- 鉴权：有效 App 用户 JWT

成功响应 `data`：`null`。

补充说明：

- 当前实现为软删除，设置 `status = archived`
- 默认商品知识列表不会返回已归档知识

## 22. 获取商品资产上传签名

- 方法：`POST`
- 路径：`/api/app/products/{productId}/assets/presign-upload`
- 鉴权：有效 App 用户 JWT

请求体：

```json
{
  "extension": "png",
  "contentType": "image/png"
}
```

成功响应 `data`：

```json
{
  "method": "PUT",
  "uploadUrl": "http://s3.example/private/products/user/product/assets/file.png",
  "bucket": "private",
  "objectKey": "products/user/product/assets/file.png",
  "contentType": "image/png",
  "expiresSecs": 600
}
```

补充说明：

- 商品资产固定使用 private bucket
- 前端先使用 `uploadUrl` 上传对象，再调用资产登记接口写入业务表
- `objectKey` 必须使用服务端签发的当前用户当前商品路径

## 23. 查询商品资产列表

- 方法：`GET`
- 路径：`/api/app/products/{productId}/assets`
- 鉴权：有效 App 用户 JWT

查询参数：

| 参数 | 类型 | 必填 | 说明 |
| --- | --- | --- | --- |
| `status` | `active \| archived` | 否 | 不传时默认查询 `active` |
| `assetType` | `docs \| images \| videos \| other` | 否 | 资产类型 |
| `keyword` | `string` | 否 | 匹配文件名、对象 key、描述 |

成功响应 `data`：`ProductAssetResponse[]`。

## 24. 登记商品资产

- 方法：`POST`
- 路径：`/api/app/products/{productId}/assets`
- 鉴权：有效 App 用户 JWT

请求体：

```json
{
  "bucket": "private",
  "objectKey": "products/user/product/assets/file.png",
  "assetType": "images",
  "fileName": "main.png",
  "mimeType": "image/png",
  "fileSize": 12345,
  "description": "主图素材",
  "metadata": {}
}
```

成功响应 `data`：`ProductAssetResponse`。

补充说明：

- 服务端会校验 `objectKey` 是否位于当前用户当前商品路径下
- `assetType` 不传时默认 `docs`
- 只登记对象元数据，不直接接收文件二进制

## 25. 获取商品资产下载签名

- 方法：`GET`
- 路径：`/api/app/products/{productId}/assets/{assetId}/access`
- 鉴权：有效 App 用户 JWT

成功响应 `data`：

```json
{
  "asset": {
    "id": "asset-uuid",
    "productId": "product-uuid",
    "assetType": "images",
    "fileName": "main.png",
    "status": "active"
  },
  "method": "GET",
  "downloadUrl": "http://s3.example/private/products/...",
  "expiresSecs": 600
}
```

## 26. 更新商品资产

- 方法：`PATCH`
- 路径：`/api/app/products/{productId}/assets/{assetId}`
- 鉴权：有效 App 用户 JWT

请求体：

```json
{
  "assetType": "docs",
  "fileName": "卖点说明.md",
  "mimeType": "text/markdown",
  "fileSize": 2048,
  "description": "更新后的说明",
  "metadata": {}
}
```

成功响应 `data`：`ProductAssetResponse`。

## 27. 归档商品资产

- 方法：`DELETE`
- 路径：`/api/app/products/{productId}/assets/{assetId}`
- 鉴权：有效 App 用户 JWT

成功响应 `data`：`null`。

补充说明：

- 当前实现为软删除，设置 `status = archived`
- 不在本接口直接删除对象存储里的物理文件

## 28. 查询商品素材列表

- 方法：`GET`
- 路径：`/api/app/products/{productId}/materials`
- 鉴权：有效 App 用户 JWT

查询参数：

| 参数 | 类型 | 必填 | 说明 |
| --- | --- | --- | --- |
| `status` | `active \| archived` | 否 | 不传时默认查询 `active` |
| `materialType` | `string` | 否 | 例如 `main_image`、`detail_page`、`copy`、`video_script` |
| `keyword` | `string` | 否 | 匹配标题和内容 |

成功响应 `data`：`ProductMaterialResponse[]`。

## 29. 创建商品素材

- 方法：`POST`
- 路径：`/api/app/products/{productId}/materials`
- 鉴权：有效 App 用户 JWT

请求体：

```json
{
  "materialType": "copy",
  "title": "五一活动文案",
  "content": "限时活动文案内容",
  "metadata": {}
}
```

成功响应 `data`：`ProductMaterialResponse`。

补充说明：

- `materialType` 必填
- `version` 由服务端按同商品同素材类型自动递增

## 30. 查询商品素材详情

- 方法：`GET`
- 路径：`/api/app/products/{productId}/materials/{materialId}`
- 鉴权：有效 App 用户 JWT

成功响应 `data`：`ProductMaterialResponse`。

## 31. 更新商品素材

- 方法：`PATCH`
- 路径：`/api/app/products/{productId}/materials/{materialId}`
- 鉴权：有效 App 用户 JWT

请求体：

```json
{
  "title": "更新后的活动文案",
  "content": "更新后的内容",
  "metadata": {}
}
```

成功响应 `data`：`ProductMaterialResponse`。

补充说明：

- 更新 `content` 时 `version` 自动加 1
- 仅更新 `title` 或 `metadata` 时版本号保持不变

## 32. 归档商品素材

- 方法：`DELETE`
- 路径：`/api/app/products/{productId}/materials/{materialId}`
- 鉴权：有效 App 用户 JWT

成功响应 `data`：`null`。

## 33. 查询同类型素材版本

- 方法：`GET`
- 路径：`/api/app/products/{productId}/materials/{materialId}/versions`
- 鉴权：有效 App 用户 JWT

成功响应 `data`：`ProductMaterialResponse[]`。

补充说明：

- 返回当前素材所属 `materialType` 在该商品下的所有版本，按 `version` 倒序

## 34. 查询商品竞品列表

- 方法：`GET`
- 路径：`/api/app/products/{productId}/competitors`
- 鉴权：有效 App 用户 JWT

查询参数：

| 参数 | 类型 | 必填 | 说明 |
| --- | --- | --- | --- |
| `status` | `active \| archived` | 否 | 不传时默认查询 `active` |
| `platform` | `string` | 否 | 竞品所在平台 |
| `keyword` | `string` | 否 | 匹配竞品名称、平台、URL、月销量、备注 |

成功响应 `data`：`ProductCompetitorResponse[]`。

示例：

```json
[
  {
    "id": "competitor-uuid",
    "userId": "user-uuid",
    "productId": "product-uuid",
    "name": "同类咖啡组合",
    "platform": "douyin",
    "url": "https://example.com/item",
    "price": 89.9,
    "rating": 4.8,
    "monthlySales": "1000+",
    "notes": "主打低价组合",
    "searchResults": {},
    "status": "active",
    "metadata": {},
    "lastSearchedAt": null,
    "createdAt": "2026-05-07T00:00:00Z",
    "updatedAt": "2026-05-07T00:00:00Z"
  }
]
```

## 35. 创建商品竞品

- 方法：`POST`
- 路径：`/api/app/products/{productId}/competitors`
- 鉴权：有效 App 用户 JWT

请求体：

```json
{
  "name": "同类咖啡组合",
  "platform": "douyin",
  "url": "https://example.com/item",
  "price": 89.9,
  "rating": 4.8,
  "monthlySales": "1000+",
  "notes": "主打低价组合",
  "searchResults": {},
  "metadata": {},
  "lastSearchedAt": null
}
```

成功响应 `data`：`ProductCompetitorResponse`。

补充说明：

- `name` 必填，最长 255 个字符
- `searchResults` 用于承接旧产品自动搜索返回摘要；本阶段不触发自动搜索
- `metadata` 不传时默认 `{}`

## 36. 查询商品竞品详情

- 方法：`GET`
- 路径：`/api/app/products/{productId}/competitors/{competitorId}`
- 鉴权：有效 App 用户 JWT

成功响应 `data`：`ProductCompetitorResponse`。

## 37. 更新商品竞品

- 方法：`PATCH`
- 路径：`/api/app/products/{productId}/competitors/{competitorId}`
- 鉴权：有效 App 用户 JWT

请求体：

```json
{
  "name": "更新后的竞品名称",
  "platform": "douyin",
  "url": "https://example.com/item",
  "price": 79.9,
  "rating": 4.7,
  "monthlySales": "2000+",
  "notes": "价格下降",
  "searchResults": {},
  "metadata": {},
  "lastSearchedAt": "2026-05-07T00:00:00Z"
}
```

成功响应 `data`：`ProductCompetitorResponse`。

补充说明：

- 只能更新当前用户自己商品下的竞品
- 省略字段表示保持原值
- 已归档商品或已归档竞品不能更新

## 38. 归档商品竞品

- 方法：`DELETE`
- 路径：`/api/app/products/{productId}/competitors/{competitorId}`
- 鉴权：有效 App 用户 JWT

成功响应 `data`：`null`。

补充说明：

- 当前实现为软删除，设置 `status = archived`
- 默认竞品列表不会返回已归档竞品

## 39. 查询商品竞品变化历史

- 方法：`GET`
- 路径：`/api/app/products/{productId}/competitors/changes`
- 鉴权：有效 App 用户 JWT

查询参数：

| 参数 | 类型 | 必填 | 说明 |
| --- | --- | --- | --- |
| `competitorId` | `uuid` | 否 | 只看某个竞品 |
| `changeType` | `string` | 否 | 例如 `price_drop`、`price_rise`、`rating_change`、`manual_note` |
| `limit` | `number` | 否 | 默认 100，最大 500 |

成功响应 `data`：`CompetitorChangeResponse[]`。

示例：

```json
[
  {
    "id": "change-uuid",
    "userId": "user-uuid",
    "productId": "product-uuid",
    "competitorId": "competitor-uuid",
    "competitorName": "同类咖啡组合",
    "competitorPlatform": "douyin",
    "changeType": "price_drop",
    "oldValue": "89.9",
    "newValue": "79.9",
    "summary": "竞品价格下降",
    "sourceType": "manual",
    "metadata": {},
    "detectedAt": "2026-05-07T00:00:00Z",
    "createdAt": "2026-05-07T00:00:00Z"
  }
]
```

## 40. 查询商品竞品分析

- 方法：`GET`
- 路径：`/api/app/products/{productId}/competitors/analysis`
- 鉴权：有效 App 用户 JWT

查询参数：

| 参数 | 类型 | 必填 | 说明 |
| --- | --- | --- | --- |
| `status` | `active \| archived` | 否 | 不传时默认分析 `active` 竞品 |
| `platform` | `string` | 否 | 只分析某个平台的竞品 |
| `changeLimit` | `number` | 否 | 近期变化数量，默认 100，最大 500 |

成功响应 `data`：`ProductCompetitorAnalysisResponse`。

示例：

```json
{
  "product": {
    "id": "product-uuid",
    "name": "咖啡组合",
    "platform": "douyin",
    "category": "coffee",
    "brand": "brand",
    "price": "99",
    "numericPrice": 99,
    "currency": "CNY"
  },
  "readiness": "ready",
  "summary": "Competitor analysis is ready: 3 competitor(s), 3 priced, 2 rated, 1 recent change(s), product price is +10.0% vs competitor avg (premium).",
  "counts": {
    "competitors": 3,
    "pricedCompetitors": 3,
    "ratedCompetitors": 2,
    "monthlySalesCompetitors": 2,
    "recentChanges": 1
  },
  "price": {
    "productPrice": 99,
    "minCompetitorPrice": 79,
    "maxCompetitorPrice": 109,
    "avgCompetitorPrice": 90,
    "pricePosition": "premium",
    "gapToAvg": 9,
    "gapPercentToAvg": 10,
    "lowerPricedCompetitors": 2,
    "higherPricedCompetitors": 1
  },
  "rating": {
    "minRating": 4.5,
    "maxRating": 4.8,
    "avgRating": 4.65,
    "ratedCompetitors": 2
  },
  "platforms": [
    {
      "platform": "douyin",
      "competitors": 2,
      "avgPrice": 89,
      "avgRating": 4.6
    }
  ],
  "topCompetitors": [],
  "recentChanges": [],
  "refs": [],
  "signals": ["price_position:premium", "recent_price_change"],
  "warnings": [],
  "agentContext": {}
}
```

补充说明：

- 本接口只读取当前用户当前商品下的竞品和变化记录
- 本接口不调用 LLM，不触发外部搜索，不自动创建工作区 artifact
- `readiness` 可能为 `ready`、`partial`、`empty`
- `agentContext` 可供后续 Agent 问答、竞品分析 skill、定价策略 skill 复用

## 41. 创建竞品变化记录

- 方法：`POST`
- 路径：`/api/app/products/{productId}/competitors/{competitorId}/changes`
- 鉴权：有效 App 用户 JWT

请求体：

```json
{
  "changeType": "price_drop",
  "oldValue": "89.9",
  "newValue": "79.9",
  "summary": "竞品价格下降",
  "sourceType": "manual",
  "metadata": {},
  "detectedAt": "2026-05-07T00:00:00Z"
}
```

成功响应 `data`：`CompetitorChangeResponse`。

补充说明：

- 本阶段提供手动记录能力，不调用 Agent 或搜索技能
- `sourceType` 不传时默认 `manual`
- `detectedAt` 不传时由数据库使用当前时间

## 42. 查询营销活动列表

- 方法：`GET`
- 路径：`/api/app/campaigns`
- 鉴权：有效 App 用户 JWT

查询参数：

| 参数 | 类型 | 必填 | 说明 |
| --- | --- | --- | --- |
| `status` | `draft \| active \| paused \| ended \| archived` | 否 | 活动状态 |
| `productId` | `uuid` | 否 | 只查询某个商品关联的活动 |
| `campaignType` | `string` | 否 | 例如 `promotion`、`launch`、`seasonal` |
| `keyword` | `string` | 否 | 匹配活动名称、类型、描述 |
| `page` | `number` | 否 | 默认 1 |
| `pageSize` | `number` | 否 | 默认 20，最大 100 |

成功响应 `data`：

```json
{
  "campaigns": [
    {
      "id": "campaign-uuid",
      "userId": "user-uuid",
      "productId": "product-uuid",
      "name": "新品上市活动",
      "campaignType": "launch",
      "description": "新品首发推广",
      "budget": 5000,
      "startDate": "2026-05-01",
      "endDate": "2026-05-31",
      "status": "draft",
      "metadata": {},
      "output": {},
      "actionMetadata": {},
      "createdAt": "2026-05-07T00:00:00Z",
      "updatedAt": "2026-05-07T00:00:00Z"
    }
  ],
  "total": 1,
  "page": 1,
  "pageSize": 20
}
```

## 43. 创建营销活动

- 方法：`POST`
- 路径：`/api/app/campaigns`
- 鉴权：有效 App 用户 JWT

请求体：

```json
{
  "productId": "product-uuid",
  "name": "新品上市活动",
  "campaignType": "launch",
  "description": "新品首发推广",
  "budget": 5000,
  "startDate": "2026-05-01",
  "endDate": "2026-05-31",
  "status": "draft",
  "metadata": {},
  "output": {},
  "actionMetadata": {}
}
```

成功响应 `data`：`CampaignResponse`。

补充说明：

- `name` 必填，最长 255 个字符
- `productId` 可为空；不为空时必须属于当前用户且商品未归档
- `status` 不传时默认 `draft`
- `metadata`、`output`、`actionMetadata` 不传时默认 `{}`
- 本阶段只存储活动记录、生成结果和建议动作，不触发 Agent 自动生成或平台执行

## 43.1 创建营销活动草稿

- 方法：`POST`
- 路径：`/api/app/campaigns/draft`
- 鉴权：有效 App 用户 JWT

请求体：

```json
{
  "productId": "product-uuid",
  "workspaceId": "workspace-uuid",
  "sourceArtifactId": "artifact-uuid",
  "name": "五一新品冷启动草稿",
  "campaignType": "launch",
  "description": "围绕新品首发做冷启动投放",
  "budget": 3000,
  "startDate": "2026-05-01",
  "endDate": "2026-05-07",
  "goal": "验证首批人群和素材方向",
  "targetAudience": "18-30 岁咖啡兴趣人群",
  "channels": ["douyin", "xiaohongshu"],
  "strategy": "先小预算测素材，再放大高转化计划",
  "sourceType": "agent_output",
  "sourceSummary": "Agent 生成的冷启动方案",
  "draft": {
    "tasks": []
  },
  "metadata": {},
  "actionMetadata": {}
}
```

成功响应 `data`：`CampaignDraftResponse`。

```json
{
  "campaign": {
    "id": "campaign-uuid",
    "productId": "product-uuid",
    "name": "五一新品冷启动草稿",
    "campaignType": "launch",
    "description": "围绕新品首发做冷启动投放",
    "budget": 3000,
    "startDate": "2026-05-01",
    "endDate": "2026-05-07",
    "status": "draft",
    "metadata": {},
    "output": {},
    "actionMetadata": {}
  },
  "source": {
    "sourceType": "agent_output",
    "productId": "product-uuid",
    "workspaceId": "workspace-uuid",
    "sourceArtifactId": "artifact-uuid",
    "sourceSummary": "Agent 生成的冷启动方案",
    "refKeys": [
      "product:product-uuid",
      "workspace:workspace-uuid",
      "workspace_artifact:artifact-uuid"
    ]
  },
  "warnings": [],
  "agentContext": {}
}
```

补充说明：

- 本接口总是创建 `draft` 状态活动，不接受前端传入其它状态
- 如果传入 `productId`，商品必须属于当前用户且未归档
- 如果传入 `workspaceId`，工作区必须属于当前用户且未归档
- 如果传入 `sourceArtifactId`，必须同时传入 `workspaceId`，且交付物必须属于该工作区
- 本接口不调用 LLM，不发布平台活动，不绕过后续审批
- `metadata.source`、`metadata.planning`、`output`、`actionMetadata` 会保存草稿来源和后续执行提示

## 44. 查询营销活动详情

- 方法：`GET`
- 路径：`/api/app/campaigns/{campaignId}`
- 鉴权：有效 App 用户 JWT

成功响应 `data`：`CampaignResponse`。

## 45. 更新营销活动

- 方法：`PATCH`
- 路径：`/api/app/campaigns/{campaignId}`
- 鉴权：有效 App 用户 JWT

请求体：

```json
{
  "productId": "product-uuid",
  "name": "更新后的活动名称",
  "campaignType": "promotion",
  "description": "更新后的活动描述",
  "budget": 8000,
  "startDate": "2026-06-01",
  "endDate": "2026-06-30",
  "status": "active",
  "metadata": {},
  "output": {},
  "actionMetadata": {}
}
```

成功响应 `data`：`CampaignResponse`。

补充说明：

- 只能更新当前用户自己的活动
- 省略字段表示保持原值
- 如更新 `productId`，新商品必须属于当前用户且未归档

## 46. 归档营销活动

- 方法：`DELETE`
- 路径：`/api/app/campaigns/{campaignId}`
- 鉴权：有效 App 用户 JWT

成功响应 `data`：`null`。

补充说明：

- 当前实现为软删除，设置 `status = archived`

## 46. 查询工作区列表

- 方法：`GET`
- 路径：`/api/app/workspaces`
- 鉴权：有效 App 用户 JWT

查询参数：

| 参数 | 类型 | 必填 | 说明 |
| --- | --- | --- | --- |
| `status` | `active \| archived` | 否 | 工作区状态 |
| `productId` | `uuid` | 否 | 只查询某个商品关联的工作区 |
| `campaignId` | `uuid` | 否 | 只查询某个活动关联的工作区 |
| `workspaceType` | `string` | 否 | 工作区类型，例如 `general`、`campaign` |
| `keyword` | `string` | 否 | 匹配标题或类型 |
| `page` | `number` | 否 | 默认 1 |
| `pageSize` | `number` | 否 | 默认 20，最大 100 |

成功响应 `data`：

```json
{
  "workspaces": [
    {
      "id": "workspace-uuid",
      "userId": "user-uuid",
      "productId": "product-uuid",
      "campaignId": "campaign-uuid",
      "title": "新品上市执行工作区",
      "workspaceType": "campaign",
      "status": "active",
      "config": {},
      "createdAt": "2026-05-07T00:00:00Z",
      "updatedAt": "2026-05-07T00:00:00Z"
    }
  ],
  "total": 1,
  "page": 1,
  "pageSize": 20
}
```

## 47. 创建工作区

- 方法：`POST`
- 路径：`/api/app/workspaces`
- 鉴权：有效 App 用户 JWT

请求体：

```json
{
  "productId": "product-uuid",
  "campaignId": "campaign-uuid",
  "title": "新品上市执行工作区",
  "workspaceType": "campaign",
  "config": {}
}
```

成功响应 `data`：`WorkspaceResponse`。

补充说明：

- `title` 必填，最长 255 个字符
- `productId` 可为空；不为空时必须属于当前用户且商品未归档
- `campaignId` 可为空；不为空时必须属于当前用户
- `workspaceType` 不传时默认 `general`
- `config` 不传时默认 `{}`

## 48. 查询工作区详情

- 方法：`GET`
- 路径：`/api/app/workspaces/{workspaceId}`
- 鉴权：有效 App 用户 JWT

成功响应 `data`：`WorkspaceResponse`。

## 49. 更新工作区

- 方法：`PATCH`
- 路径：`/api/app/workspaces/{workspaceId}`
- 鉴权：有效 App 用户 JWT

请求体：

```json
{
  "productId": "product-uuid",
  "campaignId": "campaign-uuid",
  "title": "更新后的工作区标题",
  "workspaceType": "campaign",
  "status": "active",
  "config": {}
}
```

成功响应 `data`：`WorkspaceResponse`。

补充说明：

- 只能更新当前用户自己的工作区
- 省略字段表示保持原值
- 如更新 `productId` 或 `campaignId`，目标资源必须属于当前用户

## 50. 归档工作区

- 方法：`DELETE`
- 路径：`/api/app/workspaces/{workspaceId}`
- 鉴权：有效 App 用户 JWT

成功响应 `data`：`null`。

补充说明：

- 当前实现为软删除，设置 `status = archived`

## 51. 查询工作区任务列表

- 方法：`GET`
- 路径：`/api/app/workspaces/{workspaceId}/tasks`
- 鉴权：有效 App 用户 JWT

查询参数：

| 参数 | 类型 | 必填 | 说明 |
| --- | --- | --- | --- |
| `status` | `pending \| in_progress \| done \| blocked \| cancelled \| archived` | 否 | 任务状态 |
| `ownerRole` | `string` | 否 | 负责人角色 |
| `keyword` | `string` | 否 | 匹配标题或描述 |
| `page` | `number` | 否 | 默认 1 |
| `pageSize` | `number` | 否 | 默认 20，最大 100 |

成功响应 `data`：

```json
{
  "tasks": [
    {
      "id": "task-uuid",
      "workspaceId": "workspace-uuid",
      "userId": "user-uuid",
      "title": "整理投放素材",
      "description": "准备首轮投放素材",
      "ownerRole": "planner",
      "status": "pending",
      "priority": 3,
      "dependsOn": [],
      "acceptanceCriteria": "素材通过人工确认",
      "result": null,
      "metadata": {},
      "createdAt": "2026-05-07T00:00:00Z",
      "updatedAt": "2026-05-07T00:00:00Z"
    }
  ],
  "total": 1,
  "page": 1,
  "pageSize": 20
}
```

## 52. 创建工作区任务

- 方法：`POST`
- 路径：`/api/app/workspaces/{workspaceId}/tasks`
- 鉴权：有效 App 用户 JWT

请求体：

```json
{
  "title": "整理投放素材",
  "description": "准备首轮投放素材",
  "ownerRole": "planner",
  "status": "pending",
  "priority": 3,
  "dependsOn": [],
  "acceptanceCriteria": "素材通过人工确认",
  "result": null,
  "metadata": {}
}
```

成功响应 `data`：`WorkspaceTaskResponse`。

补充说明：

- `title` 必填
- `status` 不传时默认 `pending`
- `priority` 范围为 0 到 10，不传时默认 0
- `dependsOn` 不传时默认 `[]`
- 本阶段只存储任务记录，不触发 Agent 执行

## 53. 查询工作区任务详情

- 方法：`GET`
- 路径：`/api/app/workspaces/{workspaceId}/tasks/{taskId}`
- 鉴权：有效 App 用户 JWT

成功响应 `data`：`WorkspaceTaskResponse`。

## 54. 更新工作区任务

- 方法：`PATCH`
- 路径：`/api/app/workspaces/{workspaceId}/tasks/{taskId}`
- 鉴权：有效 App 用户 JWT

请求体字段同创建任务，所有字段均可省略。

成功响应 `data`：`WorkspaceTaskResponse`。

## 55. 归档工作区任务

- 方法：`DELETE`
- 路径：`/api/app/workspaces/{workspaceId}/tasks/{taskId}`
- 鉴权：有效 App 用户 JWT

成功响应 `data`：`null`。

补充说明：

- 当前实现为软删除，设置 `status = archived`

## 56. 查询工作区运行记录列表

- 方法：`GET`
- 路径：`/api/app/workspaces/{workspaceId}/runs`
- 鉴权：有效 App 用户 JWT

查询参数：

| 参数 | 类型 | 必填 | 说明 |
| --- | --- | --- | --- |
| `status` | `planned \| running \| succeeded \| failed \| cancelled \| archived` | 否 | 运行状态 |
| `source` | `string` | 否 | 来源，例如 `manual`、`agent_output` |
| `keyword` | `string` | 否 | 匹配目标、来源或摘要 |
| `page` | `number` | 否 | 默认 1 |
| `pageSize` | `number` | 否 | 默认 20，最大 100 |

成功响应 `data`：

```json
{
  "runs": [
    {
      "id": "run-uuid",
      "workspaceId": "workspace-uuid",
      "userId": "user-uuid",
      "goal": "完成新品上市投放准备",
      "requirements": [],
      "status": "planned",
      "source": "manual",
      "summary": null,
      "report": {},
      "metadata": {},
      "createdAt": "2026-05-07T00:00:00Z",
      "updatedAt": "2026-05-07T00:00:00Z"
    }
  ],
  "total": 1,
  "page": 1,
  "pageSize": 20
}
```

## 57. 创建工作区运行记录

- 方法：`POST`
- 路径：`/api/app/workspaces/{workspaceId}/runs`
- 鉴权：有效 App 用户 JWT

请求体：

```json
{
  "goal": "完成新品上市投放准备",
  "requirements": [],
  "status": "planned",
  "source": "manual",
  "summary": null,
  "report": {},
  "metadata": {}
}
```

成功响应 `data`：`WorkspaceRunResponse`。

补充说明：

- `goal` 必填
- `status` 不传时默认 `planned`
- `source` 不传时默认 `manual`
- `requirements` 不传时默认 `[]`
- `report` 和 `metadata` 不传时默认 `{}`
- 普通 HTTP 创建接口不启动 Agent TaskGraph，只保存运行/报告记录
- Agent SSE 会自动创建 `source=agent_session` 的运行记录；这类记录由 `/api/sse/agent/session/stream` 绑定或创建，最终输出写入 `report.output`，任务图节点写入 `workspace_run_tasks`，完成时同步创建 `artifactType=final_report` 的工作区交付物

## 58. 查询工作区运行记录详情

- 方法：`GET`
- 路径：`/api/app/workspaces/{workspaceId}/runs/{runId}`
- 鉴权：有效 App 用户 JWT

成功响应 `data`：`WorkspaceRunResponse`。

## 59. 更新工作区运行记录

- 方法：`PATCH`
- 路径：`/api/app/workspaces/{workspaceId}/runs/{runId}`
- 鉴权：有效 App 用户 JWT

请求体字段同创建运行记录，所有字段均可省略。

成功响应 `data`：`WorkspaceRunResponse`。

## 60. 查询运行任务快照列表

- 方法：`GET`
- 路径：`/api/app/workspaces/{workspaceId}/runs/{runId}/tasks`
- 鉴权：有效 App 用户 JWT

查询参数：

| 参数 | 类型 | 必填 | 说明 |
| --- | --- | --- | --- |
| `status` | `pending \| running \| succeeded \| failed \| skipped \| cancelled` | 否 | 运行任务状态 |

成功响应 `data`：`WorkspaceRunTaskResponse[]`。

补充说明：

- Agent SSE 产生的任务快照会写入 `metadata.kind=agent_task_node`
- `metadata.graphId`、`metadata.nodeId`、`metadata.workerId` 可用于把运行任务快照和原始 agent SSE 事件对齐
- Agent SSE 会在启动 agent runtime 前执行本地 RAG 检索：后端按当前登录用户检索 active 商品和 active 商品知识，发出 `rag_context_loaded` 事件，并把检索摘要注入 agent metadata。当前不伪造外部搜索；未命中本地知识时会返回明确降级原因。
- Agent SSE 的商品指定只认本轮显式 `metadata.context.productIds` / `metadata.context.contextRefs`。显式选择后，web-server 会写入 `context.selected_summary`，worker 自动补偿 `creative_video_script`、`ops_listing_copy`、`web_product_description`、`design_detail_page` 等商品型 skill input 时会从这里解析商品名称、平台、类目、卖点和 `product_ref=product:{uuid}`。未选择商品时，不会从会话历史、RAG 摘要、工作区摘要或测试样例中注入默认商品，也不会写入空的 `product_ref/product_refs`。旧 `product_id` 仍只表示旧整数参数兼容字段。
- worker report 会写入 `metadata.report`，evaluation 会写入 `metadata.evaluation`

## 61. 创建运行任务快照

- 方法：`POST`
- 路径：`/api/app/workspaces/{workspaceId}/runs/{runId}/tasks`
- 鉴权：有效 App 用户 JWT

请求体：

```json
{
  "workspaceTaskId": "task-uuid",
  "seq": 1,
  "ownerRole": "planner",
  "status": "pending",
  "attempts": 0,
  "acceptanceScore": 0,
  "acceptancePassed": false,
  "acceptanceNotes": null,
  "executionNotes": null,
  "metadata": {}
}
```

成功响应 `data`：

```json
{
  "id": "run-task-uuid",
  "runId": "run-uuid",
  "workspaceId": "workspace-uuid",
  "workspaceTaskId": "task-uuid",
  "userId": "user-uuid",
  "seq": 1,
  "ownerRole": "planner",
  "status": "pending",
  "attempts": 0,
  "acceptanceScore": 0,
  "acceptancePassed": false,
  "acceptanceNotes": null,
  "executionNotes": null,
  "metadata": {},
  "createdAt": "2026-05-07T00:00:00Z",
  "updatedAt": "2026-05-07T00:00:00Z"
}
```

## 62. 更新运行任务快照

- 方法：`PATCH`
- 路径：`/api/app/workspaces/{workspaceId}/runs/{runId}/tasks/{runTaskId}`
- 鉴权：有效 App 用户 JWT

请求体可包含 `seq`、`ownerRole`、`status`、`attempts`、`acceptanceScore`、`acceptancePassed`、`acceptanceNotes`、`executionNotes`、`metadata`，所有字段均可省略。

成功响应 `data`：`WorkspaceRunTaskResponse`。

## 63. 查询 Agent Skill Run 记录

- 方法：`GET`
- 路径：`/api/app/workspaces/{workspaceId}/runs/{runId}/skill-runs`
- 鉴权：有效 App 用户 JWT

查询参数：

| 参数 | 类型 | 必填 | 说明 |
| --- | --- | --- | --- |
| `status` | `requested \| success \| deferred \| pending_approval \| not_found \| validation_error \| failed` | 否 | Skill 调用状态 |
| `skillId` | `string` | 否 | 只查询某个 skill |
| `role` | `string` | 否 | 只查询某个虚拟员工角色 |
| `page` | `number` | 否 | 默认 1 |
| `pageSize` | `number` | 否 | 默认 20，最大 100 |

补充：`artifactType` 当前还支持 `data_store_metrics_report`、`data_product_metrics_report` 和 `data_campaign_metrics_report`。它们分别来自 `query_store_metrics`、`query_product_metrics` 与 `query_campaign_metrics` 的真实成功输出，用于保存店铺指标报告、商品 / SKU 指标报告和活动 / 投放指标报告。

成功响应 `data`：

```json
{
  "skillRuns": [
    {
      "id": "skill-run-uuid",
      "userId": "user-uuid",
      "workspaceId": "workspace-uuid",
      "runId": "run-uuid",
      "sessionId": "session-id",
      "workerId": "worker.ops",
      "taskId": "task-1",
      "requestId": "task-1-skill-1",
      "skillId": "ops_campaign_plan",
      "role": "ops",
      "status": "success",
      "reason": "需要生成运营计划",
      "summary": "已生成运营计划摘要",
      "error": null,
      "input": {},
      "output": {
        "plan": "launch"
      },
      "executionMode": "native",
      "riskLevel": "low",
      "approvalRequired": false,
      "metadata": {
        "source": "agent_sse"
      },
      "requestedAt": "2026-05-12T00:00:00Z",
      "completedAt": "2026-05-12T00:00:02Z",
      "createdAt": "2026-05-12T00:00:00Z",
      "updatedAt": "2026-05-12T00:00:02Z"
    }
  ],
  "total": 1,
  "page": 1,
  "pageSize": 20,
  "agentContext": {
    "kind": "agent_skill_runs",
    "workspaceId": "workspace-uuid",
    "runId": "run-uuid"
  }
}
```

补充说明：

- Agent SSE 收到 `skill_requested` 时会写入 `status=requested` 的记录。
- Agent SSE 收到 `skill_result` 时会按 `requestId` 更新同一条记录；如果缺少请求记录，会补写一条结果记录，避免审计断点。
- `executionMode`、`riskLevel`、`approvalRequired` 来自后端 Agent runtime 的真实 `skill_result`，前端不应自行推断。
- 搜索证据 artifact、审批记录和告警仍按各自已有接口查询；本接口提供 skill 调用维度的审计索引。

## 工作区交付物接口

工作区交付物用于保存 Agent 运行或人工流程产生的最终报告、岗位交付件、文件引用等结果。Agent SSE 正常完成时会自动创建一个 `artifactType=final_report`、`storageKind=inline`、`format=markdown` 的交付物；当搜索类 `search_*` skill 返回真实 live 来源时，还会自动创建 `artifactType=search_evidence`、`storageKind=inline`、`format=json` 的搜索证据交付物；当部分 creative / design / data / accounting / service / ops / web / engineering 内容类 skill 返回真实成功结果时，还会自动创建对应 markdown 内容交付物。普通 HTTP 接口可用于查询、人工登记、更新或归档交付物。

### 查询工作区交付物列表

- 方法：`GET`
- 路径：`/api/app/workspaces/{workspaceId}/artifacts`
- 鉴权：有效 App 用户 JWT

查询参数：

| 参数 | 类型 | 必填 | 说明 |
| --- | --- | --- | --- |
| `runId` | `uuid` | 否 | 只查询某次运行产生的交付物 |
| `workspaceTaskId` | `uuid` | 否 | 只查询某个工作区任务关联的交付物 |
| `runTaskId` | `uuid` | 否 | 只查询某个运行任务快照关联的交付物 |
| `artifactType` | `string` | 否 | 交付物类型，例如 `final_report`、`search_evidence`、`video_script`、`seeding_copy`、`ip_branding`、`content_calendar`、`live_script`、`product_article`、`creative_trend_report`、`creative_title_ctr_score`、`design_main_image_brief`、`design_detail_page_brief`、`design_color_scheme`、`design_material_spec`、`design_campaign_poster_brief`、`data_dashboard`、`data_funnel_report`、`data_channel_roi_report`、`data_refund_report`、`data_anomaly_report`、`data_trend_forecast_report`、`data_customer_segmentation_report`、`data_competitor_report`、`data_ab_test_report`、`data_attribution_report`、`data_multi_period_trend_report`、`data_ltv_report`、`data_cohort_report`、`data_seasonal_decompose_report`、`data_price_elasticity_report`、`data_demand_forecast_report`、`data_comprehensive_diagnosis_report`、`accounting_cost_report`、`accounting_profit_report`、`accounting_roi_report`、`accounting_budget_plan`、`accounting_break_even_report`、`accounting_compliance_report`、`accounting_pl_statement`、`accounting_budget_vs_actual_report`、`accounting_gmv_waterfall_report`、`accounting_scenario_report`、`accounting_cash_flow_forecast_report`、`accounting_financial_narrative_report`、`service_ticket_playbook`、`service_faq_playbook`、`service_escalation_playbook`、`service_dsr_improvement_plan`、`service_return_plan`、`service_nps_report`、`service_sentiment_report`、`service_nps_driver_report`、`ops_promo_plan`、`ops_pricing_strategy`、`ops_channel_strategy`、`ops_inventory_plan`、`ops_listing_copy`、`ops_assortment_plan`、`ops_execution_plan`、`ops_smart_pricing_report`、`ops_inventory_optimization_report`、`ops_ad_fatigue_report`、`ops_abc_xyz_report`、`web_seo_plan`、`web_title_candidates`、`web_conversion_plan`、`web_store_design_brief`、`web_keyword_matrix`、`web_product_description`、`web_title_seo_score`、`engineering_arch_review_report`、`engineering_bug_analysis_report`、`engineering_perf_plan`、`engineering_tech_selection_report`、`engineering_sla_report`、`role_report`、`file` |
| `status` | `active \| archived` | 否 | 不传时默认查询 `active` |
| `keyword` | `string` | 否 | 匹配标题、预览、文件名、对象 key |
| `page` | `number` | 否 | 默认 1 |
| `pageSize` | `number` | 否 | 默认 20，最大 100 |

成功响应 `data`：

```json
{
  "artifacts": [
    {
      "id": "artifact-uuid",
      "workspaceId": "workspace-uuid",
      "runId": "run-uuid",
      "workspaceTaskId": null,
      "runTaskId": null,
      "userId": "user-uuid",
      "artifactType": "final_report",
      "title": "Agent final report",
      "content": "最终回答正文",
      "preview": "最终回答正文",
      "mimeType": "text/markdown",
      "format": "markdown",
      "storageKind": "inline",
      "bucket": null,
      "objectKey": null,
      "fileName": null,
      "fileSize": null,
      "sha256": null,
      "source": "agent_session",
      "status": "active",
      "validationStatus": "unchecked",
      "validationReport": {},
      "metadata": {},
      "createdAt": "2026-05-09T00:00:00Z",
      "updatedAt": "2026-05-09T00:00:00Z"
    }
  ],
  "total": 1,
  "page": 1,
  "pageSize": 20
}
```

`artifactType=search_evidence` 时：

- `content` 是完整搜索输出 JSON 字符串，包含 `status=live`、`sources`、`data.items`、`provenance`、`confidence`、`freshness`、`evidence`
- `preview` 是后端生成的来源摘要，便于前端列表快速展示
- `metadata.kind = agent_search_evidence`
- `metadata` 包含 `skillId`、`requestId`、`role`、`query`、`sourceCount`、`domains`、`confidence`、`provenance`、`freshness`
- 如果 live 搜索输出的 `output.query` 带有本轮显式选品上下文，`metadata.productContext` 会包含 `productRef/productName/productCategory`，并冗余写入顶层 `metadata.productRef/productName/productCategory`；`preview` 也会出现 `Product:` 行
- 未显式选商品时不会写空商品字段，也不会从 query 文本、历史对话、工作区摘要或预设商品推断商品
- 只有真实 live 来源会创建该交付物；adapter 未配置、provider 失败或无来源时不会创建假搜索证据

`artifactType=video_script / seeding_copy / ip_branding / content_calendar / live_script / product_article / creative_trend_report / creative_title_ctr_score / design_main_image_brief / design_detail_page_brief / design_color_scheme / design_material_spec / design_campaign_poster_brief / data_dashboard / data_funnel_report / data_channel_roi_report / data_refund_report / data_anomaly_report / data_trend_forecast_report / data_customer_segmentation_report / data_competitor_report / data_ab_test_report / data_attribution_report / data_multi_period_trend_report / data_ltv_report / data_cohort_report / data_seasonal_decompose_report / data_price_elasticity_report / data_demand_forecast_report / data_comprehensive_diagnosis_report / accounting_cost_report / accounting_profit_report / accounting_roi_report / accounting_budget_plan / accounting_break_even_report / accounting_compliance_report / accounting_pl_statement / accounting_budget_vs_actual_report / accounting_gmv_waterfall_report / accounting_scenario_report / accounting_cash_flow_forecast_report / accounting_financial_narrative_report / service_ticket_playbook / service_faq_playbook / service_escalation_playbook / service_dsr_improvement_plan / service_return_plan / service_nps_report / service_sentiment_report / service_nps_driver_report / ops_promo_plan / ops_pricing_strategy / ops_channel_strategy / ops_inventory_plan / ops_listing_copy / ops_assortment_plan / ops_execution_plan / ops_smart_pricing_report / ops_inventory_optimization_report / ops_ad_fatigue_report / ops_abc_xyz_report / web_seo_plan / web_title_candidates / web_conversion_plan / web_store_design_brief / web_keyword_matrix / web_product_description / web_title_seo_score / engineering_arch_review_report / engineering_bug_analysis_report / engineering_perf_plan / engineering_tech_selection_report / engineering_sla_report` 时：

- `content` 是后端将对应 creative、design、data、accounting、service、ops、web 或 engineering skill 成功输出包装成 markdown 的结果
- `preview` 是后端从 skill 输出摘要或正文中截取的预览
- `metadata.kind = agent_generated_content`
- `metadata.contentCategory = creative`、`design`、`data`、`accounting`、`service`、`ops`、`web` 或 `engineering`
- `metadata` 还包含 `artifactType`、`skillId`、`requestId`、`role`、`summary`
- `creative_video_script`、`creative_seeding_copy`、`creative_ip_branding`、`creative_content_calendar`、`creative_live_script`、`creative_product_article`、`creative_trend_catch`、`creative_title_ctr_scorer` 成功输出会分别创建脚本、种草文案、IP定位、内容排期、直播脚本、产品软文、趋势报告和标题CTR评分交付物；使用默认商品/排期/直播参数时会以 `data_readiness=calculated_from_defaults` 明示，缺少必填参数时返回 validation error，不会创建假创意交付物
- `design_main_image`、`design_detail_page`、`design_color_scheme`、`design_material_spec`、`design_campaign_poster` 成功输出会分别创建主图 brief、详情页 brief、配色方案、素材规范和活动海报 brief 交付物；主图、详情页或素材规范使用默认商品/平台规则时会以 `data_readiness=calculated_from_defaults` 明示，本阶段不生成真实图片文件，缺少必填参数时返回 validation error
- 只有真实 `success` 的支持名单 skill 才会创建；不会为 deferred/pending_approval 结果伪造内容交付物
- `data_dashboard` 会保留后端 skill 输出中的 `has_data`、`adapter_status`、指标分区、预警和下一步动作；如果 `adapter_status=store_metrics_required`，前端不能把该结果展示成平台实时数据
- `data_funnel_report`、`data_channel_roi_report`、`data_refund_report` 只会在对应 data skill 有用户提供数据并返回 `status=success` 时创建；缺少数据时后端返回 `deferred`，不会创建假报告。`data_channel_roi_report` 和 `data_refund_report` 可由本轮显式自然语言指标触发，例如渠道 GMV / 广告费 / 订单 / UV，或退款原因 / 退款金额；这些数据只来自本轮用户输入，不来自默认商品或未接入平台实时数据
- `data_store_metrics_report`、`data_product_metrics_report` 和 `data_campaign_metrics_report` 分别来自 `query_store_metrics`、`query_product_metrics`、`query_campaign_metrics` 的真实成功输出；只有本轮显式提供店铺 / 商品 / 活动指标并返回 `status=success` 时才会创建，缺指标的 `deferred` 状态不会创建假报告，也不会把未选择商品、预设商品或平台实时数据当作来源
- `data_anomaly_report`、`data_trend_forecast_report`、`data_customer_segmentation_report`、`data_competitor_report` 和 `data_ab_test_report` 来自数据岗位智能分析类 skill 的真实成功输出；异常/趋势缺少必要时序数据时返回 `deferred`，客户分层或竞品分析使用默认估算时会以 `data_readiness=calculated_from_defaults` 明示
- `data_attribution_report` 来自渠道归因分析 skill 的真实成功输出；缺少多渠道输入、少于两个有效渠道或总 GMV 为 0 时返回 `deferred`，不会伪造平台渠道数据
- `data_multi_period_trend_report`、`data_ltv_report`、`data_cohort_report`、`data_seasonal_decompose_report`、`data_price_elasticity_report` 和 `data_demand_forecast_report` 来自数据岗位进阶预测、留存、LTV 和价格弹性 skill 的真实成功输出；多周期趋势、季节性分解或需求预测缺少必要时序数据时返回 `deferred`，LTV、队列留存或价格弹性使用默认估算时会以 `data_readiness=calculated_from_defaults` 明示
- `data_comprehensive_diagnosis_report` 来自综合业务诊断 skill 的 Rust native 编排输出；它会组合经营看板、漏斗、异常、客户、退款、预测、归因和 LTV 分项，缺少输入的分项以 `deferred` 和 `缺失数据` 明示，整体无有效用户数据时不会创建交付物
- `accounting_profit_report` 和 `accounting_roi_report` 缺少必要财务输入时会返回 `deferred`，不会创建假财务报告；`accounting_cost_report` 和 `accounting_budget_plan` 如使用默认参数，会在输出中以 `data_readiness=calculated_from_defaults` 明示
- `accounting_break_even_report`、`accounting_compliance_report`、`accounting_pl_statement`、`accounting_budget_vs_actual_report`、`accounting_gmv_waterfall_report`、`accounting_scenario_report`、`accounting_cash_flow_forecast_report` 和 `accounting_financial_narrative_report` 来自高级财务/经营分析 skill 的真实成功输出；缺少必要核心输入时返回 `deferred`，不会创建假财务报告，使用默认假设时会以 `data_readiness=calculated_from_defaults` 明示
- `service_ticket_playbook`、`service_faq_playbook`、`service_escalation_playbook` 和 `service_dsr_improvement_plan` 来自客服支持类 skill 的真实成功输出；DSR 如使用默认样例评分，会在输出中以 `data_readiness=calculated_from_defaults` 明示
- `service_return_plan`、`service_nps_report`、`service_sentiment_report` 和 `service_nps_driver_report` 来自客服售后/洞察类 skill 的真实成功输出；NPS 或驱动因素分析如使用默认样例数据，会在输出中以 `data_readiness=calculated_from_defaults` 明示
- `ops_promo_plan`、`ops_pricing_strategy`、`ops_channel_strategy`、`ops_inventory_plan`、`ops_listing_copy`、`ops_assortment_plan` 和 `ops_execution_plan` 来自运营岗位核心计划类 skill 的真实成功输出；缺少必要用户输入时后端返回 `deferred`，不会创建假运营交付物
- `ops_smart_pricing_report`、`ops_inventory_optimization_report`、`ops_ad_fatigue_report` 和 `ops_abc_xyz_report` 来自运营岗位高级分析类 skill 的真实成功输出；智能定价缺少竞品价格或 ABC-XYZ 使用演示 SKU 时会以 `data_readiness=calculated_from_defaults` 明示，库存优化和广告疲劳检测缺少必要 SKU/渠道输入时返回 `deferred`，不会创建假高级运营交付物
- `web_seo_plan`、`web_title_candidates`、`web_conversion_plan`、`web_store_design_brief`、`web_keyword_matrix`、`web_product_description` 和 `web_title_seo_score` 来自 Web 增长岗位 skill 的真实成功输出；使用平台规则、通用主推商品或页面基准默认值时会以 `data_readiness=calculated_from_defaults` 明示，缺少必填参数时返回 validation error，不会创建假 Web 增长交付物
- `engineering_arch_review_report`、`engineering_bug_analysis_report`、`engineering_perf_plan`、`engineering_tech_selection_report` 和 `engineering_sla_report` 来自工程岗位 skill 的真实成功输出；当前基于用户输入和内置工程规则生成，不读取真实代码仓库、日志、APM、监控系统或实时外部 benchmark，缺少关键上下文时会以 `data_readiness=calculated_from_defaults` 明示；`engineering_system_check` 可由平台 API 连通性 / 接口健康 / 授权 / 同步状态问法触发，但仍需平台 adapter 和审批，真实测试连接时返回 `pending_approval`，不会创建假健康检查交付物

### 创建工作区交付物

- 方法：`POST`
- 路径：`/api/app/workspaces/{workspaceId}/artifacts`
- 鉴权：有效 App 用户 JWT

请求体：

```json
{
  "runId": "run-uuid",
  "workspaceTaskId": null,
  "runTaskId": null,
  "artifactType": "role_report",
  "title": "投放岗位分析报告",
  "content": "报告正文",
  "preview": "报告摘要",
  "mimeType": "text/markdown",
  "format": "markdown",
  "storageKind": "inline",
  "bucket": null,
  "objectKey": null,
  "fileName": null,
  "fileSize": null,
  "sha256": null,
  "source": "manual",
  "validationStatus": "unchecked",
  "validationReport": {},
  "metadata": {}
}
```

成功响应 `data`：`WorkspaceArtifactResponse`。

补充说明：

- `artifactType` 和 `title` 必填
- `runId`、`workspaceTaskId`、`runTaskId` 可为空；不为空时必须属于当前用户当前工作区
- `storageKind=inline` 时主要使用 `content` 保存正文；`storageKind=object` 时主要使用 `bucket`、`objectKey`、`fileName` 等对象存储字段
- `status` 创建时固定为 `active`

### 查询工作区交付物详情

- 方法：`GET`
- 路径：`/api/app/workspaces/{workspaceId}/artifacts/{artifactId}`
- 鉴权：有效 App 用户 JWT

成功响应 `data`：`WorkspaceArtifactResponse`。

### 导出工作区交付物

- 方法：`GET`
- 路径：`/api/app/workspaces/{workspaceId}/artifacts/{artifactId}/export`
- 鉴权：有效 App 用户 JWT

查询参数：

| 参数 | 类型 | 必填 | 说明 |
| --- | --- | --- | --- |
| `format` | `markdown \| text \| json` | 否 | 导出格式；不传时优先使用交付物原 `format`，无法识别时回退 `markdown` |
| `includeMetadata` | `boolean` | 否 | 是否在导出内容和响应 `metadata` 中包含来源、校验、存储等元信息，默认 `false` |

成功响应 `data`：

```json
{
  "artifactId": "artifact-uuid",
  "workspaceId": "workspace-uuid",
  "title": "Agent final report",
  "fileName": "Agent_final_report.md",
  "mimeType": "text/markdown; charset=utf-8",
  "format": "markdown",
  "content": "# Agent final report\n\n最终回答正文",
  "contentChars": 32,
  "sourceFormat": "markdown",
  "sourceStorageKind": "inline",
  "metadata": {},
  "warnings": []
}
```

补充说明：

- 导出接口只读取当前数据库里已经沉淀的 `content` 或 `preview`，不会读取 `bucket/objectKey` 对应的对象存储文件。
- `content` 为空但 `preview` 有值时会导出 `preview`，并在 `warnings` 中说明。
- `content` 和 `preview` 都为空时返回空内容和 warning，不伪造导出结果。
- `storageKind` 不是 `inline` 时会给出 warning，提示对象存储正文未读取。
- archived artifact 不允许导出，按不存在处理。

### 更新工作区交付物

- 方法：`PATCH`
- 路径：`/api/app/workspaces/{workspaceId}/artifacts/{artifactId}`
- 鉴权：有效 App 用户 JWT

请求体字段均可选：`title`、`content`、`preview`、`mimeType`、`format`、`storageKind`、`bucket`、`objectKey`、`fileName`、`fileSize`、`sha256`、`status`、`validationStatus`、`validationReport`、`metadata`。

成功响应 `data`：`WorkspaceArtifactResponse`。

### 归档工作区交付物

- 方法：`DELETE`
- 路径：`/api/app/workspaces/{workspaceId}/artifacts/{artifactId}`
- 鉴权：有效 App 用户 JWT

成功响应 `data`：`null`。

补充说明：

- 当前实现为软删除，设置 `status = archived`
- 只归档业务记录，不直接删除对象存储里的物理文件

## 工作区 Agent 质量、信任与学习接口

这些接口用于查询或触发 Agent run 的质量检查、保存用户反馈、查询学习记录和信任分。SSE Agent 正常完成时也会自动触发一次质量检查，并发出 `quality_checked` 事件。

### 查询运行质量检查

- 方法：`GET`
- 路径：`/api/app/workspaces/{workspaceId}/runs/{runId}/quality-checks`
- 鉴权：有效 App 用户 JWT

查询参数：

| 参数 | 类型 | 必填 | 说明 |
| --- | --- | --- | --- |
| `page` | `number` | 否 | 默认 1 |
| `pageSize` | `number` | 否 | 默认 20，最大 100 |

成功响应 `data`：

```json
{
  "checks": [
    {
      "id": "quality-check-uuid",
      "workspaceId": "workspace-uuid",
      "runId": "run-uuid",
      "artifactId": "artifact-uuid",
      "userId": "user-uuid",
      "role": "agent",
      "score": 0.82,
      "passed": true,
      "dimensions": {
        "completeness": 0.82,
        "actionability": 0.76,
        "relevance": 0.9,
        "clarity": 0.8,
        "riskAwareness": 0.62,
        "valueDensity": 0.71,
        "confidenceMarking": 0.62,
        "roleDelivery": 0.72,
        "fabricationGuard": 0.86
      },
      "issues": [],
      "suggestions": [],
      "source": "agent_session",
      "metadata": {},
      "createdAt": "2026-05-09T00:00:00Z"
    }
  ],
  "total": 1,
  "page": 1,
  "pageSize": 20
}
```

### 触发运行质量检查

- 方法：`POST`
- 路径：`/api/app/workspaces/{workspaceId}/runs/{runId}/quality-checks`
- 鉴权：有效 App 用户 JWT

请求体：

```json
{
  "artifactId": "artifact-uuid",
  "output": "可选。为空时后端优先读取 artifact.content/preview，再读取 run.report.output"
}
```

成功响应 `data`：

```json
{
  "qualityCheck": {
    "id": "quality-check-uuid",
    "workspaceId": "workspace-uuid",
    "runId": "run-uuid",
    "artifactId": "artifact-uuid",
    "role": "agent",
    "score": 0.82,
    "passed": true,
    "dimensions": {},
    "issues": [],
    "suggestions": [],
    "source": "agent_session",
    "metadata": {},
    "createdAt": "2026-05-09T00:00:00Z"
  },
  "trustUpdate": {
    "role": "agent",
    "skillId": null,
    "previousScore": 0.5,
    "newScore": 0.596,
    "previousLevel": "moderate",
    "newLevel": "moderate",
    "delta": 0.096,
    "reason": "quality check passed; trust score moved toward the answer quality"
  },
  "learningRecords": [
    {
      "id": "learning-record-uuid",
      "workspaceId": "workspace-uuid",
      "runId": "run-uuid",
      "qualityCheckId": "quality-check-uuid",
      "role": "agent",
      "category": "success",
      "content": "High quality answer pattern for similar tasks: ...",
      "confidence": 0.82,
      "sourceAction": "quality_check_success",
      "feedbackRating": null,
      "feedbackComment": null,
      "status": "active",
      "metadata": {},
      "createdAt": "2026-05-09T00:00:00Z",
      "updatedAt": "2026-05-09T00:00:00Z"
    }
  ]
}
```

补充说明：

- `artifactId` 可为空；不为空时必须属于当前用户、当前工作区和当前运行。
- 质量检查当前为后端确定性规则评分，不额外调用 LLM。
- 评分会写入 `agent_quality_checks`，同时更新 `agent_trust_scores`，并写入 `agent_learning_records`。
- 若绑定了交付物，后端会同步更新交付物的 `validationStatus` 与 `validationReport`。

### 提交运行反馈

- 方法：`POST`
- 路径：`/api/app/workspaces/{workspaceId}/runs/{runId}/feedback`
- 鉴权：有效 App 用户 JWT

请求体：

```json
{
  "rating": 5,
  "comment": "回答可直接执行，后续继续保持这种结构。",
  "metadata": {}
}
```

成功响应 `data`：`AgentRunFeedbackResponse`，包含新创建的 `learningRecord`。

补充说明：

- `rating` 范围为 1 到 5。
- 4 到 5 分写入 `feedback_positive`，3 分写入 `feedback_neutral`，1 到 2 分写入 `feedback_negative`。
- 反馈只写入当前用户、当前工作区、当前运行下的学习记录。

### 查询工作区学习记录

- 方法：`GET`
- 路径：`/api/app/workspaces/{workspaceId}/learnings`
- 鉴权：有效 App 用户 JWT

查询参数：

| 参数 | 类型 | 必填 | 说明 |
| --- | --- | --- | --- |
| `runId` | `uuid` | 否 | 只查询某次运行 |
| `role` | `string` | 否 | 只查询某个角色，当前自动评估写入 `agent` |
| `category` | `string` | 否 | 例如 `success`、`blindspot`、`improvement`、`feedback_positive` |
| `page` | `number` | 否 | 默认 1 |
| `pageSize` | `number` | 否 | 默认 20，最大 100 |

成功响应 `data`：

```json
{
  "records": [],
  "total": 0,
  "page": 1,
  "pageSize": 20
}
```

### 查询 Agent 信任分

- 方法：`GET`
- 路径：`/api/app/trust-scores`
- 鉴权：有效 App 用户 JWT

查询参数：

| 参数 | 类型 | 必填 | 说明 |
| --- | --- | --- | --- |
| `role` | `string` | 否 | 只查询某个角色，当前自动评估写入 `agent` |
| `page` | `number` | 否 | 默认 1 |
| `pageSize` | `number` | 否 | 默认 20，最大 100 |

成功响应 `data`：

```json
{
  "scores": [
    {
      "id": "trust-score-uuid",
      "userId": "user-uuid",
      "role": "agent",
      "skillId": null,
      "score": 0.596,
      "trustLevel": "moderate",
      "previousScore": 0.5,
      "previousLevel": "moderate",
      "delta": 0.096,
      "reason": "quality check passed; trust score moved toward the answer quality",
      "source": "quality_check",
      "qualityCheckId": "quality-check-uuid",
      "metadata": {},
      "updatedAt": "2026-05-09T00:00:00Z"
    }
  ],
  "total": 1,
  "page": 1,
  "pageSize": 20
}
```

## Agent 质量、信任与学习接口权限与错误

- 所有接口只允许访问当前 JWT 用户自己的工作区、运行、交付物、质量记录、学习记录和信任分。
- 质量检查不存在返回 `404`，错误码 `-12062`。
- 质量检查归属不合法返回 `403`，错误码 `-12063`。
- 质量检查请求参数或关联关系不合法返回 `400`，错误码 `-12064`。
- 学习记录不存在返回 `404`，错误码 `-12065`。
- 信任分不存在返回 `404`，错误码 `-12066`。

## 63. 查询执行建议队列

- 方法：`GET`
- 路径：`/api/app/execution-queue`
- 鉴权：有效 App 用户 JWT

查询参数：

| 参数 | 类型 | 必填 | 说明 |
| --- | --- | --- | --- |
| `workspaceId` | `uuid` | 否 | 只查询某个工作区 |
| `runId` | `uuid` | 否 | 只查询某个运行记录 |
| `status` | `pending \| approved \| rejected \| executed \| failed \| cancelled` | 否 | 建议状态 |
| `actionType` | `string` | 否 | 动作类型 |
| `page` | `number` | 否 | 默认 1 |
| `pageSize` | `number` | 否 | 默认 20，最大 100 |

成功响应 `data`：

```json
{
  "items": [
    {
      "id": "execution-uuid",
      "userId": "user-uuid",
      "workspaceId": "workspace-uuid",
      "runId": "run-uuid",
      "workspaceTaskId": "task-uuid",
      "actionType": "publish_campaign",
      "description": "建议发布新品上市活动",
      "platform": "douyin",
      "payload": {},
      "expectedImpact": "提升新品曝光",
      "status": "pending",
      "decisionReason": null,
      "result": {
        "adapterStatus": "adapter_not_implemented",
        "message": "Platform execution adapter is not implemented yet for price update."
      },
      "createdAt": "2026-05-07T00:00:00Z",
      "decidedAt": null
    }
  ],
  "total": 1,
  "page": 1,
  "pageSize": 20
}
```

## 64. 创建执行建议

- 方法：`POST`
- 路径：`/api/app/execution-queue`
- 鉴权：有效 App 用户 JWT

请求体：

```json
{
  "workspaceId": "workspace-uuid",
  "runId": "run-uuid",
  "workspaceTaskId": "task-uuid",
  "actionType": "publish_campaign",
  "description": "建议发布新品上市活动",
  "platform": "douyin",
  "payload": {},
  "expectedImpact": "提升新品曝光"
}
```

成功响应 `data`：`ExecutionQueueResponse`。

补充说明：

- `actionType` 和 `description` 必填
- `workspaceId`、`runId`、`workspaceTaskId` 可为空
- 如果传 `runId` 或 `workspaceTaskId`，必须同时传 `workspaceId`
- 当前阶段只记录建议和审批结果，不执行真实平台动作

## 65. 查询执行建议详情

- 方法：`GET`
- 路径：`/api/app/execution-queue/{executionId}`
- 鉴权：有效 App 用户 JWT

成功响应 `data`：`ExecutionQueueResponse`。

## 66. 批准执行建议

- 方法：`POST`
- 路径：`/api/app/execution-queue/{executionId}/approve`
- 鉴权：有效 App 用户 JWT

请求体：

```json
{
  "reason": "人工确认可以执行",
  "result": {}
}
```

成功响应 `data`：`ExecutionQueueResponse`。

补充说明：

- 只有 `pending` 状态的建议可以批准
- 批准后设置 `status = approved`，并写入 `decidedAt`
- 批准后会写入一条 `execution_queue.approve` 审计日志
- 本阶段批准不等于真实执行平台动作，也不会调用 Agent 工具重放
- 如果该执行建议来自工具审批后的平台执行尝试，`result.adapterStatus` 和 `result.message` 会记录真实失败原因，例如 `validation_error`、`connection_missing`、`connection_disabled`、`unsupported_action` 或 `adapter_not_implemented`
- 从 Post 8-49 开始，支持的平台执行动作失败结果会额外返回 `result.adapterRequest`，用于记录本次准备调用 adapter 的规范化请求信封，包括 `skillName`、`platform`、`executionMode`、`idempotencyKey`、`idempotencyKeyFields` 和 `args`；该字段只用于审计和后续 adapter 接入，不代表真实平台写操作已经执行
- 从 Post 8-50 开始，平台执行失败结果由 service 内部 adapter outcome 契约生成，并包含 `result.providerResult` 和 `result.retryable`；当前 adapter 未实现时 `providerResult=null`、`retryable=false`
- 从 Post 8-51 开始，审批通过后的平台执行尝试会通过 service 内部统一 adapter dispatch outcome 入口生成失败结果；外部响应字段不变，当前仍不会调用真实平台写接口，也不会把 `adapter_not_implemented` 伪造成成功

## 67. 拒绝执行建议

- 方法：`POST`
- 路径：`/api/app/execution-queue/{executionId}/reject`
- 鉴权：有效 App 用户 JWT

请求体：

```json
{
  "reason": "预算或素材未确认",
  "result": {}
}
```

成功响应 `data`：`ExecutionQueueResponse`。

补充说明：

- 只有 `pending` 状态的建议可以拒绝
- 拒绝后设置 `status = rejected`，并写入 `decidedAt`
- 拒绝后会写入一条 `execution_queue.reject` 审计日志

## 68. 查询工作区上下文摘要

- 方法：`GET`
- 路径：`/api/app/workspaces/{workspaceId}/context-summary`
- 鉴权：有效 App 用户 JWT

查询参数：

| 参数 | 类型 | 必填 | 说明 |
| --- | --- | --- | --- |
| `conversationId` | `uuid` | 否 | 聚合该会话下已注册的 active 附件 |
| `productId` | `uuid` | 否 | 聚合该商品下 active 资产和素材；为空时优先使用工作区绑定商品 |
| `runId` | `uuid` | 否 | 只聚合该运行记录下的 active 工作区交付物 |
| `contextRefs` | `string` | 否 | 逗号分隔的显式上下文引用，当前支持 `workspace_artifact:{artifactId}` 和 `artifact:{artifactId}`，最多 20 个 |

成功响应 `data`：

```json
{
  "workspaceId": "workspace-uuid",
  "conversationId": "conversation-uuid",
  "productId": "product-uuid",
  "runId": "run-uuid",
  "readiness": "ready",
  "summary": "Workspace context is ready: 1 conversation files, 2 product assets, 3 product materials, 4 workspace artifacts are available for agent grounding and delivery.",
  "counts": {
    "conversationFiles": 1,
    "productAssets": 2,
    "productMaterials": 3,
    "workspaceArtifacts": 4
  },
  "attachmentSources": [
    {
      "sourceType": "conversation_file",
      "id": "file-uuid",
      "title": "brief.pdf",
      "mimeType": "application/pdf",
      "fileSize": 123456,
      "status": "active",
      "refKey": "conversation_file:file-uuid",
      "preview": "brief.pdf",
      "metadata": {
        "conversationId": "conversation-uuid",
        "objectKey": "chat/user/conversation/file.pdf"
      }
    }
  ],
  "productAssetSources": [],
  "productMaterialSources": [],
  "artifactSources": [],
  "missing": [],
  "warnings": [],
  "agentContext": {
    "kind": "workspace_context_summary",
    "readiness": "ready",
    "refs": ["conversation_file:file-uuid"]
  }
}
```

字段说明：

- `readiness=ready` 表示传入或工作区绑定的关键上下文存在，且至少有一个真实来源。
- `readiness=partial` 表示已有部分来源，但缺少会话、商品或其它关键上下文。
- `readiness=empty` 表示没有任何 active 附件、素材、资产或交付物来源。
- `agentContext.refs` 是后续 Agent 上下文注入和前端来源面板可复用的引用 key。
- 该接口只做后端状态聚合，不读取文件正文，不伪造 OCR/文档摘要。
- `contextRefs` 只读取当前用户、当前 workspace 下的 active artifact；不会因为传入 `workspaceId` 自动读取全部历史 run artifact。
- Agent SSE 会在绑定 `workspace_run` 后自动调用该摘要能力，发送 `workspace_context_loaded` 事件，并把 `agentContext.refs` 与 `summary` 注入 agent runtime metadata。

## 68.1 查询工作区上下文内容包

- 方法：`GET`
- 路径：`/api/app/workspaces/{workspaceId}/context-pack`
- 鉴权：有效 App 用户 JWT

查询参数：

| 参数 | 类型 | 必填 | 说明 |
| --- | --- | --- | --- |
| `conversationId` | `uuid` | 否 | 聚合该会话下已注册的 active 附件；只读取 metadata 中已经存在的抽取文本 |
| `productId` | `uuid` | 否 | 聚合该商品下 active 资产描述和素材正文；为空时优先使用工作区绑定商品 |
| `runId` | `uuid` | 否 | 只聚合该运行记录下的 active 工作区交付物 |
| `contextRefs` | `string` | 否 | 逗号分隔的显式上下文引用，当前支持 `workspace_artifact:{artifactId}` 和 `artifact:{artifactId}`，最多 20 个 |
| `maxItems` | `number` | 否 | 返回来源上限，默认 20，最大 50 |
| `maxChars` | `number` | 否 | 返回文本总字符上限，默认 12000，最大 24000 |

成功响应 `data`：

```json
{
  "workspaceId": "workspace-uuid",
  "conversationId": "conversation-uuid",
  "productId": "product-uuid",
  "runId": null,
  "readiness": "ready",
  "summary": "Workspace context pack is ready: 3 readable item(s), 4800 text chars, 0 omitted item(s), from 1 conversation files, 1 product assets, 1 product materials, 1 workspace artifacts.",
  "counts": {
    "conversationFiles": 1,
    "productAssets": 1,
    "productMaterials": 1,
    "workspaceArtifacts": 1
  },
  "items": [
    {
      "sourceType": "product_material",
      "id": "material-uuid",
      "title": "五一咖啡卖点",
      "refKey": "product_material:material-uuid",
      "contentKind": "content",
      "text": "商品素材正文...",
      "textChars": 120,
      "truncated": false,
      "metadata": {
        "productId": "product-uuid",
        "materialType": "selling_points",
        "version": 1
      }
    },
    {
      "sourceType": "conversation_file",
      "id": "file-uuid",
      "title": "brief.pdf",
      "refKey": "conversation_file:file-uuid",
      "contentKind": "metadata_only",
      "text": null,
      "textChars": 0,
      "truncated": false,
      "metadata": {
        "objectKey": "chat/user/conversation/file.pdf",
        "mimeType": "application/pdf"
      }
    }
  ],
  "omitted": [],
  "warnings": [
    "1 conversation file(s) are still pending text extraction"
  ],
  "agentContext": {
    "kind": "workspace_context_pack",
    "readiness": "ready",
    "refs": ["product_material:material-uuid", "conversation_file:file-uuid"],
    "contentRefs": ["product_material:material-uuid"]
  }
}
```

字段说明：

- `contentKind=content` 表示来自商品素材或交付物正文。
- `contentKind=search_evidence_summary` 表示来自 `artifactType=search_evidence` 的真实 live 搜索证据摘要；后端会从搜索 JSON 中整理 query、显式商品上下文、证据摘要、来源数量、置信度、域名和 Top sources，不会触发新搜索。
- `contentKind=preview` 表示交付物只有预览文本可用。
- `contentKind=description` 表示来自商品资产描述。
- `contentKind=extracted_text` 表示会话附件 metadata 已经有可信抽取文本。
- `contentKind=metadata_only` 表示附件只有文件名、对象 key、mime 等元数据；服务端没有可用抽取正文。
- `contentKind=extraction_pending` 表示附件已上传但解析仍未完成。
- `contentKind=extraction_failed` 表示附件解析失败；失败原因位于 `metadata.metadata.extraction.failureReason`。
- `contentKind=extraction_unsupported` 表示当前解析 adapter 不支持该附件。
- `contentKind=unavailable` 表示该来源没有可读文本。
- 该接口不会读取 object storage，不做 OCR，不解析 PDF/图片/视频，也不会把二进制附件伪造成已读正文。
- Agent SSE 会在 `workspace_context_loaded` 后自动调用该内容包能力，发送 `workspace_context_pack_loaded` 事件，并把可读文本追加进 runtime `context.summary`。
- `contextRefs` 中的显式 artifact 会和 `runId` 范围 artifact 合并去重后进入内容包；未显式引用的历史 artifact 不会进入 SSE Agent 本轮上下文。

## 69. 查询支持的平台列表

- 方法：`GET`
- 路径：`/api/app/platform-connect/platforms`
- 鉴权：有效 App 用户 JWT

成功响应 `data`：

```json
{
  "platforms": [
    {
      "platform": "douyin",
      "display": "抖音电商",
      "requiredFields": [
        {
          "key": "app_key",
          "label": "App ID",
          "secret": false,
          "required": true
        }
      ],
      "docUrl": "https://op.jinritemai.com/"
    }
  ]
}
```

## 69.1 查询平台执行 Adapter 能力目录

- 方法：`GET`
- 路径：`/api/app/platform-connect/execution-adapters`
- 鉴权：有效 App 用户 JWT

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

补充说明：

- 该接口只读展示当前用户的平台执行 adapter 能力目录和可执行边界，不触发真实平台写操作。
- `connected`、`enabled`、`connectionStatus` 来自当前用户的 `platform_connections`。
- 当前真实平台写 adapter 尚未实现，因此 `adapterStatus` 固定为 `adapter_not_implemented`，`canExecuteNow=false`，不会伪造成执行成功。
- 前端只能展示该目录和不可执行原因，不能基于该接口新增假执行按钮。
- 从 Post 8-48 开始，该目录的动作定义、required/optional 参数、幂等字段和 adapter 未实现文案来自 service 内部平台执行动作规格注册表；本说明不改变外部响应结构。

## 69.2 平台执行 Preflight 预检

- 方法：`POST`
- 路径：`/api/app/platform-connect/execution-preflight`
- 鉴权：有效 App 用户 JWT

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

补充说明：

- 该接口只做创建审批前的只读预检，不创建审批，不创建执行队列，不调用真实平台写接口。
- `args` 支持顶层字段、`normalized_inputs` / `normalizedInputs`、`output.normalized_inputs` / `output.normalizedInputs`，以兼容 Agent skill 产物和前端表单参数。
- 非平台动作返回 `status=unsupported`、`adapterStatus=unsupported_action`。
- 缺平台、缺商品 ID、缺价格或缺库存返回 `adapterStatus=validation_error`。
- 缺平台连接返回 `connection_missing`，连接禁用返回 `connection_disabled`。
- 当前真实平台写 adapter 尚未实现，因此参数和连接都齐全时仍返回 `adapter_not_implemented`、`canExecuteNow=false`。
- 从 Post 8-47 开始，平台执行 preflight、工具审批 `result.executionReadiness`、审批通过后的执行队列失败详情共用 service 内部平台执行规则模块；本说明不改变外部响应结构。
- 从 Post 8-48 开始，preflight 支持动作判断和 adapter 未实现文案也复用同一份平台执行动作规格注册表。

## 70. 查询平台连接列表

- 方法：`GET`
- 路径：`/api/app/platform-connect/connections`
- 鉴权：有效 App 用户 JWT

查询参数：

| 参数 | 类型 | 必填 | 说明 |
| --- | --- | --- | --- |
| `platform` | `taobao \| jd \| pdd \| douyin` | 否 | 平台 |
| `enabled` | `boolean` | 否 | 是否启用 |

成功响应 `data`：

```json
{
  "connections": [
    {
      "id": "connection-uuid",
      "userId": "user-uuid",
      "platform": "douyin",
      "display": "抖音电商",
      "appKey": "abcd****wxyz",
      "appSecret": "****",
      "accessToken": "****",
      "refreshToken": null,
      "tokenExpiresAt": null,
      "shopId": "shop-001",
      "shopName": "示例店铺",
      "enabled": true,
      "status": "configured",
      "lastTestedAt": null,
      "testResult": {},
      "lastSyncedAt": null,
      "syncStatus": "idle",
      "syncMessage": null,
      "extra": {},
      "createdAt": "2026-05-07T00:00:00Z",
      "updatedAt": "2026-05-07T00:00:00Z"
    }
  ]
}
```

补充说明：

- 敏感字段只返回脱敏值
- 本阶段只保存连接记录，不验证真实平台凭证

## 71. 保存平台连接

- 方法：`POST`
- 路径：`/api/app/platform-connect/connections`
- 鉴权：有效 App 用户 JWT

请求体：

```json
{
  "platform": "douyin",
  "appKey": "app-id",
  "appSecret": "app-secret",
  "accessToken": "access-token",
  "refreshToken": "refresh-token",
  "tokenExpiresAt": null,
  "shopId": "shop-001",
  "shopName": "示例店铺",
  "enabled": true,
  "testResult": {},
  "lastSyncedAt": null,
  "syncStatus": "idle",
  "syncMessage": null,
  "extra": {}
}
```

成功响应 `data`：`PlatformConnectionResponse`。

补充说明：

- `platform` 必须是 `taobao`、`jd`、`pdd`、`douyin`
- 同一用户同一平台重复保存时更新原连接
- `tokenExpiresAt`、`lastSyncedAt` 可省略，也可传 `null` 或 RFC3339 时间字符串
- `syncStatus` 可为 `idle`、`syncing`、`success`、`failed`

## 72. 查询平台连接详情

- 方法：`GET`
- 路径：`/api/app/platform-connect/connections/{connectionId}`
- 鉴权：有效 App 用户 JWT

成功响应 `data`：`PlatformConnectionResponse`。

## 73. 更新平台连接

- 方法：`PATCH`
- 路径：`/api/app/platform-connect/connections/{connectionId}`
- 鉴权：有效 App 用户 JWT

请求体字段同保存平台连接，所有字段均可省略。

成功响应 `data`：`PlatformConnectionResponse`。

## 74. 禁用平台连接

- 方法：`DELETE`
- 路径：`/api/app/platform-connect/connections/{connectionId}`
- 鉴权：有效 App 用户 JWT

成功响应 `data`：`null`。

补充说明：

- 当前实现为软禁用，设置 `enabled = false`、`status = disabled`

## 75. 查询支持的触达渠道列表

- 方法：`GET`
- 路径：`/api/app/platform-connect/channels`
- 鉴权：有效 App 用户 JWT

成功响应 `data`：

```json
{
  "channels": [
    {
      "channel": "wecom",
      "display": "企业微信机器人",
      "requiredFields": [
        {
          "key": "webhook_url",
          "label": "Webhook URL",
          "secret": false,
          "required": true
        }
      ],
      "docUrl": "https://developer.work.weixin.qq.com/document/path/91770"
    }
  ]
}
```

## 76. 查询触达渠道连接列表

- 方法：`GET`
- 路径：`/api/app/platform-connect/channels/connections`
- 鉴权：有效 App 用户 JWT

查询参数：

| 参数 | 类型 | 必填 | 说明 |
| --- | --- | --- | --- |
| `channel` | `wecom \| feishu \| telegram \| discord` | 否 | 触达渠道 |
| `enabled` | `boolean` | 否 | 是否启用 |

成功响应 `data`：`ChannelConnectionListResponse`。

## 77. 保存触达渠道连接

- 方法：`POST`
- 路径：`/api/app/platform-connect/channels/connections`
- 鉴权：有效 App 用户 JWT

请求体：

```json
{
  "channel": "wecom",
  "webhookUrl": "https://example.com/webhook",
  "botToken": null,
  "chatId": null,
  "secret": "secret",
  "extraCredentials": {},
  "enabled": true,
  "lastError": null
}
```

成功响应 `data`：`ChannelConnectionResponse`。

补充说明：

- `channel` 必须是 `wecom`、`feishu`、`telegram`、`discord`
- 同一用户同一渠道重复保存时更新原连接
- 本阶段只保存连接记录，不发送真实消息

## 78. 查询触达渠道连接详情

- 方法：`GET`
- 路径：`/api/app/platform-connect/channels/connections/{connectionId}`
- 鉴权：有效 App 用户 JWT

成功响应 `data`：`ChannelConnectionResponse`。

## 79. 更新触达渠道连接

- 方法：`PATCH`
- 路径：`/api/app/platform-connect/channels/connections/{connectionId}`
- 鉴权：有效 App 用户 JWT

请求体字段同保存触达渠道连接，所有字段均可省略。

成功响应 `data`：`ChannelConnectionResponse`。

## 80. 禁用触达渠道连接

- 方法：`DELETE`
- 路径：`/api/app/platform-connect/channels/connections/{connectionId}`
- 鉴权：有效 App 用户 JWT

成功响应 `data`：`null`。

补充说明：

- 当前实现为软禁用，设置 `enabled = false`、`status = disabled`

## 81. 查询本地店铺指标

- 方法：`GET`
- 路径：`/api/app/platform-connect/metrics`
- 鉴权：有效 App 用户 JWT

查询参数：

| 参数 | 类型 | 必填 | 说明 |
| --- | --- | --- | --- |
| `platform` | `string` | 否 | 平台 |
| `metricKey` | `string` | 否 | 指标键，例如 `gmv`、`orders`、`uv`、`ad_spend` |
| `metricDateFrom` | `string` | 否 | 起始日期，格式 `YYYY-MM-DD` |
| `metricDateTo` | `string` | 否 | 结束日期，格式 `YYYY-MM-DD` |
| `source` | `string` | 否 | 来源，例如 `import` |
| `page` | `number` | 否 | 默认 1 |
| `pageSize` | `number` | 否 | 默认 20，最大 100 |

成功响应 `data`：

```json
{
  "metrics": [
    {
      "id": "metric-uuid",
      "userId": "user-uuid",
      "platform": "douyin",
      "metricDate": "2026-05-07",
      "metricKey": "gmv",
      "metricValue": 1000,
      "dimensionKey": "",
      "dimensionValue": "",
      "source": "import",
      "createdAt": "2026-05-07T00:00:00Z"
    }
  ],
  "total": 1,
  "page": 1,
  "pageSize": 20
}
```

## 82. 保存本地店铺指标

- 方法：`POST`
- 路径：`/api/app/platform-connect/metrics`
- 鉴权：有效 App 用户 JWT

请求体：

```json
{
  "platform": "douyin",
  "metricDate": "2026-05-07",
  "metricKey": "gmv",
  "metricValue": 1000,
  "dimensionKey": "",
  "dimensionValue": "",
  "source": "import"
}
```

成功响应 `data`：`StoreMetricResponse`。

补充说明：

- 同一用户下 `(platform, metricDate, metricKey, dimensionKey, dimensionValue)` 重复时更新指标值
- 本阶段指标来自本地写入或外部同步后的入库，不直接拉取平台 API

## 83. 查询平台同步状态

- 方法：`GET`
- 路径：`/api/app/platform-connect/sync-status`
- 鉴权：有效 App 用户 JWT

成功响应 `data`：

```json
{
  "platforms": [
    {
      "platform": "douyin",
      "display": "抖音电商",
      "connected": true,
      "enabled": true,
      "syncStatus": "idle",
      "syncMessage": null,
      "lastTestedAt": null,
      "lastSyncedAt": null,
      "latestMetricDate": "2026-05-07"
    }
  ]
}
```

## 84. 查询平台只读汇总

- 方法：`GET`
- 路径：`/api/app/platform-connect/summary`
- 鉴权：有效 App 用户 JWT

成功响应 `data`：

```json
{
  "platforms": [
    {
      "platform": "douyin",
      "display": "抖音电商",
      "connected": true,
      "enabled": true,
      "lastTestedAt": null,
      "gmv7d": 1000,
      "orders7d": 20,
      "gmv30d": 5000,
      "orders30d": 90,
      "uv30d": 10000,
      "adRoi30d": 2.5
    }
  ]
}
```

补充说明：

- 汇总只读取 `store_metrics` 本地指标
- `adRoi30d = gmv_30d / ad_spend_30d`，没有广告花费时为 `null`

## 85. 查询工具审批请求列表

- 方法：`GET`
- 路径：`/api/app/approvals/tool-requests`
- 鉴权：有效 App 用户 JWT

查询参数：

| 参数 | 类型 | 必填 | 说明 |
| --- | --- | --- | --- |
| `workspaceId` | `uuid` | 否 | 只查询某个工作区 |
| `runId` | `uuid` | 否 | 只查询某个运行记录，传入时必须同时传 `workspaceId` |
| `status` | `pending \| approved \| executed \| failed \| rejected \| cancelled` | 否 | 审批状态 |
| `skillName` | `string` | 否 | 工具或能力名称 |
| `riskLevel` | `string` | 否 | 风险等级，例如 `L1`、`L2` |
| `page` | `number` | 否 | 默认 1 |
| `pageSize` | `number` | 否 | 默认 20，最大 100 |

成功响应 `data`：

```json
{
  "items": [
    {
      "id": "approval-uuid",
      "userId": "user-uuid",
      "workspaceId": "workspace-uuid",
      "runId": "run-uuid",
      "conversationId": "conversation-uuid",
      "requestRole": "operator",
      "skillName": "platform_update_price",
      "toolCallId": "tool-call-001",
      "riskLevel": "L3",
      "args": {
        "platform": "douyin",
        "product_id": "prod-1",
        "new_price": 99
      },
      "status": "pending",
      "reason": "平台写操作需要审批",
      "actionKey": "agent:run:...",
      "result": {
        "status": "pending",
        "source": "tool_approval_create",
        "executionReadiness": {
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
            "skuId": null,
            "newPrice": 99,
            "quantity": null,
            "page": null,
            "pageSize": null
          },
          "checkedAt": "2026-05-13T00:00:00Z"
        }
      },
      "approvedBy": null,
      "approvedAt": null,
      "executedAt": null,
      "createdAt": "2026-05-07T00:00:00Z",
      "updatedAt": "2026-05-07T00:00:00Z"
    }
  ],
  "total": 1,
  "page": 1,
  "pageSize": 20
}
```

## 86. 创建工具审批请求

- 方法：`POST`
- 路径：`/api/app/approvals/tool-requests`
- 鉴权：有效 App 用户 JWT

请求体：

```json
{
  "workspaceId": "workspace-uuid",
  "runId": "run-uuid",
  "conversationId": "conversation-uuid",
  "requestRole": "operator",
  "skillName": "publish_campaign",
  "toolCallId": "tool-call-001",
  "riskLevel": "L2",
  "args": {},
  "actionKey": "publish:campaign-001",
  "reason": "发布前需要人工确认"
}
```

成功响应 `data`：`ToolApprovalResponse`。

补充说明：

- `skillName` 必填
- `riskLevel` 不传时默认 `L1`
- `args` 不传时默认 `{}`
- `actionKey` 不为空时会做去重；同一用户重复提交返回原审批记录
- 当前阶段只创建审批记录，不执行真实工具
- Agent SSE 收到高风险平台写 skill 结果时，也会通过本服务能力自动创建或复用审批记录；该记录的 `actionKey` 由 `runId`、`skillName`、`toolCallId` 生成，并会同步写入 `tool_approval.create` 审计日志和 `agent_approval_required` 告警
- SSE 自动生成的审批记录会把 skill 输出中的 `normalizedInputs` 和原始 `output` 一并保存到 `args`，供后续审批通过后的自动执行闭环使用
- 平台写类审批创建时，`result.executionReadiness` 会前置记录真实可执行性诊断。当前真实平台 adapter 尚未实现，因此不会返回 `ready`；即使平台连接和参数齐全，也会返回 `adapter_not_implemented`，避免把批准误解成可执行成功。
- `executionReadiness.adapterStatus` 当前可能为 `validation_error`、`connection_missing`、`connection_disabled`、`adapter_not_implemented` 或 `unsupported_action`。

## 87. 查询工具审批请求详情

- 方法：`GET`
- 路径：`/api/app/approvals/tool-requests/{approvalId}`
- 鉴权：有效 App 用户 JWT

成功响应 `data`：`ToolApprovalResponse`。

## 88. 通过工具审批请求

- 方法：`POST`
- 路径：`/api/app/approvals/tool-requests/{approvalId}/approve`
- 鉴权：有效 App 用户 JWT

请求体：

```json
{
  "reason": "人工确认可以继续",
  "result": {}
}
```

成功响应 `data`：`ToolApprovalResponse`。

补充说明：

- 只有 `pending` 状态的审批请求可以通过
- 通过后会先写入 `approvedBy`、`approvedAt` 和 `tool_approval.approve` 审计日志
- 如果该审批对应的是高风险平台动作，后端会自动创建一条 `execution_queue` 记录，并立即尝试进入平台执行闭环
- 若当前平台连接缺失、连接被禁用、参数缺失或平台 adapter 尚未实现，后端会把真实失败结果写回 `toolApproval.result` 和 `executionQueue.result`，不会伪造成功
- 因此通过接口返回的最终 `ToolApprovalResponse.status` 可能不只停留在 `approved`，也可能直接变成 `executed` 或 `failed`
- 当前版本已补齐审批后的执行闭环与审计落库，但远端平台 API adapter 仍未真正接入；因此已连接平台的动作当前会返回 `adapter_not_implemented`
- 从 Post 8-49 开始，支持的平台执行动作失败结果会包含 `result.adapterRequest`，用于记录规范化 adapter 请求和幂等键；unsupported 动作或缺少平台时该字段为 `null`
- 从 Post 8-50 开始，该失败结果还会包含 `result.providerResult` 和 `result.retryable`，用于承接后续真实 adapter 的原始返回和重试语义；当前未实现 adapter 返回 `providerResult=null`、`retryable=false`
- 从 Post 8-51 开始，`validation_error`、`connection_missing`、`connection_disabled`、`unsupported_action` 和 `adapter_not_implemented` 都由 service 内部统一 dispatch outcome 入口生成，审批模块只负责事务、执行队列、审批状态和审计写回

## 89. 拒绝工具审批请求

- 方法：`POST`
- 路径：`/api/app/approvals/tool-requests/{approvalId}/reject`
- 鉴权：有效 App 用户 JWT

请求体：

```json
{
  "reason": "素材或预算未确认",
  "result": {}
}
```

成功响应 `data`：`ToolApprovalResponse`。

补充说明：

- 只有 `pending` 状态的审批请求可以拒绝
- 拒绝后设置 `status = rejected`，写入 `approvedBy`、`approvedAt`，并写入 `tool_approval.reject` 审计日志

## 90. 查询审计日志

- 方法：`GET`
- 路径：`/api/app/approvals/audit-logs`
- 鉴权：有效 App 用户 JWT

查询参数：

| 参数 | 类型 | 必填 | 说明 |
| --- | --- | --- | --- |
| `actorUserId` | `uuid` | 否 | 操作人用户 ID |
| `workspaceId` | `uuid` | 否 | 工作区 ID |
| `runId` | `uuid` | 否 | 运行记录 ID，传入时必须同时传 `workspaceId` |
| `executionQueueId` | `uuid` | 否 | 执行建议 ID |
| `toolApprovalId` | `uuid` | 否 | 工具审批 ID |
| `action` | `string` | 否 | 动作名，例如 `execution_queue.approve` |
| `targetType` | `string` | 否 | 目标类型，例如 `execution_queue`、`tool_approval` |
| `status` | `string` | 否 | 审计状态，默认业务写入为 `success` |
| `page` | `number` | 否 | 默认 1 |
| `pageSize` | `number` | 否 | 默认 20，最大 100 |

成功响应 `data`：

```json
{
  "logs": [
    {
      "id": "audit-uuid",
      "userId": "user-uuid",
      "actorUserId": "user-uuid",
      "workspaceId": "workspace-uuid",
      "runId": "run-uuid",
      "executionQueueId": "execution-uuid",
      "toolApprovalId": null,
      "action": "execution_queue.approve",
      "targetType": "execution_queue",
      "targetId": "execution-uuid",
      "status": "success",
      "reason": "人工确认可以执行",
      "requestId": null,
      "traceId": null,
      "metadata": {},
      "createdAt": "2026-05-07T00:00:00Z"
    }
  ],
  "total": 1,
  "page": 1,
  "pageSize": 20
}
```

## 91. 创建审计日志

- 方法：`POST`
- 路径：`/api/app/approvals/audit-logs`
- 鉴权：有效 App 用户 JWT

请求体：

```json
{
  "actorUserId": "user-uuid",
  "workspaceId": "workspace-uuid",
  "runId": "run-uuid",
  "executionQueueId": "execution-uuid",
  "toolApprovalId": null,
  "action": "manual.review",
  "targetType": "campaign",
  "targetId": "campaign-uuid",
  "status": "success",
  "reason": "人工复核记录",
  "requestId": "request-id",
  "traceId": "trace-id",
  "metadata": {}
}
```

成功响应 `data`：`AuditLogResponse`。

补充说明：

- `action`、`targetType` 必填
- `actorUserId` 不传时默认为当前 JWT 用户
- `status` 不传时默认 `success`
- 审计日志从业务 API 视角只新增，不提供更新或删除接口

## 92. 查询长任务列表

- 方法：`GET`
- 路径：`/api/app/long-tasks`
- 鉴权：有效 App 用户 JWT

查询参数：

| 参数 | 类型 | 必填 | 说明 |
| --- | --- | --- | --- |
| `workspaceId` | `uuid` | 否 | 只查询某个工作区 |
| `runId` | `uuid` | 否 | 只查询某个运行记录，传入时必须同时传 `workspaceId` |
| `status` | `pending \| running \| completed \| failed \| cancelled` | 否 | 任务状态 |
| `taskType` | `string` | 否 | 任务类型 |
| `page` | `number` | 否 | 默认 1 |
| `pageSize` | `number` | 否 | 默认 20，最大 100 |

成功响应 `data`：`LongTaskListResponse`。

## 93. 创建长任务记录

- 方法：`POST`
- 路径：`/api/app/long-tasks`
- 鉴权：有效 App 用户 JWT

请求体：

```json
{
  "workspaceId": "uuid",
  "runId": "uuid",
  "taskType": "daily_briefing",
  "title": "生成日报",
  "status": "pending",
  "progress": 0,
  "totalSteps": 100,
  "currentStep": "queued",
  "role": "ops",
  "inputParams": {},
  "result": {},
  "errorMessage": null
}
```

成功响应 `data`：`LongTaskResponse`。

补充说明：

- 当前阶段只创建任务记录，不启动真实异步执行器
- `runId` 不为空时必须同时传 `workspaceId`

## 94. 查询长任务详情

- 方法：`GET`
- 路径：`/api/app/long-tasks/{taskId}`
- 鉴权：有效 App 用户 JWT

成功响应 `data`：`LongTaskResponse`。

## 95. 更新长任务记录

- 方法：`PATCH`
- 路径：`/api/app/long-tasks/{taskId}`
- 鉴权：有效 App 用户 JWT

请求体字段均可选：`status`、`title`、`progress`、`totalSteps`、`currentStep`、`role`、`result`、`errorMessage`、`startedAt`、`completedAt`。

成功响应 `data`：`LongTaskResponse`。

## 96. 取消长任务记录

- 方法：`POST`
- 路径：`/api/app/long-tasks/{taskId}/cancel`
- 鉴权：有效 App 用户 JWT

请求体：

```json
{
  "errorMessage": "用户取消"
}
```

成功响应 `data`：`LongTaskResponse`。

## 97. 查询今日简报摘要

- 方法：`GET`
- 路径：`/api/app/briefing/daily`
- 鉴权：有效 App 用户 JWT

查询参数：

| 参数 | 类型 | 必填 | 说明 |
| --- | --- | --- | --- |
| `date` | `YYYY-MM-DD` | 否 | 不传时按服务端当前日期 |

成功响应 `data`：`DailyBriefingSummaryResponse`。

补充说明：

- 如果当天已有未归档日报记录，直接返回该记录
- 如果没有日报记录，返回从告警数量和长任务状态派生的摘要
- 当前阶段不调用 LLM 生成日报正文

## 98. 查询日报记录列表

- 方法：`GET`
- 路径：`/api/app/briefing/daily-records`
- 鉴权：有效 App 用户 JWT

查询参数：`archived`、`status`、`dateFrom`、`dateTo`、`page`、`pageSize`。

成功响应 `data`：`DailyBriefingListResponse`。

## 99. 新增或更新日报记录

- 方法：`POST`
- 路径：`/api/app/briefing/daily-records`
- 鉴权：有效 App 用户 JWT

请求体：

```json
{
  "briefingDate": "2026-05-07",
  "title": "2026-05-07 daily briefing",
  "summary": "今日摘要",
  "sections": [],
  "productSummary": {},
  "alertSummary": {},
  "taskSummary": {},
  "source": "manual",
  "status": "ready",
  "archived": false
}
```

成功响应 `data`：`DailyBriefingResponse`。

补充说明：

- 同一用户同一 `briefingDate` 只保留一条记录，重复提交会更新原记录

## 100. 查询日报记录详情

- 方法：`GET`
- 路径：`/api/app/briefing/daily-records/{briefingId}`
- 鉴权：有效 App 用户 JWT

成功响应 `data`：`DailyBriefingResponse`。

## 101. 归档日报记录

- 方法：`POST`
- 路径：`/api/app/briefing/daily-records/{briefingId}/archive`
- 鉴权：有效 App 用户 JWT

成功响应 `data`：`DailyBriefingResponse`。

## 102. 查询后台通知列表

- 方法：`GET`
- 路径：`/api/app/briefing/notifications`
- 鉴权：有效 App 用户 JWT

查询参数：`conversationId`、`eventType`、`consumed`、`page`、`pageSize`。

成功响应 `data`：`BackgroundEventListResponse`。

## 103. 创建后台通知

- 方法：`POST`
- 路径：`/api/app/briefing/notifications`
- 鉴权：有效 App 用户 JWT

请求体：

```json
{
  "conversationId": "uuid",
  "eventType": "task.completed",
  "title": "任务完成",
  "data": {},
  "consumed": false
}
```

成功响应 `data`：`BackgroundEventResponse`。

## 104. 标记单条后台通知已读

- 方法：`POST`
- 路径：`/api/app/briefing/notifications/{eventId}/consume`
- 鉴权：有效 App 用户 JWT

成功响应 `data`：`BackgroundEventResponse`。

## 105. 批量标记后台通知已读

- 方法：`POST`
- 路径：`/api/app/briefing/notifications/consume-all`
- 鉴权：有效 App 用户 JWT

请求体：

```json
{
  "conversationId": "uuid",
  "eventType": "task.completed"
}
```

成功响应 `data`：`BatchUpdateResponse`，其中 `updated` 表示本次更新数量。

## 106. 查询告警列表

- 方法：`GET`
- 路径：`/api/app/briefing/alerts`
- 鉴权：有效 App 用户 JWT

查询参数：

| 参数 | 类型 | 必填 | 说明 |
| --- | --- | --- | --- |
| `workspaceId` | `uuid` | 否 | 只查询某个工作区 |
| `runId` | `uuid` | 否 | 只查询某个运行记录，传入时必须同时传 `workspaceId` |
| `alertType` | `string` | 否 | 告警类型 |
| `severity` | `info \| warning \| critical` | 否 | 严重程度，业务可扩展 |
| `status` | `active \| snoozed \| dismissed \| acted` | 否 | 告警状态 |
| `includeSnoozed` | `boolean` | 否 | 默认 false |
| `page` | `number` | 否 | 默认 1 |
| `pageSize` | `number` | 否 | 默认 20，最大 100 |

成功响应 `data`：`AlertListResponse`。

## 107. 创建告警

- 方法：`POST`
- 路径：`/api/app/briefing/alerts`
- 鉴权：有效 App 用户 JWT

请求体：

```json
{
  "workspaceId": "uuid",
  "runId": "uuid",
  "alertType": "policy_risk",
  "severity": "warning",
  "message": "素材可能触发平台规则",
  "role": "compliance",
  "dedupKey": "policy_risk:asset:123",
  "snoozedUntil": null,
  "payload": {}
}
```

成功响应 `data`：`AlertResponse`。

补充说明：

- `dedupKey` 不为空时，同一用户重复创建 active 告警会返回原记录
- 当前阶段只创建告警记录，不触发 Agent 处理链路

## 108. 查询告警摘要

- 方法：`GET`
- 路径：`/api/app/briefing/alerts/summary`
- 鉴权：有效 App 用户 JWT

成功响应 `data`：`AlertSummaryResponse`，包含 `active`、`bySeverity`、`byType`。

## 109. 批量关闭告警

- 方法：`POST`
- 路径：`/api/app/briefing/alerts/dismiss-all`
- 鉴权：有效 App 用户 JWT

请求体字段均可选：`workspaceId`、`runId`、`alertType`、`severity`。

成功响应 `data`：`BatchUpdateResponse`。

## 110. 查询告警详情

- 方法：`GET`
- 路径：`/api/app/briefing/alerts/{alertId}`
- 鉴权：有效 App 用户 JWT

成功响应 `data`：`AlertResponse`。

## 111. 暂停告警

- 方法：`POST`
- 路径：`/api/app/briefing/alerts/{alertId}/snooze`
- 鉴权：有效 App 用户 JWT

请求体：

```json
{
  "snoozedUntil": "2026-05-07T12:00:00Z"
}
```

成功响应 `data`：`AlertResponse`。

## 112. 关闭告警

- 方法：`POST`
- 路径：`/api/app/briefing/alerts/{alertId}/dismiss`
- 鉴权：有效 App 用户 JWT

请求体：

```json
{
  "reason": "已人工确认"
}
```

成功响应 `data`：`AlertResponse`。

## 113. 标记告警已处理

- 方法：`POST`
- 路径：`/api/app/briefing/alerts/{alertId}/act`
- 鉴权：有效 App 用户 JWT

请求体：

```json
{
  "result": {}
}
```

成功响应 `data`：`AlertResponse`。

补充说明：

- 当前阶段只标记告警状态，不触发 Agent 执行动作
- SSE 长任务进度流、真实后台执行器、LLM 简报生成、Agent 告警处理均未在本阶段实现

## 114. 查询能力包目录

- 方法：`GET`
- 路径：`/api/app/packages/catalog`
- 鉴权：有效 App 用户 JWT

查询参数：

| 参数 | 类型 | 必填 | 说明 |
| --- | --- | --- | --- |
| `packageType` | `domain \| role \| skill \| capability` | 否 | 只查询某类能力包 |
| `status` | `active \| disabled \| deprecated` | 否 | 只查询某个状态 |
| `includeDisabled` | `boolean` | 否 | 默认 false |
| `keyword` | `string` | 否 | 按包 ID 或名称搜索 |
| `page` | `number` | 否 | 默认 1 |
| `pageSize` | `number` | 否 | 默认 20，最大 100 |

成功响应 `data`：`PackageCatalogResponse`，包含分页 `packages` 和按类型分组的 `sections`。

## 115. 创建或更新能力包

- 方法：`POST`
- 路径：`/api/app/packages`
- 鉴权：有效 App 用户 JWT

请求体：

```json
{
  "packageType": "skill",
  "packageId": "listing_writer",
  "packageName": "Listing Writer",
  "version": "0.1.0",
  "status": "active",
  "manifestPath": "skills/listing_writer/manifest.json",
  "dependencies": {},
  "compatibility": {},
  "metadata": {},
  "reason": "initial import"
}
```

成功响应 `data`：`PackageResponse`。

## 115.1 同步旧产品内置能力包

- 方法：`POST`
- 路径：`/api/app/packages/sync-legacy-builtins`
- 鉴权：有效 App 用户 JWT

请求体：

```json
{
  "resetStatus": false
}
```

成功响应 `data`：

```json
{
  "packages": [
    {
      "packageType": "skill",
      "packageId": "builtin.ops",
      "packageName": "Builtin Ops Skill Pack",
      "version": "1.0.0",
      "status": "active",
      "manifestPath": "server/packages/skills/ops/manifest.yaml",
      "dependencies": {
        "capability": [
          "ops.strategy",
          "campaign.orchestration",
          "inventory.optimization",
          "pricing.optimization"
        ]
      },
      "compatibility": {
        "compatible": true,
        "target": "commerce-agents-fullstack"
      },
      "metadata": {
        "source": "legacy_builtin",
        "legacyKind": "skill",
        "runtimeStatus": "adapter_required",
        "requiresAgentRuntimeAdapter": true
      }
    }
  ],
  "createdCount": 52,
  "updatedCount": 0,
  "totalCount": 52,
  "manifestBackedCount": 20,
  "derivedCapabilityCount": 32,
  "resetStatus": false
}
```

补充说明：

- 本接口把旧产品 `server/packages` 中 4 个 domain、6 个 role、10 个 skill manifest 同步为当前用户的能力包治理记录。
- 服务端还会从旧 manifest 的能力字段派生 `capability` 记录，用于后续能力矩阵、校验和运行锁治理。
- 已存在包会更新名称、版本、manifest 路径、依赖、兼容性和 metadata；默认保留用户当前 `status`。
- 只有传 `resetStatus=true` 时，已有包状态才会按旧 manifest 重置。
- 该接口只同步治理目录，不动态修改 Agent runtime catalog，也不执行旧 Python skill。

## 116. 查询能力包详情

- 方法：`GET`
- 路径：`/api/app/packages/{packageType}/{packageId}`
- 鉴权：有效 App 用户 JWT

成功响应 `data`：`PackageResponse`。

## 117. 修改能力包

- 方法：`PATCH`
- 路径：`/api/app/packages/{packageType}/{packageId}`
- 鉴权：有效 App 用户 JWT

请求体字段均可选：`packageName`、`version`、`status`、`manifestPath`、`dependencies`、`compatibility`、`metadata`、`reason`。

成功响应 `data`：`PackageResponse`。

## 118. 校验能力包

- 方法：`GET`
- 路径：`/api/app/packages/{packageType}/{packageId}/validate`
- 鉴权：有效 App 用户 JWT

成功响应 `data`：`PackageValidationResponse`，包含：

- `valid`：是否通过
- `errors`：阻断性问题
- `warnings`：非阻断提醒
- `dependencies`：依赖包检查结果

补充说明：

- 当前阶段基于数据库记录校验状态、依赖、兼容性
- 不扫描或写入 manifest 文件，不接入 Agent 动态目录加载

## 119. 启用能力包

- 方法：`POST`
- 路径：`/api/app/packages/{packageType}/{packageId}/activate`
- 鉴权：有效 App 用户 JWT

请求体：

```json
{
  "reason": "resume package"
}
```

成功响应 `data`：`PackageResponse`。

## 120. 停用能力包

- 方法：`POST`
- 路径：`/api/app/packages/{packageType}/{packageId}/deactivate`
- 鉴权：有效 App 用户 JWT

请求体：

```json
{
  "reason": "temporarily disabled"
}
```

成功响应 `data`：`PackageResponse`。

## 121. 回滚能力包状态

- 方法：`POST`
- 路径：`/api/app/packages/{packageType}/{packageId}/rollback`
- 鉴权：有效 App 用户 JWT

请求体：

```json
{
  "reason": "rollback last status change"
}
```

成功响应 `data`：`PackageResponse`。

补充说明：回滚只恢复上一条状态历史中的 `fromStatus`，不修改 Agent runtime。

## 122. 查询能力包状态历史

- 方法：`GET`
- 路径：`/api/app/packages/{packageType}/{packageId}/history`
- 鉴权：有效 App 用户 JWT

查询参数：

| 参数 | 类型 | 必填 | 说明 |
| --- | --- | --- | --- |
| `limit` | `number` | 否 | 默认 50，最大 500 |

成功响应 `data`：`PackageHistoryListResponse`。

## 123. 查询运行能力包锁

- 方法：`GET`
- 路径：`/api/app/workspaces/{workspaceId}/runs/{runId}/package-locks`
- 鉴权：有效 App 用户 JWT

成功响应 `data`：`PackageLockListResponse`。

## 124. 保存运行能力包锁

- 方法：`POST`
- 路径：`/api/app/workspaces/{workspaceId}/runs/{runId}/package-locks`
- 鉴权：有效 App 用户 JWT

请求体：

```json
{
  "entries": [
    {
      "packageType": "skill",
      "packageId": "listing_writer",
      "packageName": "Listing Writer",
      "lockedVersion": "0.1.0",
      "lockedStatus": "active",
      "required": true,
      "lockSource": "manual",
      "manifestPath": "skills/listing_writer/manifest.json",
      "notes": "",
      "metadata": {}
    }
  ]
}
```

成功响应 `data`：`PackageLockListResponse`。

补充说明：保存时会替换该 `runId` 下既有锁记录。

## 125. 检查运行能力包锁漂移

- 方法：`GET`
- 路径：`/api/app/workspaces/{workspaceId}/runs/{runId}/package-locks/drift`
- 鉴权：有效 App 用户 JWT

成功响应 `data`：`PackageLockDriftResponse`，包含当前锁、漂移列表和摘要统计。

补充说明：

- 漂移类型包括包缺失、版本变化、状态变化、必需包未启用
- 当前阶段只做治理和审计，不强制 Agent 执行链路按锁运行

## 会话接口权限与错误

- 所有会话接口都只允许访问当前 JWT 用户自己的数据
- 当前用户必须存在于 `app_users`，且状态为 `enabled`
- 会话不存在返回 `404`，错误码 `-12002`
- 会话不属于当前用户返回 `403`，错误码 `-12003`
- 文件不存在返回 `404`，错误码 `-12004`
- 文件不属于当前用户返回 `403`，错误码 `-12005`
- 请求参数不合法返回 `400`，错误码 `-12006`

## 商品接口权限与错误

- 所有商品接口都只允许访问当前 JWT 用户自己的数据
- 当前用户必须存在于 `app_users`，且状态为 `enabled`
- 商品不存在或已归档商品被更新时返回 `404`，错误码 `-12007`
- 商品不属于当前用户返回 `403`，错误码 `-12008`
- 请求参数不合法返回 `400`，错误码 `-12009`
- 商品知识不存在或已归档商品知识被更新时返回 `404`，错误码 `-12010`
- 商品知识不属于当前商品或当前用户时返回 `403`，错误码 `-12011`
- 商品知识请求参数不合法返回 `400`，错误码 `-12012`
- 商品资产不存在或已归档商品资产被更新时返回 `404`，错误码 `-12013`
- 商品资产不属于当前商品或当前用户时返回 `403`，错误码 `-12014`
- 商品资产请求参数不合法返回 `400`，错误码 `-12015`
- 商品素材不存在或已归档商品素材被更新时返回 `404`，错误码 `-12016`
- 商品素材不属于当前商品或当前用户时返回 `403`，错误码 `-12017`
- 商品素材请求参数不合法返回 `400`，错误码 `-12018`
- 商品竞品不存在或已归档商品竞品被更新时返回 `404`，错误码 `-12019`
- 商品竞品不属于当前商品或当前用户时返回 `403`，错误码 `-12020`
- 商品竞品请求参数不合法返回 `400`，错误码 `-12021`

## 营销活动接口权限与错误

- 所有营销活动接口都只允许访问当前 JWT 用户自己的数据
- 当前用户必须存在于 `app_users`，且状态为 `enabled`
- 营销活动不存在返回 `404`，错误码 `-12022`
- 营销活动不属于当前用户返回 `403`，错误码 `-12023`
- 营销活动请求参数不合法返回 `400`，错误码 `-12024`
- 活动绑定商品不存在、已归档或不属于当前用户时，复用商品相关错误码 `-12007` 或 `-12008`

## 工作区与执行队列接口权限与错误

- 所有工作区、任务、运行记录、运行任务快照、工作区交付物、执行建议接口都只允许访问当前 JWT 用户自己的数据
- 当前用户必须存在于 `app_users`，且状态为 `enabled`
- 工作区不存在或已归档工作区被继续操作时返回 `404`，错误码 `-12025`
- 工作区不属于当前用户返回 `403`，错误码 `-12026`
- 工作区请求参数不合法返回 `400`，错误码 `-12027`
- 工作区任务不存在返回 `404`，错误码 `-12028`
- 工作区任务不属于当前工作区或当前用户返回 `403`，错误码 `-12029`
- 工作区运行记录不存在返回 `404`，错误码 `-12030`
- 运行任务快照不存在返回 `404`，错误码 `-12031`
- 执行建议不存在返回 `404`，错误码 `-12032`
- 执行建议不属于当前用户返回 `403`，错误码 `-12033`
- 执行建议请求参数不合法返回 `400`，错误码 `-12034`
- 工作区交付物不存在返回 `404`，错误码 `-12059`
- 工作区交付物不属于当前用户、当前工作区、运行记录或任务时返回 `403`，错误码 `-12060`
- 工作区交付物请求参数不合法返回 `400`，错误码 `-12061`
- 工作区绑定商品或活动时，复用商品/活动相关错误码 `-12007`、`-12008`、`-12022`、`-12023`

## 平台连接接口权限与错误

- 所有平台连接、触达渠道连接、本地指标接口都只允许访问当前 JWT 用户自己的数据
- 当前用户必须存在于 `app_users`，且状态为 `enabled`
- 平台连接不存在返回 `404`，错误码 `-12035`
- 平台连接不属于当前用户返回 `403`，错误码 `-12036`
- 触达渠道连接不存在返回 `404`，错误码 `-12037`
- 触达渠道连接不属于当前用户返回 `403`，错误码 `-12038`
- 平台连接请求参数不合法、平台或渠道不支持、ID 格式错误时返回 `400`，错误码 `-12039`

## 审批与审计接口权限与错误

- 所有工具审批请求和审计日志接口都只允许访问当前 JWT 用户自己的数据
- 当前用户必须存在于 `app_users`，且状态为 `enabled`
- 工具审批请求不存在返回 `404`，错误码 `-12040`
- 工具审批请求不属于当前用户返回 `403`，错误码 `-12041`
- 审批与审计请求参数不合法、ID 格式错误、状态不允许流转时返回 `400`，错误码 `-12042`
- 审计日志不存在返回 `404`，错误码 `-12043`
- 关联工作区、运行记录、会话、执行建议时，复用对应模块的归属校验和错误码

## 长任务、简报、通知与告警接口权限与错误

- 所有长任务、日报记录、后台通知、告警接口都只允许访问当前 JWT 用户自己的数据
- 当前用户必须存在于 `app_users`，且状态为 `enabled`
- 长任务不存在返回 `404`，错误码 `-12044`
- 长任务不属于当前用户返回 `403`，错误码 `-12045`
- 日报记录不存在返回 `404`，错误码 `-12046`
- 日报记录不属于当前用户返回 `403`，错误码 `-12047`
- 后台通知不存在返回 `404`，错误码 `-12048`
- 后台通知不属于当前用户返回 `403`，错误码 `-12049`
- 告警不存在返回 `404`，错误码 `-12050`
- 告警不属于当前用户返回 `403`，错误码 `-12051`
- 请求参数不合法、ID 格式错误、日期格式错误、`runId` 缺少 `workspaceId` 时返回 `400`，错误码 `-12052`
- 关联工作区、运行记录、会话时，复用对应模块的归属校验和错误码

## 能力包治理接口权限与错误

- 所有能力包注册表、状态历史、运行能力包锁接口都只允许访问当前 JWT 用户自己的数据
- 当前用户必须存在于 `app_users`，且状态为 `enabled`
- 能力包不存在返回 `404`，错误码 `-12053`
- 能力包不属于当前用户返回 `403`，错误码 `-12054`
- 请求参数不合法、包类型不支持、状态不支持、ID 格式错误时返回 `400`，错误码 `-12055`
- 能力包校验失败如需作为阻断错误返回 `409`，错误码 `-12056`
- 能力包锁不存在返回 `404`，错误码 `-12057`
- 能力包锁不属于当前用户、工作区或运行记录返回 `403`，错误码 `-12058`
- 关联工作区、运行记录时，复用对应模块的归属校验和错误码
- 动态 Agent 目录加载、manifest 写入、Agent runtime 锁强制执行未在本阶段实现
