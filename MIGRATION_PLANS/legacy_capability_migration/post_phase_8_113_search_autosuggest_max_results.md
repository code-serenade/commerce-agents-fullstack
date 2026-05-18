# Post 8-113：搜索自然语言结果数量解析

状态：done

## 本轮目标

把 Post 8-112 的 `max_results` skill 输入契约接到 worker autosuggest。用户自然语言里说“最多 3 条来源”“top 4 sources”“返回 5 条结果”时，后端自动生成的搜索 skill input 能带上数量限制。

## 本轮完成

- `autosuggest_search` 的六个搜索 input builder 都会尝试写入 `max_results`。
- 支持中文结果数量表达：`3条来源`、`3个来源`、`5条结果`、`返回4条公开资料` 等。
- 支持英文表达：`top 4 sources`、`limit 5 results`、`max 6 items` 等。
- 结果数会 clamp 到 `1..=12`。
- 不会把 `最近30天` 这类时间窗口误识别成 `max_results`。
- 不改变搜索 intent 识别、显式选品上下文或 provider 降级语义。

## 用户可感知变化

- 用户问“联网搜咖啡趋势，最多 3 条来源”时，后端会把 `max_results=3` 传入搜索 skill。
- 搜索来源数量更贴近用户表达，右侧来源面板和搜索证据交付物会更可控。
- “最近30天”仍会作为时间范围，不会误变成 30 条来源。

## 验证

- [x] `cargo fmt --all`
- [x] `cargo test -p agent search`
- [x] `cargo check -p agent -p web-server`
- [x] `cargo clippy -p agent -p web-server`
- [x] `git diff --check`
