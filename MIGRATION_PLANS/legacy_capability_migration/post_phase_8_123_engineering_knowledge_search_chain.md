# Post 8-123：工程岗位外部资料搜索链路

## 状态

done

## 本轮目标

把旧产品工程 skill 中隐藏的外部资料搜索能力，迁成新产品框架里的可见、可审计、可降级 skill 链路。

旧产品做法：

- Bug 排查内部搜索“错误类型 + 技术栈 + 解决方案 + 排查 + 最佳实践”。
- 技术选型内部搜索“选型领域 + 技术选型 + 对比 + 性能 + 2026”。
- 搜索失败时被吞掉，前端无法看到是否搜索、来源是否可信、是否降级。

新产品做法：

- 工程岗位先运行真实 native skill。
- 当用户明确要求联网、搜索、公开资料、已知解决方案、最佳实践、最新对比、性能 benchmark、技术对比时，追加 `search_market_info(topic=general)`。
- 搜索通过统一 `search_*` adapter、SSE `skill_requested` / `skill_result` 和降级规则展示。

## 涉及能力

- `engineering_bug_analysis`
- `engineering_tech_selection`
- `engineering_perf_optimize`
- `engineering_arch_review`
- `search_market_info(topic=general)`

## 后端变化

- `engineering` 岗位新增 `autosuggest_engineering_requests`，支持 native + search 双 skill 编排。
- 普通工程 Bug 排查、技术选型、性能优化、架构评估仍只触发 native skill。
- 用户明确要求外部资料、benchmark 或最佳实践时追加 `search_market_info`。
- Bug 排查搜索 query 形如：`{error_type} {tech_stack} 解决方案 排查 最佳实践 2026`。
- 技术选型搜索 query 形如：`{domain} 技术选型 对比 性能 benchmark 2026`。
- 工程搜索默认窗口为 90 天，可解析“最近 N 天”和“最多 N 条来源”。
- `engineering` 角色白名单允许使用 `search_market_info`。

## 用户可感知变化

- 用户问“最近支付接口频繁报错，帮我排查 bug 根因”时，仍只看到 Bug 排查方案。
- 用户问“最近支付接口频繁报错，并联网查已知解决方案和最佳实践”时，过程区会看到 Bug 排查和外部资料搜索两个步骤。
- 用户问“消息队列技术选型，查最新对比和性能 benchmark”时，过程区会看到技术选型报告和外部资料搜索两个步骤。
- 搜索 provider 没配置或无可信来源时，后端仍会明确降级，不伪造 benchmark、社区案例或技术对比。

## 验证

- `cargo fmt --all`
- `cargo test -p agent engineering`
- `cargo test -p agent search`
- `cargo test -p agent keeps_roles_from_requesting_unowned_skills`
- `cargo check -p agent -p web-server`
- `cargo clippy -p agent -p web-server`
- `git diff --check`
