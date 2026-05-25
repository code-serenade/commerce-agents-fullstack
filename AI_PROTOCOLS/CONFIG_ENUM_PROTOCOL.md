# Config Enum Protocol

这个协议约束配置类字段的存储、接口和前端展示。

## 根原则

配置项的数据库字段必须保存稳定枚举值，不允许保存面向用户的展示文案。

允许保存：

- 枚举值：`fast`、`commander`、`manual`、`higher_vocational`
- 稳定业务 key：`role.ops`、`ops`、`campaign.pacing`
- 显式版本号或配置 id：`pricing_rule_v1`

不允许保存：

- 中文或英文按钮文案
- 可随语言切换变化的 label、description、title
- 为了前端展示临时拼出来的字符串

## 数据库规则

1. 新增配置相关字段时，优先使用数据库 enum 或代码层 enum 映射后的受限字符串。
2. 如果数据库暂时无法使用原生 enum，也必须在 service/domain 层定义 enum 类型并做白名单校验。
3. 字段命名应表达语义，例如 `routing_mode`、`collaboration_mode`、`runtime_role`，不要命名为 `label`、`display_name` 后再承担业务判断。
4. 展示文案只能作为附加 metadata 或前端 i18n 资源，不得作为业务状态的唯一来源。
5. 历史 string 字段如果实际承载配置枚举，新需求触碰时必须优先收敛到 enum 校验。

## API 规则

1. 接口必须返回稳定 id/code 作为前后端契约。
2. `label`、`description`、`name` 这类展示字段只能用于兼容或管理端辅助查看，前端业务页面不得依赖它们完成国际化。
3. 前端展示应根据 id/code/runtimeRole 查本地 i18n 文案。
4. 前端提交配置时只能提交 enum/id/code，不允许提交当前语言下的展示文案。

## AI 实现要求

涉及配置、设置、状态、模式、角色、策略、类型、等级、来源、渠道等字段时，AI 必须先判断它是不是枚举语义。

如果是枚举语义：

1. 先定义允许值。
2. 再设计数据库字段和接口 DTO。
3. 再设计前端 i18n 映射。
4. 最后才写展示组件。

不得因为当前只有中文界面，就把中文文案写进数据库或接口契约。
