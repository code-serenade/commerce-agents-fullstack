# Post 8-114：外部搜索来源时效过滤

状态：done

## 本轮目标

继续补强外部搜索 / 实时资料能力。provider 返回来源后，不能只看 URL 是否可追溯，还要看有发布时间的来源是否落在用户请求的时间窗口内，避免旧资料污染实时问答和搜索证据。

## 本轮完成

- App 外部搜索 API 会按 `days/freshnessDays` 过滤有发布时间且超出时间窗口的来源。
- Agent `search_*` skill 会按 `output.query.days` 过滤有发布时间且超出时间窗口的来源。
- 保留无发布时间或无法解析日期的来源，但不会把它们当作 freshness 加分依据。
- `provenance` 增加 stale 过滤数量，App API 使用 `staleFilteredCount`，Agent skill 使用 `stale_filtered_count`。
- Agent `freshness` 增加 `stale_filtered_count`。
- 过滤后仍有可信来源时继续 live；过滤后没有可信来源时降级，不创建假搜索证据 artifact。

## 用户可感知变化

- 用户说“最近 7 天咖啡趋势”时，90 天前的旧文章不会进入实时来源。
- 如果 provider 只返回过期资料，系统会明确提示没有可采信实时来源。
- 后续前端来源面板可以展示“已过滤 N 条过期来源”，让用户知道不是系统漏掉了，而是后端按时间窗口做了可信度筛选。

## 验证

- [x] `cargo fmt --all`
- [x] `cargo test -p agent search`
- [x] `cargo test -p service external_search`
- [x] `cargo check -p service -p agent -p web-server`
- [x] `cargo clippy -p service -p agent -p web-server`
- [x] `git diff --check`
