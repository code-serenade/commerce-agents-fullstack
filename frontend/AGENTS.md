# 仓库 Agent 指令

本仓库是协议驱动项目。根目录 `AGENTS.md` 是 Codex 入口。回答架构问题、判断文件落点或修改代码前，必须先阅读下面的协议入口，再根据任务范围阅读对应协议文件。

## 必读入口

1. `AGENT_PROTOCOL/PROTOCOL.md`
2. `AGENT_PROTOCOL/BUILD_RULES.md`

## 协议目录

所有给 AI 阅读的仓库协议都放在 `AGENT_PROTOCOL/` 下。

按任务范围继续阅读：

- App 工作：先读 `AGENT_PROTOCOL/protocols/apps/index.md`，再读 `AGENT_PROTOCOL/protocols/apps/<app>.md`
- 页面工作：同时阅读 `AGENT_PROTOCOL/protocols/page-types/` 下匹配页面类型的协议
- DataTable 使用或实现：读 `AGENT_PROTOCOL/protocols/components/data-table.md`
- Package 工作：读 `AGENT_PROTOCOL/protocols/packages/` 下对应包协议
- Showcase 工作：读 `AGENT_PROTOCOL/protocols/showcases/` 下对应协议

## 硬规则

不要把 app 工作当成空白前端页面实现。App 必须优先消费框架已有能力；只有检查共享库存后，才允许补充 app 本地业务装配代码。
