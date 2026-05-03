# Codex 协议入口

## 根协议

1. `AI_PROTOCOLS/AI_PROTOCOL.md`
2. `AI_PROTOCOLS/FULLSTACK_WORKFLOW.md`

## 前端协议

前端工作必须先读：

1. `frontend/AGENT_PROTOCOL/PROTOCOL.md`
2. `frontend/AGENT_PROTOCOL/BUILD_RULES.md`

再按任务范围继续阅读对应前端协议：

- `frontend/AGENT_PROTOCOL/protocols/apps/`
- `frontend/AGENT_PROTOCOL/protocols/page-types/`
- `frontend/AGENT_PROTOCOL/protocols/components/`
- `frontend/AGENT_PROTOCOL/protocols/packages/`

## 后端协议

后端工作必须先读：

1. `backend/AI_PROTOCOLS/PROTOCOL.md`
2. `backend/AI_PROTOCOLS/BUILD_RULES.md`

再按任务范围继续阅读对应后端协议：

- 数据库或 migration：`backend/AI_PROTOCOLS/MIGRATION_GUIDE.md` 和 `backend/AI_PROTOCOLS/TABLE_ADDING_PROTOCOL.md`
- Repo 层：`backend/AI_PROTOCOLS/REPO_GUIDE.md`
- Service 层：`backend/AI_PROTOCOLS/SERVICE_GUIDE.md`
- Web server、route、handler、DTO、SSE 或 WebSocket：`backend/AI_PROTOCOLS/WEB_SERVER_GUIDE.md`
- 认证集成：`backend/AI_PROTOCOLS/AUTH_INTEGRATION_GUIDE.md`
- 错误码：`backend/AI_PROTOCOLS/ERROR_CODE_GUIDE.md`
- LLM client 或模型调用：`backend/AI_PROTOCOLS/LLM_CLIENT_GUIDE.md`

## 全栈共享文件

- 需求：`temp/REQUIREMENTS/`
- 后端设计文档：`temp/DEVELOPMENT_DOCS/backend/`
- 前端设计文档：`temp/DEVELOPMENT_DOCS/frontend/`
- API 文档：`API_DOCS/`

## 规则

1. 需求可以来自 `temp/REQUIREMENTS/`，也可以来自当前聊天；如果需求来自聊天，设计前先整理需求摘要。
2. 任何需要新 API 或数据库变更的功能，都必须从后端设计开始。
3. 新增或修改 API 时，必须同步更新 `API_DOCS/`。
4. 不允许在 `backend/` 下维护第二份 API 文档目录。
