# Post 8-112：搜索 Skill 结果数量输入契约

状态：done

## 本轮目标

把旧产品搜索策略里的“搜索结果数量”迁入新产品 Agent skill 输入契约。Post 8-111 已让后端归一化阶段遵守数量限制，本轮让 `search_*` skill 自身也能声明和解析该策略。

## 本轮完成

- 六个搜索 skill 都声明可选输入 `max_results`。
- Runtime 同时兼容 `max_results` 和 `maxResults`。
- 显式 skill 输入优先级高于 `[external_search].default_max_results`。
- 超出范围的整数会被 clamp 到 `1..=12`。
- `output.query.max_results`、provider 请求参数、live `sources/data.items/provenance/evidence` 和搜索证据 artifact 继续保持同一个数量契约。
- 不改变 provider 未配置、失败或无可信来源时的 degraded/deferred 语义。

## 用户可感知变化

- 后续前端或 Agent 策略可以按任务显式要求“只看 3 条来源”“最多 5 条证据”。
- 系统不会只能依赖全局默认数量，搜索过程更贴近策略选择和用户指令。
- 来源面板和搜索证据交付物的数量会更可控，避免展示过多来源。

## 验证

- [x] `cargo fmt --all`
- [x] `cargo test -p agent search`
- [x] `cargo check -p agent -p web-server`
- [x] `cargo clippy -p agent -p web-server`
- [x] `git diff --check`
