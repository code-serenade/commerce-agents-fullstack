# Post 8-16：会话附件抽取文本登记能力

## 状态

done

## 本轮定位

旧产品迁移主体计划已完成 `12/12`。本轮是主体计划后的第 16 个补强项：把旧产品“附件正文可进入问答上下文”的能力迁到新产品后端契约里。

## 产品内核变化

- 会话附件不再只能保存文件元数据。
- 后端新增统一登记入口，把可信抽取文本写入 `conversation_files.metadata`。
- 现有工作区 `context-pack` 会自动读取 `metadata.extractedText`，因此附件正文可以进入 Agent 问答上下文。

## 后端变化

- 新增设计文档：`temp/DEVELOPMENT_DOCS/backend/conversation_file_text_extraction_post_8_16.md`。
- Repo 层新增 `ConversationFileService::update_metadata`，只更新 `conversation_files` 单表 metadata。
- Service 层新增 `ConversationApi::update_file_extraction`，负责登录用户、附件归属、会话归属、active 状态、文本长度和 metadata object 校验。
- Web-server 新增：
  - `PUT /api/app/conversation-files/{fileId}/extraction`
  - 请求 DTO：`UpdateConversationFileExtractionRequest`
  - 响应复用 `ConversationFileResponse`
- `ConversationFileResponse` 增加 `metadata` 字段，便于前端和联调侧看到抽取状态。

## 前端变化

- `apps/web/src/api/conversation.ts` 同步新增 `updateConversationFileExtractionApi`。
- `ConversationFileResponse` 类型同步新增 `metadata`。
- 本轮不新增页面入口，避免前端先出现未规划的手工按钮；后续 OCR/解析器或附件管理 UI 可直接复用该 API。

## 用户可感知变化

- 当后续附件解析器或手工登记调用该 API 后，用户上传的附件正文会成为 Agent 回答时可用的真实上下文。
- 工作区上下文内容包会从 `metadata_only` 推进到 `extracted_text`，右侧上下文过程也能呈现更明确的可读内容状态。
- 如果没有解析器调用该 API，系统不会假装已经读取 PDF、图片或视频正文。

## 后续缺口

- OCR、PDF/Word/Excel 解析、图片识别和 object storage 文件读取还未实现。
- 前端附件解析入口、解析状态队列和失败重试需要后续独立开发。
