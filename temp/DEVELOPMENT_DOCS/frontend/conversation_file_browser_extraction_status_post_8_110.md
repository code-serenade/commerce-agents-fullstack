# Post 8-110 会话附件浏览器文本抽取状态对齐

## 需求摘要

用户要求继续把旧产品资料/skill 能力迁移到新产品，并且前端只适配后端已存在的真实能力。本轮围绕会话附件资料链路：后端已经提供可信附件抽取登记接口，workspace context pack 也只读取 `extractionStatus=extracted` 的附件正文；前端上传附件时应把“上传成功”和“抽取状态”分开处理。

## 后端接口

不新增 API。

复用已有接口：

```text
PUT /api/app/conversation-files/{fileId}/extraction
```

已有状态：

- `extracted`
- `failed`
- `unsupported`
- `pending`

## 前端落点

```text
frontend/apps/web/src/pages/site/agent-chat/view-model/agent-chat-attachments.ts
```

这是 Agent Chat 页面私有上传流程，不新增共享组件，不复制 app-kit/ui-components 能力。

## 业务流程

上传文件后：

1. 正常走 presign upload。
2. 正常登记 conversation file。
3. 前端判断文件是否是浏览器可读文本。
4. 可读文本：
   - 使用 `File.text()` 读取。
   - 非空时调用 extraction API 登记 `status=extracted`。
   - 最多登记 100000 字符，并在 metadata 标记 `truncated`。
5. 空文本：
   - 调用 extraction API 登记 `status=failed`。
   - `failureReason=File contains no readable text.`
6. 非文本 / 浏览器不可读文件：
   - 调用 extraction API 登记 `status=unsupported`。
   - 不伪造正文。
7. 抽取状态登记失败：
   - 上传仍视为成功。
   - 前端不把该附件标记为可读资料。

## 用户可感知变化

- 上传 `.txt`、`.md`、`.csv`、`.json`、HTML 等文本资料后，用户再点“资料”选择它，Agent 可以读取真实文本内容。
- 上传 PDF、图片、Office 等非文本资料时，附件仍然上传成功，但系统会明确标记当前浏览器文本抽取不支持，不会假装已读取正文。
- 文本为空或读取失败时，上传不再被误判为失败；只是该附件不会作为可读正文进入问答上下文。

## 验证

计划运行：

- `pnpm -C frontend/apps/web rules`
- `pnpm -C frontend/apps/web lint`
- `pnpm -C frontend/apps/web test`
- `pnpm -C frontend/apps/web build`
- `pnpm -C frontend rules`
- `git diff --check`

