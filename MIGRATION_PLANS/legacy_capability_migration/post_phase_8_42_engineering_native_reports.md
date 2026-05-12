# Post 8-42 Engineering Native Reports

## 状态

done

## 当前进度

- 旧产品能力迁移主体计划已完成 `12/12`。
- 本轮属于主体计划后的第 42 个补强项：`Post 8-42`。

## 本次目标

- 把工程岗位里仍停留在骨架状态的分析类 skill 转成新产品 Rust Agent native 输出。
- 让工程岗位的架构评估、Bug 排查、性能优化、技术选型和 SLA 监控结果能进入 workspace artifact 体系。
- 保留真实边界：当前不读取真实代码仓库、日志、APM、监控系统或实时外部 benchmark。

## 涉及范围

- 产品内核：工程虚拟员工从“触达 skill 骨架”推进到“能生成可沉淀、可审计、可复用的工程报告”。
- 后端 Agent：`backend/crates/agent/src/skills/engineering.rs` 统一成功输出 `status=success`、`has_data=true`、`adapter_status=manual_input`、`data_readiness`、`summary`、`data_boundary`。
- 后端 service：`WorkspaceApi::materialize_agent_content_artifact` 新增工程类 artifact 映射。
- API/SSE：更新 `API_DOCS/app.md` 与 `API_DOCS/sse.md`，说明 engineering 内容 artifact 的支持范围和数据边界。
- 前端：本阶段不新增页面和假按钮；后续前端只需按已有 workspace artifact / SSE 事件展示真实工程报告。

## 用户可感知变化

- 用户在问答中触发工程岗位时，架构评估、Bug 排查、性能优化、技术选型和 SLA 监控成功结果会出现在工作区交付物里。
- 如果用户没有提供足够上下文，输出会带 `data_readiness=calculated_from_defaults`，前端可以提示“基于默认工程规则”。
- 平台接口健康检查仍不会伪造成已执行；需要真实平台 adapter 和审批后再进入执行闭环。

## 当前验证

- [x] `cargo fmt`
- [x] `cargo test -p agent engineering_ -- --nocapture`
- [x] `cargo test -p service content_artifact_builds_from_engineering_ -- --nocapture`

## 后续缺口

- 接真实代码仓库、日志、trace、APM 和压测数据 adapter。
- 将工程报告在前端工作区右侧过程面板、artifact 面板中做更清晰展示。
- `engineering_system_check` 需要接平台配置读取、审批和 adapter 执行闭环。
