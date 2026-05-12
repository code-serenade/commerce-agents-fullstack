# Post 8-21：Native skill 可达性补强

状态：done

当前进度：

- 旧产品能力迁移主体计划已完成 `12/12`。
- 本轮属于主体计划后的第 21 个补强项：`Post 8-21`。

本次目标：

- 不新增假前端入口，继续以后端为主。
- 把已经迁到 Rust 后端、但真实自然问法命中率还不够高的旧产品能力，补进主问答链路。
- 通过 autosuggest 补强，让用户更容易触发正确的 native skill 或正确的 deferred engineering skill。

涉及范围：

- 产品内核：问答主链路从“已有 skill”继续推进到“已有 skill 更容易被真实用户问法命中”。
- 后端 agent：补强 `autosuggest_ops/data/accounting/service/engineering` 的意图覆盖。
- 文档：重写统一后的迁移主计划，消除已失效的 `backend/crates/agent` 旧硬约束。

本次完成的命中补强：

- `ops_channel_strategy`
- `ops_listing_copy`
- `ops_assortment_planning`
- `data_competitor_analysis`
- `data_channel_roi`
- `data_refund_decomposition`
- `accounting_compliance_check`
- `service_nps_driver_analysis`
- `engineering_arch_review`
- `engineering_bug_analysis`
- `engineering_perf_optimize`
- `engineering_tech_selection`

用户可感知变化：

- 用户问“渠道分配”“上架文案”“货盘组合”“竞品分析”“退款帕累托”“税务合规”“NPS驱动因素”“架构评审”“性能优化”“技术选型”等问题时，更容易看到后端命中对应能力，而不是落回泛化执行计划。
- 这些变化都来自后端真实 skill 选择和 SSE 过程事件，不是前端伪造。
- engineering 相关能力当前仍多为 `deferred` 工程分析骨架，但现在可以更准确命中正确 skill，并向用户说明真实缺口。

文件变更：

- `temp/DEVELOPMENT_DOCS/backend/commerce_capability_migration_master_plan.md`
- `temp/DEVELOPMENT_DOCS/backend/agent_native_skill_reachability_post_8_21.md`
- `backend/crates/agent/src/agent/worker/runner.rs`
- `MIGRATION_PLANS/legacy_capability_migration/skill_migration_matrix.md`
- `MIGRATION_PLANS/legacy_capability_migration/development_log.md`

验证：

- [x] `cargo fmt`
- [x] `cargo test -p agent autosuggest`
- [x] `cargo check -p agent`
- [x] `cargo check`
- [x] `cargo clippy`
- [x] `git diff --check`

后续缺口：

- 这些补强主要解决“可达性”，不是全部解决“真实外部依赖”。
- `engineering_arch_review`、`engineering_bug_analysis`、`engineering_perf_optimize`、`engineering_tech_selection` 仍属于 `adapter_required/deferred`，后续需要真正的工程分析上下文或外部资料增强。
- `data_channel_roi`、`data_refund_decomposition` 当前命中更准确，但真实平台级明细数据和前端专门可视化仍待后续阶段继续补齐。
