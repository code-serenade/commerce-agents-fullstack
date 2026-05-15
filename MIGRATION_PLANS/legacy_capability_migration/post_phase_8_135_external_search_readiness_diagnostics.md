# Post 8-135：外部搜索 readiness 诊断契约

状态：done

当前进度：

- 旧产品能力迁移主体计划已完成 `12/12`。
- 本轮属于主体计划后的第 135 个补强项：`Post 8-135`。

本次目标：

- 让旧产品里的“联网搜索 / 实时资料 / 公开来源”能力迁到新产品后，不只知道 `search_*` 成功或降级，还能知道外部搜索 adapter 为什么不可用。
- 给 App API 和 Agent `search_*` skill 补统一 readiness 诊断，方便后续前端把外部搜索状态展示清楚。

涉及范围：

- 后端 service：`ExternalSearchStatusResponse` 与 `ExternalSearchResponse` 增加 `readiness`。
- 后端 agent：`search_*` skill 的 `output.adapter.readiness` 和 `metadata.adapter_readiness` 增加同语义诊断。
- API 文档：同步 `API_DOCS/app.md` 和 `API_DOCS/sse.md`。
- 迁移矩阵：同步搜索能力状态说明。

用户可感知变化：

- 后续前端展示“联网 / 实时资料”步骤时，可以明确告诉用户当前是“外部搜索未启用”、“已启用但缺配置”，还是“已配置但 provider 本次没有返回可信来源”。
- 如果搜索 provider 未配置，系统仍不会假装搜到了内容；回答和过程卡片可以更准确说明降级原因。
- 如果 provider 已配置但失败或返回无 URL 来源，用户会看到这是本次 provider 结果不可采信，而不是误以为产品没有搜索能力。

边界说明：

- 本轮没有接淘宝、抖音、巨量千川或其它官方平台。
- 本轮没有配置真实搜索 provider，也没有写入假 URL 或假 API key。
- `readiness.hasApiKey` 只返回布尔值，不返回密钥原文。
- `readiness.status=configured` 只表示后端会尝试请求 provider，不保证本次一定有 live 来源。

当前验证：

- [x] `cargo fmt --all`
- [x] `cargo test -p agent search`
- [x] `cargo test -p service external_search`
- [x] `cargo check -p agent -p service -p web-server`
- [x] `cargo clippy -p agent -p service -p web-server`
- [x] `git diff --check`

后续缺口：

- 真实部署仍需要配置可访问的 `[external_search]` provider。
- 前端后续需要把 `readiness` 映射到来源面板、过程卡片和降级提示。
- 更进一步的 provider 健康检查历史、后台探活和 provider 管理界面仍未实现。
