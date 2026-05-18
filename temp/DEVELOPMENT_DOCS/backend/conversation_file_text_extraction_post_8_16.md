# Post 8-16：会话附件抽取文本登记 API 后端设计

## 需求摘要

旧产品支持附件、素材和教学/导出类能力，问答链路需要能读取用户上传附件里的可读内容。新产品当前 `WorkspaceApi::context_pack` 已经会从 `conversation_files.metadata` 中读取抽取文本，但缺少一个后端 API 把可信抽取结果登记进去。

本阶段只建立“附件可读文本登记契约”，不做 OCR、PDF 解析、图片识别，也不读取 object storage。后续解析器、OCR worker 或外部 adapter 都通过同一个 API 写入抽取结果。

## 数据表设计

不新增表，不新增 migration。

复用已有表：

```text
conversation_files
```

相关字段：

```text
id              uuid primary key
user_id         uuid
conversation_id uuid
status          text
metadata        jsonb
```

写入策略：

```json
{
  "extractedText": "可被 Agent 上下文读取的文本",
  "textContent": "兼容字段，同 extractedText",
  "extraction": {
    "status": "ready",
    "contentKind": "text",
    "source": "manual",
    "language": "zh-CN",
    "textChars": 128,
    "updatedAt": "2026-05-10T00:00:00Z",
    "metadata": {}
  }
}
```

## 权限设计

- 接口必须登录，使用 App JWT。
- `user_id` 来自 JWT，按 UUID 字符串处理。
- 只能更新当前用户自己的附件。
- 附件必须是 `active` 状态。
- 附件所属会话必须属于当前用户且未删除。

## HTTP 接口

```text
PUT /api/app/conversation-files/{fileId}/extraction
```

请求体：

```json
{
  "text": "附件抽取出的正文",
  "contentKind": "text",
  "source": "manual",
  "language": "zh-CN",
  "metadata": {}
}
```

字段约束：

- `text`：必填，trim 后不能为空，最多 100000 个字符。
- `contentKind`：可选，默认 `text`，最多 32 字符。
- `source`：可选，默认 `manual`，最多 64 字符。
- `language`：可选，最多 32 字符。
- `metadata`：可选，必须是 JSON object。

成功响应：

```text
ConversationFileResponse
```

其中会包含更新后的 `metadata`。

## 错误码

复用已有错误码：

- `CONVERSATION_FILE_NOT_FOUND`
- `CONVERSATION_FILE_FORBIDDEN`
- `CONVERSATION_NOT_FOUND`
- `CONVERSATION_FORBIDDEN`
- `CONVERSATION_REQUEST_INVALID`

## 与 Agent 上下文关系

本接口写入的 `metadata.extractedText` 会被当前 `WorkspaceApi::context_pack` 自动读取，并进入 Agent 的 `workspaceContextPack` 和 `context.summary`。因此它是附件内容进入问答主链路的后端入口。

## 开发顺序

1. Repo 层新增 `ConversationFileService::update_metadata`。
2. Service 层新增 `ConversationApi::update_file_extraction`。
3. Web-server 新增 DTO、handler 和路由。
4. 更新 `API_DOCS/app.md`。
5. 前端 API 类型同步，不新增页面入口。
6. 更新迁移记录与验收记录。
