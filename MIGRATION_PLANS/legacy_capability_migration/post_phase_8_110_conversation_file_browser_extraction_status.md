# Post 8-110 会话附件浏览器文本抽取状态对齐

## 本轮目标

继续把旧产品“上传资料后进入问答上下文”的能力装进新产品框架。

本轮不新增 OCR、PDF、Office、图片识别或 object storage 后台解析 worker；只把前端上传流程和已有后端抽取状态 API 对齐，让文本类附件马上变成可读资料，非文本附件明确进入 unsupported 状态。

## 修改范围

前端：

- `frontend/apps/web/src/pages/site/agent-chat/view-model/agent-chat-attachments.ts`
- `frontend/apps/web/src/pages/site/agent-chat/view-model/agent-chat-attachments.test.ts`

文档：

- `temp/DEVELOPMENT_DOCS/frontend/conversation_file_browser_extraction_status_post_8_110.md`
- `API_DOCS/app.md`
- `MIGRATION_PLANS/legacy_capability_migration/development_log.md`
- `MIGRATION_PLANS/legacy_capability_migration/phase_acceptance_log.md`
- `MIGRATION_PLANS/legacy_capability_migration/skill_migration_matrix.md`

## 后端能力使用方式

复用已有后端接口：

```text
PUT /api/app/conversation-files/{fileId}/extraction
```

前端上传并登记附件后，会根据文件类型登记：

- 文本可读且非空：`status=extracted`
- 文本为空：`status=failed`
- 非浏览器文本类：`status=unsupported`
- 读取失败：`status=failed`

后端 `workspace_context_pack` 仍只读取 `status=extracted` 的可信正文。

## 用户可感知变化

- 用户上传 `.txt`、`.md`、`.csv`、`.json` 等文本资料后，系统会立即登记正文，后续勾选“资料”时 Agent 能读取真实文本。
- 用户上传 PDF、图片、Office 等文件时，附件仍会上传成功，但不会假装已解析正文；当前状态会是 unsupported，后续等真实 parser adapter 接入。
- 如果文本文件为空或浏览器读取失败，上传不会被误显示为失败，只是该附件不会作为可读上下文进入问答。

## 当前边界

- 不做 OCR。
- 不解析 PDF / Office / 图片 / 视频。
- 不从 object storage 后台读取文件。
- 不伪造附件正文。

## 验收结论

通过。

