# Post 8-43：会话附件解析状态契约

## 状态

done

## 本次迁移目标

旧产品里附件、素材、教学/导出能力的前提是“文件内容能进入问答上下文”。新产品之前已有附件上传、可信文本登记和 `context-pack`，但缺少解析状态。本轮补齐状态契约，让后端可以表达：

- 等待解析：`pending`
- 已成功抽取正文：`extracted`
- 解析失败：`failed`
- 当前解析器不支持：`unsupported`

## 后端变化

- `ConversationApi::register_file`：新注册会话附件默认写入 `metadata.extractionStatus=pending`。
- `ConversationApi::update_file_extraction`：扩展请求字段 `status/text/contentKind/source/parser/language/failureReason/metadata`。
- `WorkspaceApi::context_pack`：只读取 `extractionStatus=extracted` 的附件文本。
- `WorkspaceApi::context_pack`：不可读附件返回 `extraction_pending`、`extraction_failed`、`extraction_unsupported`，并生成对应 warnings。

## 前端变化

- `frontend/apps/web/src/api/conversation.ts` 同步 API adapter 类型。
- 本阶段不新增页面入口，不做假解析状态 UI。

## 用户可感知变化

- 用户上传附件后，后端状态会进入 `pending`，后续前端可真实展示“等待解析”。
- 解析成功后，附件正文会进入问答上下文。
- 解析失败或格式不支持时，Agent 不会继续读取旧正文，避免回答误以为已经看过附件。

## 边界

本阶段不实现：

- OCR
- PDF / Office 解析
- 图片识别
- object storage 文件读取
- 后台 parser worker

这些能力应在后续 adapter 阶段继续接入，并复用本轮状态契约。

## 验证

- `cargo fmt`
- `cargo test -p service conversation_file_extraction -- --nocapture`
- `cargo test -p service context_pack_respects_attachment_extraction_status -- --nocapture`
- `cargo check -p service`
- `cargo check -p web-server`
- `cargo check -p service -p web-server`
- `cargo clippy -p service -p web-server`
- `pnpm -C frontend/apps/web rules`
- `pnpm -C frontend/apps/web lint`
- `pnpm -C frontend/apps/web test`
- `pnpm -C frontend/apps/web build`
- `pnpm -C frontend rules`
- `git diff --check`
