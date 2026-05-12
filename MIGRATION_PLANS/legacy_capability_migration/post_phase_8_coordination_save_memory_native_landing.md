# Post 8-24：Coordination 保存工作记忆真实落地

## 1. 本次完成内容

本轮把旧产品里的“保存工作记忆”能力，按新产品架构真实迁进来了。

完成点：

- `coordination_save_memory` 从骨架改为 Rust native deterministic skill
- 自然问法补强：`记住 / 记下来 / 保存这个结论 / 保存这个事实 / 记下这个决策 / 记住这个风险`
- SSE 在 `skill_result` 后调用 service，将记忆草案沉淀为真实 `agent_learning_records`
- 新增 `agent_learning_record_created` 事件，前端后续可直接消费
- 学习上下文加载改为优先读取当前 workspace 的记忆，再回退到全局 agent 学习记录

## 2. 架构决策

没有新增第二套 `workspace_memory` 表。

原因：

- 新产品已经有 `agent_learning_records`、`quality/trust/learning`、SSE 和 workspace 归属校验链路
- 当前阶段先复用这套后端基础，更符合“新产品架构 = 唯一落地框架”
- 这样能更快把旧产品 memory 能力变成真实可用后端能力

## 3. 落地语义

这次不是简单 append 一条日志，而是保留了旧产品更重要的语义：

- 按 `workspace + memoryKey` 做 upsert
- 同一个 key 再保存时更新旧记录
- 记忆类型继续区分：
  - `fact`
  - `decision`
  - `insight`
  - `warning`

## 4. 用户可感知变化

用户在问答里说：

- “记住这个结论”
- “把这个风险记下来”
- “保存这个定价决策”

现在会产生三类真实后端变化：

1. skill 会返回成功，不再是 `pending_approval`
2. 当前 workspace 下会真实新增或更新一条记忆记录
3. SSE 会发出 `agent_learning_record_created`

后续同工作区继续提问时，这些记忆会优先进入后端学习上下文。

## 5. 仍未完成的部分

本轮没有做：

- 独立的 workspace memory 管理页面
- 长期记忆冲突治理
- 记忆修剪策略
- 跨会话 profile memory
- 前端右侧专门的 memory 面板

这些都属于后续更完整的记忆治理阶段。
