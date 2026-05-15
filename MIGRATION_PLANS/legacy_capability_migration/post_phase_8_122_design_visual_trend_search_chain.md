# Post 8-122：设计岗位视觉趋势搜索链路

## 状态

done

## 本轮目标

把旧产品设计 skill 中隐藏的视觉趋势搜索能力，迁成新产品框架里的可见、可审计、可降级 skill 链路。

旧产品做法：

- 主图设计内部搜索“平台 + 商品/电商 + 主图设计 + 爆款 + 高点击率 + 视觉趋势 + 2026”。
- 活动海报内部搜索“活动名 + 平台 + 活动海报 + 设计趋势 + 爆款 + 2026”。
- 搜索失败时被吞掉，前端看不到搜索是否发生，也看不到来源降级。

新产品做法：

- 设计岗位先运行真实 native skill。
- 当用户明确要求视觉趋势、设计趋势、爆款主图、高点击率、爆款海报、竞品视觉、最新/联网/公开来源时，追加 `search_trends`。
- 纯“查最新视觉趋势”类请求只触发 `search_trends`，不强行生成设计 brief。
- 搜索通过统一 `search_*` adapter、SSE `skill_requested` / `skill_result` 和降级规则展示。

## 涉及能力

- `design_main_image`
- `design_campaign_poster`
- `design_detail_page`
- `search_trends`

## 后端变化

- `design` 岗位新增 `autosuggest_design_requests`，支持 native + search 双 skill 编排。
- 主图、活动海报、详情页设计在用户有明确实时趋势诉求时可追加 `search_trends`。
- `search_trends.input.query` 按设计场景生成：
  - 主图：`{platform} {subject} 主图设计 爆款 高点击率 视觉趋势 2026`
  - 活动海报：`{campaign} {platform} {subject} 活动海报 设计趋势 爆款 2026`
  - 详情页：`{platform} {subject} 详情页设计 高转化 视觉趋势 2026`
- 主图/详情页趋势默认搜索窗口为 60 天，活动海报默认 30 天；用户说“最近 N 天”时按自然语言覆盖。
- `design` 角色白名单允许使用 `search_trends`。

## 商品上下文边界

- 只有本轮显式选中商品时，搜索 input 才携带 `product_ref/product_name/product_category`。
- 未显式选品时，不从历史摘要、默认商品或演示商品注入商品上下文。
- 这继续保持“指定商品”和“未指定商品”两条链路分开。

## 用户可感知变化

- 用户问“给冷萃咖啡做主图设计方案，联网查最新爆款主图和高点击率视觉趋势”时，过程区会看到两个步骤：主图设计 brief、外部视觉趋势搜索。
- 用户问“做 618 活动海报，并查最新活动海报设计趋势”时，过程区会看到活动海报 brief 和趋势搜索两个步骤。
- 用户只问“查一下淘宝咖啡类目最新爆款主图视觉趋势”时，只会进入搜索步骤，不会生成无关设计 brief。
- 搜索 provider 没配置或无可信来源时，后端仍会明确降级，不伪造设计趋势资料。

## 验证

- `cargo fmt --all`
- `cargo test -p agent design`
- `cargo test -p agent search`
- `cargo test -p agent keeps_roles_from_requesting_unowned_skills`
- `cargo check -p agent -p web-server`
- `cargo clippy -p agent -p web-server`
- `git diff --check`
