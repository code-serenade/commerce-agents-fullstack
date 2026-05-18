# Post 8-116：数据竞品与价格弹性搜索链路

状态：done

## 本轮目标

迁移旧产品数据分析里“竞品分析 / 价格弹性分析会静默搜索实时竞品或市场定价数据”的能力，但按新产品架构拆成可见 skill 链路，让用户、前端和审计记录都能看到外部搜索步骤。

## 本轮完成

- data worker 支持竞品分析实时链路：
  - `data_competitor_analysis`
  - `search_competitor`
- data worker 支持价格弹性实时链路：
  - `data_price_elasticity`
  - `search_competitor`
- 只有用户明确要求最新、实时、联网、搜索、公开资料、市场定价或竞品价格区间时才追加搜索。
- 普通竞品分析和普通价格弹性分析仍只运行 native skill。
- 价格弹性链路的搜索 focus 稳定为 `price`。
- 本轮顺带修复 `infer_search_max_results` 解析连续数字时遇到中文标点可能触发 byte 边界 panic 的问题。
- 搜索 provider 降级测试兼容真实 HTTP 状态错误文案，仍然不创建假来源。

## 用户可感知变化

- 用户说“做冷萃咖啡竞品分析，联网搜最新竞品价格和公开资料”时，过程区会先显示竞品分析 skill，再显示竞品外部搜索 skill。
- 用户说“分析这个商品价格弹性，并联网查实时市场定价和竞品价格区间”时，过程区会先显示价格弹性分析，再显示价格外部搜索。
- 如果搜索未配置、失败、无可信来源或来源过期，系统会明确降级，不会把 native 分析包装成实时市场情报。

## 边界

- 本轮不新增 HTTP API，不新增数据库表，不新增前端页面。
- 本轮不在 data native skill 内部调用搜索 adapter。
- 本轮不接淘宝、抖音、千川等官方平台 API。
- 本轮不默认注入任何商品；只有本轮显式选中商品才携带商品上下文。

## 验证

- [x] `cargo fmt --all`
- [x] `cargo test -p agent data`
- [x] `cargo test -p agent search`
- [x] `cargo check -p agent -p web-server`
- [x] `cargo clippy -p agent -p web-server`
- [x] `git diff --check`
