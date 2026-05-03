# Codex 执行协议

在本仓库写代码前，Codex 必须先阅读 `AI_PROTOCOLS/` 下的相关协议文档。

## 必读文档

每次开始编码任务时：

1. 先阅读 `AI_PROTOCOLS/AI_WORKFLOW.md`。
2. 再按变更范围阅读对应协议：
   - 数据库或 migration：`AI_PROTOCOLS/MIGRATION_GUIDE.md` 和 `AI_PROTOCOLS/TABLE_ADDING_PROTOCOL.md`
   - repo 层：`AI_PROTOCOLS/REPO_GUIDE.md`
   - service 层：`AI_PROTOCOLS/SERVICE_GUIDE.md`
   - web-server、route、handler、DTO、SSE 或 WebSocket：`AI_PROTOCOLS/WEB_SERVER_GUIDE.md`
   - 认证集成：`AI_PROTOCOLS/AUTH_INTEGRATION_GUIDE.md`
   - 错误码：`AI_PROTOCOLS/ERROR_CODE_GUIDE.md`
   - LLM client 或模型调用：`AI_PROTOCOLS/LLM_CLIENT_GUIDE.md`

## 工作规则

协议文档就是项目边界。新增实现前，优先参考现有的 migration、repo、service 和 web-server 示例，不要随意引入新的分层方式或代码风格。

如果用户需求引入了 `AI_PROTOCOLS/` 尚未覆盖的新行为，必须在改代码的同时更新对应协议或 API 文档。
