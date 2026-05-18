# Post 8-11 Agent 搜索证据沉淀

## 定位

这是主体迁移计划完成后的第 11 个补强点。Post 8-10 已让 Agent `search_*` skill 能读取 `[external_search]` 并返回 live/degraded 结构化结果，本轮把 live 搜索结果沉淀进新产品 workspace artifact 体系。

## 已完成

- `skill_result` SSE 原始事件增加可选 `output` 字段，承载 skill 结构化输出。
- `WorkspaceApi::record_agent_search_evidence` 新增后端入口。
- 当 `search_*` skill 满足 `status=success`、`executionMode=adapter`、`output.status=live`、`sources` 非空时，创建 `artifactType=search_evidence`。
- `search_evidence` 使用现有 `workspace_artifacts` 表：
  - `format=json`
  - `storageKind=inline`
  - `source=agent_session`
  - `content` 保存完整搜索输出 JSON
  - `metadata.kind=agent_search_evidence`
- SSE 写入成功后复用 `workspace_artifact_created` 事件。
- 降级、未配置或无来源时不创建 artifact。

## 用户可感知变化

用户发起实时搜索类电商问题时，如果外部搜索 provider 返回真实来源，工作区会产生可追溯的“搜索证据”交付物。前端后续可以在右侧来源面板、工作区历史或任务详情里展示来源、域名、置信度和 freshness。

## 后续

- 前端把 `search_evidence` artifact 做成 Kimi Agent 风格的来源面板。
- 部署环境配置真实 `[external_search]` provider。
- 后续视重复程度，把 Agent 和 App API 的搜索归一化逻辑抽到共享 crate。
