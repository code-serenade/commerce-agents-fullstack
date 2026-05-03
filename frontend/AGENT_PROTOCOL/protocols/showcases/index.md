# showcases 协议

`showcases/*` 是展示与参考层。

这里不是业务应用层，而是给 AI 和开发者提供高质量合法样例的地方。

## 1. 宪章

- showcase 负责展示推荐用法、边界样例和组件组合方式。
- showcase 不是业务流程承载层。
- showcase 不得为了演示方便绕过 packages 分层。
- showcase 中的代码默认会被 AI 当作模板，因此规范优先级高于速度。

## 2. 默认分层规则

- primitive 能力：`@workspace/ui-core`
- 稳定基础协议：`@workspace/ui-components`
- 共享应用协议：`@workspace/app-kit`
- 展示站点私有内容：对应 `showcases/*`

如果展示代码已经稳定到足以成为共享规范，应优先回收到对应 packages，而不是长期堆在 showcase 里。

## 3. AI 在 showcase 层的默认行为

AI 在 showcase 中开发内容时，必须按以下顺序：

1. 先判断展示目标属于哪一层。
2. 优先展示 `ui-components` 与 `@workspace/app-kit` 的合法消费方式。
3. 只有在确实属于 showcase 私有表达时，才新增本地实现。
4. 如果展示中暴露出共享层缺口，应优先回到对应 package 补能力。

## 4. showcase 层禁止事项

- 禁止把 showcase 当成业务 app 承载真实业务流程。
- 禁止在 showcase 中沉淀长期脏实验。
- 禁止为了演示方便复制共享组件内部实现。
- 禁止创造会误导 AI 的临时写法或违规样例。

## 5. 本目录下 showcase 的角色

- `guide`：示例与参考站点

## 6. 本地协议要求

- 每个 showcase 都必须有自己的 `AGENT_PROTOCOL/protocols/showcases/<showcase>.md`。
- 顶层 `AGENT_PROTOCOL/protocols/showcases/index.md` 负责统一展示层总原则。
- AI 在 showcase 中写页面时，除了读取 showcase 本地协议，还应判断是否存在对应 package 协议需要一起遵守。

## 7. 完成定义

只有在以下条件同时满足时，showcase 改动才算完成：

- 分层判断正确。
- 展示方式没有误导 AI。
- 没有把共享能力错误地下沉到 showcase。
- 对应 showcase 的本地门禁通过。
