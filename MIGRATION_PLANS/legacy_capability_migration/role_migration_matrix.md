# 角色迁移矩阵

## 目标角色

| 旧角色 | 新 runtime_role | 当前状态 | 新落点 | 迁移重点 |
| --- | --- | --- | --- | --- |
| 对话助手 | `chat` | exists | `backend/config/json/agent/roles.json` | 快速问答、澄清、轻流程 |
| 运营执行 | `ops` | exists | `backend/config/json/agent/roles.json` | 活动、转化、选品、库存、渠道、执行计划 |
| 数据处理 | `data` | exists | `backend/config/json/agent/roles.json` | 指标、漏斗、趋势、异常、竞品、看板 |
| 客服 | `service` | exists | `backend/config/json/agent/roles.json` | 售前售后、投诉、退款、DSR、升级 |
| 设计 | `design` | exists | `backend/config/json/agent/roles.json` | 主图、详情页、素材规格、视觉方案 |
| 财务 | `accounting` | exists | `backend/config/json/agent/roles.json` | 成本、利润、ROI、预算、合规 |
| 工程 | `engineering` | exists | `backend/config/json/agent/roles.json` | 架构、接口、性能、平台连接、SLA |
| 网站增长 | `web` | exists | `backend/config/json/agent/roles.json` | SEO、关键词、页面转化、店铺内容 |
| 创意 | `creative` | exists | `backend/config/json/agent/roles.json` | 视频脚本、种草、直播、内容日历 |
| 千川投放 | `douyin_ads` | exists_new | `backend/config/json/agent/roles.json` | 新产品扩展角色，保留并与 data/creative/accounting 协同 |

## 需要补齐的角色维度

每个角色后续需要补齐或核对：

- `keywords`：触发词是否覆盖旧产品和电商真实说法。
- `preferred_actions`：是否能区分 plan/analyze/create/execute/query。
- `required_capabilities`：是否和 skill catalog 对齐。
- `optional_capabilities`：是否能支持跨角色协作。
- role prompt：`backend/config/prompts/zh/roles/*.md` 是否足够业务化。
- 可用 skill：`backend/crates/agent/src/roles/skill_catalog.rs` 是否允许正确 skill。
- 协作关系：是否能形成 primary/support roles。

## 典型协作关系

- `ops` -> `data`、`creative`、`accounting`、`design`
- `data` -> `ops`、`accounting`、`service`
- `service` -> `ops`、`data`
- `design` -> `creative`、`web`、`ops`
- `accounting` -> `ops`、`data`、`douyin_ads`
- `engineering` -> `data`、`ops`
- `web` -> `creative`、`data`、`design`
- `creative` -> `ops`、`design`、`web`
- `douyin_ads` -> `data`、`creative`、`accounting`、`ops`
