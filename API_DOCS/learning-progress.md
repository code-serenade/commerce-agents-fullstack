# 学习进度 API

所有 `/api/app/learning/*` 路由要求 app JWT，且只处理当前 `user_id` 的资料；所有 `/api/admin/learning/*` 路由要求 admin JWT 和 `learning:read`。

## `GET /api/app/learning/profile`

返回框架版本、覆盖率、知识点状态、分析状态与用户开关。无画像时返回一个稳定的空画像，不返回其他用户数据。

`masteryMap` 的领域项保持原 `status`、`evidenceCount`、`strength` 兼容字段。经过题库量规评价后可增加 `assessedCount`、`correctCount`、`partialCount`、`incorrectCount`、`lastAssessmentOutcome`、`evaluationStatus`（`assessed` / `developing` / `needs_attention`）及 `lastAssessedAt`。`incorrect` 不提高 `strength`。

响应新增加法兼容字段 `assessmentSummary`，不新增数据库字段。为兼容早期已经形成量规证据、但画像 JSON 尚未写入计数的记录，服务优先从现有 `student_learning_evidence.rubric_hits.outcome` 汇总确定性评价；没有评价明细时回退现有 `masteryMap` 计数：

```json
{
  "assessedKnowledgeCount": 2,
  "assessmentCount": 4,
  "distinctAssessmentQuestionCount": 3,
  "multiAngleKnowledgeCount": 1,
  "demonstratedKnowledgeCount": 1,
  "retainedKnowledgeCount": 0,
  "correctCount": 2,
  "partialCount": 1,
  "incorrectCount": 1,
  "needsAttentionKnowledgeCount": 1
}
```

`assessedKnowledgeCount` 表示至少完成过一次确定性量规评价的知识领域数；`distinctAssessmentQuestionCount` 仍表示按“领域 + 题目 ID”去重后的可评分题目数。`multiAngleKnowledgeCount` 则按版本化课程角度映射统计，只有至少两个不同 `assessmentAngleCode` 形成可评分证据才计入；同一知识角度下的两道近似题仍是两个不同题目，但只算一个角度。

`demonstratedKnowledgeCount` 使用每个课程角度的最新确定性结果，统计“至少两个不同角度最新均为 `correct/partial`、至少一个角度为 `correct`，且没有角度最新为 `incorrect`”的领域数。它表示学生已经在不同角度体现出理解，但仍不等同于课程通过、熟练或最终掌握。同一角度下后续作答会更新该角度用于派生的最新结果，不删除历史评价次数或不同题目数。

`retainedKnowledgeCount` 在上述理解门槛上继续要求不同课程角度的最新正向结果最早与最晚至少间隔 7 天。它表示系统已经获得跨时段复核证据，仍不等同于课程通过或永久掌握。新 probe 快照和 evidence 内部记录 `assessmentAngleCode`；历史 evidence 缺少该字段时按不可变题目身份从版本化角度表恢复，无法映射的旧题在所属领域保守合并为一个 `legacy_unmapped` 角度，不会凭多个未知题目虚增掌握结论。

`needsAttentionKnowledgeCount` 表示领域最新评价结果为 `incorrect`（画像回退时为 `needs_attention`）的领域数。自然 Chat 的 `observed_unscored`、历史未评分、不可判定结果及缺少题目 ID 的旧记录不计入多角度或理解体现指标。旧客户端可以忽略新增字段。

响应同时加法返回 `assessmentTimeline`，固定包含当前周在内的最近 13 个 UTC 自然周，用于在原学习进度页展示结构化评价状态随时间的变化：

```json
[
  {
    "periodStart": "2026-08-24",
    "assessedKnowledgeCount": 3,
    "demonstratedKnowledgeCount": 1,
    "retainedKnowledgeCount": 0
  }
]
```

每个点表示截至该周结束时的状态快照，不是当周答题次数。三个计数与当前 `assessmentSummary` 使用同一套同题最新结果、课程角度去重、多角度理解和 7 天保持性规则；后续错误可能使理解或保持数量下降。时间序列只使用结构化 `correct/partial/incorrect` evidence，普通 Chat 的 `observed_unscored`、历史未评分和不可判定记录不进入掌握曲线。服务端从当前用户已有的完整 evidence 集合派生，不新增数据库字段、请求参数或额外写入，也不返回题目、选择、答案键、参考答案或内部量规。旧客户端可以忽略该字段。

响应还加法返回 `domainAssessments`，固定按知识框架顺序给出 8 个领域的可解释评价。该字段从当前用户的完整 evidence 集合派生，不读取客户端分数，也不新增数据库字段：

```json
[
  {
    "knowledgeCode": "conversion",
    "statusCode": "collecting_evidence",
    "reasonCode": "single_angle_only",
    "observed": true,
    "observationCount": 2,
    "assessmentCount": 1,
    "distinctQuestionCount": 1,
    "assessmentAngleCount": 1,
    "correctCount": 1,
    "partialCount": 0,
    "incorrectCount": 0,
    "latestOutcome": "correct",
    "latestAssessedAt": "2026-09-08T08:00:00Z",
    "retentionVerified": false,
    "missingEvidenceCodes": ["another_assessment_angle", "spaced_review"],
    "nextActionCode": "answer_another_angle",
    "recentEvidence": [
      {
        "evidenceId": "uuid",
        "conversationId": "uuid",
        "questionId": "conversion.ga4_begin_checkout.choice.003",
        "questionText": "……",
        "selectedChoiceText": "……",
        "outcome": "correct",
        "createdAt": "2026-09-08T08:00:00Z"
      }
    ]
  }
]
```

`statusCode` 支持 `not_assessed` / `collecting_evidence` / `needs_attention` / `developing` / `demonstrated` / `retained`。领域状态使用每个课程角度的最新确定性结果；总答题次数仍保留历史，不能用重复作答同一角度虚增多角度结论。`reasonCode`、`missingEvidenceCodes` 和 `nextActionCode` 都是稳定 code，客户端负责本地化展示“为什么、还缺什么、下一步”。`recentEvidence` 最多返回该领域最近 3 条确定性答题摘要，并允许旧记录的题目或选择正文为 `null`；不返回正确答案、参考答案或内部量规。自然 Chat 的 `observed_unscored` 只影响 `observed/observationCount`，不计入任何答题统计或掌握状态。

## `GET /api/app/learning/evidence`

返回当前用户的学习证据摘要（仅本人数据），按时间倒序。查询参数：

- `limit`：可选。未传 `range` 时默认 10，最大 50；传入阶段范围时默认 50，最大 50。
- `range`：可选枚举，`recent` / `30d` / `90d`。省略或使用 `recent` 时保持原有“最近记录”行为；`30d`、`90d` 分别仅返回当前 UTC 时间向前 30 天、90 天内的记录。

每条包含 `id`、`knowledgeCode`、`evidenceType`（`explanation` / `application` / `correction` / `probe_answer`）、`direction`、`strength`、`summary`、`createdAt`，并加法返回 `conversationId`，用于回到产生该记录的本人历史 Chat。接口仍只按 JWT 用户查询本人 evidence，客户端不能借此读取其他学生会话。范围仅影响读取结果，不改变任何学习画像、证据或聊天记录。

新评价证据还会返回可选字段 `assessmentOutcome`、`questionId`、`rubricVersion`、`probeId`、`questionText`、`selectedChoiceId`、`selectedChoiceText`。后三项选择题字段来自当前学生 evidence 所关联的不可变 probe 快照，用于把题目和学生实际选择作为独立结构化内容展示；普通 Chat 观察、历史开放题或无法补齐的旧记录允许返回 `null`。本接口不返回 `correctChoiceId`、参考答案或内部量规答案键。

自然 Chat 观察证据的 `assessmentOutcome` 为 `observed_unscored`；题库回答为 `correct` / `partial` / `incorrect` / `unscorable`；没有量规的历史回答为 `legacy_unscored`。客户端不得通过解析 `summary` 猜测题目、选项或正误。

## `GET /api/app/learning/probes/next`

查询参数：`conversationId`、`assistantMessageId`、`continueFromProbeId`（均可选）。聊天内普通调用应同时携带前两项；只有学生在上一题反馈中主动选择“再答一题”时才传 `continueFromProbeId`，该 probe 必须属于当前用户、当前会话、状态为 `answered`，且当前助手消息是其答案消息的直接回复。返回一条可选关切问题或 `null`。一次最多返回一道，接口失败不得影响聊天。

题目投放模式只由服务端 `[learning] question_delivery_mode` 配置控制，客户端没有也不能传入投放模式字段。稳定模式如下：

- `disabled`：不创建或展示新的未答题；配置缺失时缺省使用此模式。
- `internal_pilot`：允许从当前结构、来源和内部工程审核均有效的 v10 题库中投放。
- `production`：只允许投放独立生产发布清单中与当前题库版本、题目 ID 和题目版本完全匹配的具名教研批准题。

当前生产发布清单为空，准确表示生产审核 `0/100`，所以 `production` 模式返回 `null`，不会回退到内部灰度题、LLM 临时出题或来源不明题目。模式只约束新题和未答题展示；已经回答的 probe、结果、evidence、画像及历史 Chat 继续可读，并继续按其不可变题库快照恢复，不会因模式切换而删除或重判。

当前回合存在可靠地区、平台或题目级主题信号时，服务端只从与该信号一致的可信候选中选择；候选为空时返回 `null`，不得随机回退到同一级领域的其他题。v10 已分别提供商品标题优化和卖点表达可信题，因此明确“优化标题和卖点”的回合只在这两道相关候选间稳定选择，不返回库存盘点或补货题；明确库存盘点回合仍只返回库存相关候选。

“每个 Chat 回合有题”准确指每个合格电商回合：当前登录学生开启学习追踪、部署模式允许、目标是本人当前会话中已完成的助手消息、学生原话或助手回答能可靠识别电商领域、当前细分信号存在可信可投放候选、同一消息没有重复初题且不存在已主动参与但未完成的其他题。已有正向评价、阶段性理解或普通 Chat 观察不再停止后续合格回合的初题投放。非电商闲聊、信号不足或冲突、投放关闭、生产无批准题、来源过期及接口失败仍返回 `null`，不能用无关题满足数量要求。

新普通题卡优先使用当前助手消息前一条、同会话且属于当前学生的用户消息正文选择领域；学生原话无法可靠匹配仍有评价缺口的电商领域时，才回退助手消息正文。这样普通 Chat observation 和题卡优先基于同一学生表达，避免助手解释中附带的其他经营术语把题卡带到无关领域。领域匹配同时包含退款、描述不符、核对证据和判断责任等售后强信号；这些信号会使客服售后主题优先于“商品”这类仅表示静态资料背景的弱信号，并映射到有官方来源的消费争议交易记录题。学生主动续题继续沿用上一 probe 的领域，不重新分类。普通 Chat 的 `observed_unscored` 不会让领域退出候选；不足两个不同课程角度、尚未出现完全正确结果或仍有未纠正最新错误时均属于评价缺口。达到至少两个不同角度均为正向、至少一个角度完全正确且没有最新错误后，该领域停止自动投放。阶段 S 不再用历史终态 probe 的小时冷却隐藏后续普通回合；自由切换扩展后，新普通回合按自己的助手消息独立选题，并保留其他回合尚未作答的结构化单选题。

题目选择优先补充未回答角度；已有至少两个角度且存在最新错误时优先提供错误题目的后续纠正机会。学生答题后的助手回复先展示本题反馈，不自动出下一题；结果 `canContinue=true` 时可由学生主动继续。主动续题先使用同一 Chat 冻结的完整 `contextualQuestionIds`，本轮必须先覆盖其中尚未出现的相关题，不会因中途出现错题而跳过剩余新角度。完成一轮后只在相同冻结候选池中开始下一轮，并按该题最新结构化结果依次优先 `incorrect`、`partial`、尚未评价和 `correct`；候选多于一道时避免立即重复上一题，只有一道时允许继续复习。`maxContinuationDepth=0` 表示主动续题没有固定次数上限，任何时刻仍只返回一道，不形成整套答卷。

成功返回示例：

```json
{
  "id": "uuid",
  "questionId": "traffic.funnel_diagnosis.001",
  "questionVersion": 1,
  "questionType": "single_choice",
  "choices": [
    { "id": "compare_funnel", "text": "比较漏斗各阶段并定位流失环节" },
    { "id": "increase_clicks", "text": "继续增加点击量即可" }
  ],
  "knowledgeCode": "traffic",
  "questionText": "点击量上涨但成交没有变化时，你会先排查流量的哪个方面？",
  "reasonCode": "context_assessment_gap",
  "interactionMode": "contextual_micro",
  "rewardCode": "progress_feedback",
  "expiresAt": "2026-08-25T00:00:00Z"
}
```

`reasonCode=context_assessment_gap` 表示当前 Chat 上下文命中了仍有评价缺口的领域。历史记录可能保留 `context_unobserved` 或 `core_unobserved`；客户端必须将 reason code 视为稳定 metadata，不直接作为展示文案。

阶段 S 新建的普通逐回合题使用 `context_assessment_turn`，学生主动续题使用 `student_continued_assessment`。历史 reason code 继续兼容。

阶段 T 中，已经体现理解但尚未经过时间间隔验证的领域，在最近评价满 7 天后的下一次相关 Chat 回合使用 `spaced_retention_review`。该 code 仅为稳定 metadata，客户端不得直接展示。

新问题只能来自版本化题库中 `approved` 且来源有效的题目；没有可信候选时返回 `null`。`questionType` 当前返回 `single_choice` 或兼容旧开放题的 `open_response`；单选题 `choices` 只包含公开选项 ID 和正文。旧 probe 可返回 `open_response` 与空数组。本接口绝不返回 `correctChoiceId`、参考答案、评分点、权重或错误匹配规则。

刷新或重新进入历史 Chat 时，如果目标助手消息已经存在结构化已回答 probe，且请求没有显式携带 `continueFromProbeId`，接口返回原 probe 并令 `engaged=true`，客户端据此通过结果接口恢复原反馈；不会为同一助手回合自动创建下一题。目标助手消息已经存在 `skipped`、`expired` 或 `superseded` 等终态 probe 时，普通请求返回 `null`，不能为同一消息重新创建替代题。只有学生主动选择“再答一题”后，客户端才携带 `continueFromProbeId` 请求续题。

目标助手消息存在尚未回答的 `offered` 单选 probe 时，服务端在返回前按当前配置对该回合做相关性复核。明确细分候选为空时返回 `null`；该 probe 题目不在当前可信候选内且学生尚未参与时，将旧 probe 标记为 `superseded` 并在同一次请求中按当前可信候选生成替代题，避免修正选题配置后题卡直接消失。旧 probe 的题目快照不会被改写，也不会伪造跳过或作答记录；已参与题、已回答结果恢复、历史开放题和主动续题不受此替换规则影响。

当前内部灰度题库为不可变 `ecommerce_diagnostic_v13`：运行时合成 v1-v13 历史版本，在 v12 的 86 题上增加 14 道原创单选题，共 100 题，其中 product 21 题、service 11 题，content/traffic/conversion/finance 各 12 题，market/compliance 各 10 题。配套 `ecommerce_assessment_angles_v4` 将 100 题完整映射到 67 个课程角度，单一角度最多 3 题。普通 Chat 观察和已有正向评价都不阻止下一合格电商回合继续出现 1 道相关题；已有错误仍优先提供纠正机会，未评价角度仍优先补齐，细分上下文候选与评价偏好没有交集时以本轮相关性为最高约束，不回退无关题。标题与卖点的相关候选池新增事实核验与证据应用题；连续答题仍仅使用冻结的本轮候选，周期内尽量不重复，相关候选耗尽后的复习不增加新的评价角度；价格异议表达显式命中 `service.price_objection_needs.choice.011`。优先池内仍排除学生最近一次 probe 的题目，再按助手消息 UUID 做稳定选择；同一助手消息重试得到相同题目，历史 v1-v12 probe 仍按各自快照版本回溯。

自 2026-09-09 的会话内自由切换扩展起，不同助手消息下的结构化单选 probe 可以同时保持 `offered`：学生连续 Chat 不会再把其他回合尚未作答的单选题标记为 `superseded`。普通请求只恢复或创建 `assistantMessageId` 对应的题目，同一助手消息最多有一张活动题；已答结果也按原消息恢复。学生可以先继续 Chat，之后回到原助手回复下结构化提交，答案仍只归属对应 probe。历史开放题只有显式 `engage` 后才可能消费下一条 Chat，未 engage 的并存单选题不参与自由文本回答识别。

Chat 相关题检索覆盖当前题库的 8 个领域。系统优先使用直接前置的学生消息识别领域与细分主题，学生消息没有题库信号时才使用助手回答；地区和平台范围作为硬过滤条件。同一内容命中多个主题时，只保留显式话题映射中命中分最高的已审核题目，并将候选集合固化到 probe 的 `contextualQuestionIds`。退款、描述不符、核对证据和责任判断场景会同时保留合理验货、消费投诉机制和争议交易记录三道最高分售后题；GA4、Google Trends 等精确主题仍只保留其最高分题目。如果已经识别出电商领域但题库中没有任何细分主题依据，接口返回 `null`，不再从该领域随机选择题目，也不调用 LLM 临时生成题目或答案。连续作答始终沿用这一固化候选集合，因此不会在后续题中偏离原 Chat 主题。学生主动点击“再答一题”时不设固定次数上限：服务端先遍历本轮尚未回答的冻结候选，完成一轮后只在同一候选池内开始下一轮；后续轮次优先精确复习最近答错或部分正确的题，再覆盖未评价和已答对题。内部快照使用 `practiceCycle` 与 `practicePurposeCode` 区分首次覆盖、纠错复习和持续巩固，但这些字段不返回客户端。每次回答均创建新的结构化 probe/evidence，历史结果不被覆盖，也不写入 Chat 消息；学习画像仍按课程角度最新结果去重，短时重复不能增加不同角度数量或替代至少 7 天的长期保持证据。

内部 probe 快照同时记录题库版本、来源复核版本、题目版本和量规版本，客户端不需要也不会收到来源审核内部字段。

已过期的关切问题会在下一次获取时自动标记为 `expired`，不再返回；学生在问题过期后的普通消息也不会被计作该问题的回答。

当携带 `conversationId` 或 `assistantMessageId` 时，二者必须属于当前登录学生，且助手消息必须属于该会话并且 `role=assistant`。不满足时返回 `LEARNING_REQUEST_INVALID`，不会创建 probe。

## `GET /api/app/learning/probes/history`

查询同一 Chat 助手回合最近 50 道已经回答的结构化轻题，用于刷新或重新进入会话后恢复“本轮已答题目”。需要 app JWT。

查询参数：

- `conversationId`：必填，会话 UUID。
- `assistantMessageId`：必填，该会话内助手消息 UUID。

服务端验证会话属于当前 JWT 用户、消息属于该会话且角色为 `assistant`。只返回同时匹配当前用户、助手消息和 `status=answered` 的结构化题，按作答时间从早到晚排列；最多返回最近 50 道，更早记录仍保留在学习进度中。

成功响应：

```json
{
  "code": 0,
  "message": "ok",
  "data": [
    {
      "probe": {
        "id": "probe-uuid",
        "questionId": "product.title_key_details.choice.013",
        "questionVersion": 1,
        "questionType": "single_choice",
        "choices": [{ "id": "choice-a", "text": "选项内容" }],
        "engaged": true,
        "knowledgeCode": "product",
        "questionText": "题目内容",
        "reasonCode": "student_continued_assessment",
        "interactionMode": "contextual_micro",
        "rewardCode": "progress_feedback",
        "expiresAt": "2026-09-10T08:00:00Z"
      },
      "result": {
        "id": "probe-uuid",
        "status": "answered",
        "canContinue": false,
        "outcome": "correct",
        "score": 1.0,
        "selectedChoiceText": "学生选择内容",
        "correctChoiceText": "正确选择内容",
        "answerComparisonCode": "matches_reference",
        "knowledgePoints": ["关键知识点"],
        "demonstratedPoints": ["已体现的判断条件"],
        "referenceExplanation": "题库版本对应的完整解析",
        "sourceLabels": []
      }
    }
  ]
}
```

历史项的 `canContinue` 固定为 `false`，续题入口只由当前题卡结果控制。接口不返回 `correctChoiceId`、评分 rubric、`practiceCycle`、`practicePurposeCode` 或完整内部快照。

## `POST /api/app/learning/probes/{id}/engage`

学生明确选择回答一个关切问题。无请求体。接口校验 probe 属于当前学生、状态为 `offered` 且未过期，在现有 `contextSnapshot` 写入参与时间。重复调用幂等。

成功响应：

```json
{
  "id": "uuid",
  "status": "offered"
}
```

只有成功 engage 的 probe 才可能把随后一条学生消息识别为 `probe_answer`。未 engage 的普通聊天仍按自然证据规则分析，不会自动消费关切问题。

`engage` 只用于兼容历史开放题。当前单选题不经过本接口，也不把选项写入 Chat；单选题使用独立的 `POST /api/app/learning/probes/{id}/answer-choice`，只提交稳定 `choiceId`。

## `POST /api/app/learning/probes/{id}/skip`

将当前用户自己的 `planned` 或 `offered` 问题标记为 `skipped`；重复调用幂等。跳过只结束当前题卡，不创建 Chat message、不触发 Agent/SSE、不改变路由，也不在同一助手消息下自动补发另一条题目。

## `GET /api/app/learning/probes/{id}/result`

查询当前学生自己的问题评价结果。问题未回答或后台评价尚未完成时返回当前 `status`，其余评价字段为 `null` 或空数组；只有回答完成且存在该 probe 的评价 evidence 后，才返回学生本人已提交的 `selectedChoiceId/selectedChoiceText`。回答完成后继续返回参考解释和公开来源。响应中的 `canContinue: boolean` 只表示服务端确认仍有评价需要、当前部署投放模式允许且同一 Chat 的冻结候选池中仍有可信题，不代表学生必须继续；主动续题没有固定次数上限。关闭投放或生产无批准题时该字段为 `false`，但既有结果、选择回显、解释、来源和入档记录仍正常返回。

```json
{
  "id": "uuid",
  "status": "answered",
  "outcome": "partial",
  "score": 0.5,
  "feedbackCode": "learning_probe_partial",
  "canContinue": true,
  "selectedChoiceId": "compare_funnel",
  "selectedChoiceText": "比较漏斗各阶段并定位流失环节",
  "demonstratedPoints": ["check_traffic_quality"],
  "nextPoint": "check_checkout_friction",
  "referenceExplanation": "先统一统计口径，再沿电商漏斗定位流失环节。",
  "sourceLabels": [
    {
      "sourceId": "google.analytics.ga4.ecommerce",
      "title": "Measure ecommerce",
      "publisher": "Google Analytics",
      "url": "https://developers.google.com/analytics/devguides/collection/ga4/ecommerce"
    }
  ]
}
```

`outcome` 支持 `correct` / `partial` / `incorrect` / `unscorable` / `legacy_unscored`。`selectedChoiceId/selectedChoiceText` 只表示该学生已经提交并入档的选项，不是答案键；优先来自 evidence，历史记录缺少选择正文时才回退同一 probe 的不可变快照和版本化题库。未回答、开放题或无法恢复的历史记录返回 `null`。接口不会返回 `correctChoiceId`、正确选项标记、内部同义词、权重和冲突匹配词。非本人访问返回 `LEARNING_PROBE_FORBIDDEN`。

## `PATCH /api/app/learning/settings`

请求：`{ "enabled": boolean }`。关闭后停止新增分析和追问，不影响既有聊天或历史会话。

## 管理端

以下路由要求 admin JWT 且权限 `learning:read`。均不返回学生完整聊天正文。

### `GET /api/admin/learning/students`

分页学生概览。查询参数（camelCase）：

- `page`：页码，从 1 开始，默认 1
- `pageSize`：每页数量，默认 20
- `keyword`：按 user id、display id、姓名、学校、队伍模糊匹配
- `sortBy`：`coverageRate` / `observedKnowledgeCount` / `lastAnalyzedAt` / `updatedAt`
- `sortOrder`：`asc` / `desc`

成功响应 `data` 结构：

```json
{
  "items": [
    {
      "userId": "uuid",
      "displayId": "id",
      "displayName": "name",
      "schoolName": "",
      "teamName": "",
      "coverageRate": 0.0,
      "observedKnowledgeCount": 0,
      "masteryStatus": "unobserved",
      "assessmentSummary": {
        "assessedKnowledgeCount": 0,
        "assessmentCount": 0,
        "distinctAssessmentQuestionCount": 0,
        "multiAngleKnowledgeCount": 0,
        "demonstratedKnowledgeCount": 0,
        "retainedKnowledgeCount": 0,
        "correctCount": 0,
        "partialCount": 0,
        "incorrectCount": 0,
        "needsAttentionKnowledgeCount": 0
      },
      "enabled": true,
      "lastAnalyzedAt": null,
      "createdAt": "2026-08-19T00:00:00Z",
      "updatedAt": "2026-08-19T00:00:00Z"
    }
  ],
  "page": 1,
  "pageSize": 20,
  "total": 1,
  "totalPages": 1,
  "hasNext": false,
  "hasPrev": false
}
```

无画像的学生也返回一行，字段取稳定空值，不返回其他用户数据。

`masteryStatus` 是旧客户端兼容字段，可能只表示已观察；新客户端必须使用 `assessmentSummary` 区分普通 Chat 观察和结构化答题评价，不得由 `coverageRate` 或 `observedKnowledgeCount` 推断已经掌握。

### `GET /api/admin/learning/students/metrics`

聚合指标，供管理端 MetricCards 使用：

```json
{
  "total": 10,
  "enabled": 8,
  "observed": 3,
  "avgCoverageRate": 0.12,
  "assessed": 2,
  "demonstrated": 1,
  "retained": 0,
  "needsAttention": 1,
  "probePilot": {
    "questionBankVersion": "ecommerce_diagnostic_v13",
    "questionCount": 100,
    "assessmentAngleCount": 67,
    "singleQuestionAngleCount": 44,
    "productionApprovedCount": 0,
    "curriculumReviewPendingCount": 100,
    "earliestSourceValidThrough": "2027-02-27",
    "issuedCount": 18,
    "activeCount": 2,
    "answeredCount": 10,
    "skippedCount": 3,
    "expiredCount": 2,
    "supersededCount": 1,
    "continuationCount": 2,
    "correctCount": 7,
    "partialCount": 0,
    "incorrectCount": 3,
    "unscorableCount": 0,
    "answerRate": 0.6667,
    "skipRate": 0.2,
    "correctRate": 0.7,
    "averageAnswerSeconds": 42,
    "coverageGapCount": 3,
    "coverageGapTurnCount": 3,
    "coverageGapDomainMetrics": [
      { "code": "product", "count": 2 },
      { "code": "traffic", "count": 1 }
    ],
    "coverageGapReasonMetrics": [
      { "code": "no_contextual_question", "count": 2 },
      { "code": "no_deliverable_question", "count": 1 }
    ],
    "questionMetrics": [
      {
        "questionId": "conversion.ga4_begin_checkout.choice.003",
        "questionVersion": 1,
        "knowledgeCode": "conversion",
        "questionText": "用户进入结账流程时，GA4 推荐使用哪个电商事件？",
        "jurisdictionCode": "not_region_specific",
        "platformCode": "google_analytics_4",
        "businessStageCode": "conversion_optimization",
        "sourcePublishers": ["Google for Developers"],
        "sourceValidThrough": "2027-02-27",
        "curriculumReviewRequired": true,
        "productionApproved": false,
        "issuedCount": 3,
        "activeCount": 0,
        "answeredCount": 2,
        "skippedCount": 1,
        "expiredCount": 0,
        "supersededCount": 0,
        "correctCount": 1,
        "partialCount": 0,
        "incorrectCount": 1,
        "unscorableCount": 0,
        "answerRate": 0.6667,
        "skipRate": 0.3333,
        "correctRate": 0.5,
        "averageAnswerSeconds": 42,
        "choiceMetrics": [
          {
            "choiceId": "begin_checkout",
            "choiceText": "begin_checkout",
            "selectedCount": 1,
            "selectedRate": 0.5
          },
          {
            "choiceId": "purchase",
            "choiceText": "purchase",
            "selectedCount": 1,
            "selectedRate": 0.5
          }
        ],
        "unmappedChoiceCount": 0
      }
    ]
  }
}
```

`observed` 只表示存在普通 Chat 或其他观察记录；其余四项按学生去重，分别表示已有结构化答题评价、已在至少两个题目上体现理解、已满足跨时段验证以及存在最新待纠正结果。观察记录不计入这四项。

`probePilot` 只统计 `answerMode=structured_choice` 的轻题，历史开放题不计入。`issuedCount` 表示服务端已经创建并返回的题卡，不等同于浏览器像素级曝光。`answerRate` 与 `skipRate` 的分母为已作答、已跳过和已过期三类已结束题卡；活动中和已替换题卡不进入分母。`correctRate` 的分母只包含 `correct/partial/incorrect` 三类可判定 evidence，`unscorable` 不进入正确率。超过 `expiresAt` 但尚未写回过期状态的题卡在本次只读聚合中按过期计算，不修改历史数据。

`coverageGapCount` 统计已识别电商领域但没有可信可投放相关题目的记录数，`coverageGapTurnCount` 按助手消息去重统计涉及回合数。`coverageGapDomainMetrics` 和 `coverageGapReasonMetrics` 分别按知识领域与失败关闭原因降序聚合；原因当前包括 `no_contextual_question` 和 `no_deliverable_question`。缺口只用于题库治理，不是学生能力证据，不改变画像、掌握度、覆盖率或答题正确率。接口不返回缺口对应的用户、会话、消息或 Chat 正文；旧客户端可忽略这些加法字段。

`questionBankVersion` 和 `questionCount` 标识当前运行时合成题库；`assessmentAngleCount` 是当前题库映射到的不同课程角度数，`singleQuestionAngleCount` 是其中只映射一道当前题目的角度数。当前 v13 固定为 67 个角度、44 个单题角度；这两个字段只表达题库覆盖深度，不自动评价题目质量，也不返回逐题角度码。`productionApprovedCount` 只统计通过独立生产发布清单校验的当前题目，不根据 `approved_for_internal_pilot` 推断。`curriculumReviewPendingCount` 统计仍要求公司教研审核且尚未生产批准的题目；`earliestSourceValidThrough` 是当前题库全部来源中最早的有效截止日期。当前 v13 发布清单为空，因此固定事实为生产批准 0、待教研 100；这不影响内部灰度模式，但生产模式继续不投放任何题目。

`questionMetrics` 固定按当前 `ecommerce_diagnostic_v13` 合成题库返回 100 行，包括零投放题。逐题状态、结果、比率和有效响应时长沿用 `probePilot` 相同口径，只归属 `questionId + questionVersion` 与当前题库完全匹配的结构化 probe；未知历史题仍保留在整体指标中，但不伪造为当前题目。`sourcePublishers` 为去重后的来源机构，`sourceValidThrough` 为该题全部来源中最早的有效截止日期；`curriculumReviewRequired` 表示题目要求公司教研审核，`productionApproved` 只来自校验后的独立生产清单。

`jurisdictionCode/platformCode/businessStageCode` 来自与 v9 一一绑定并通过 Service 完整性门禁的适用范围目录，只返回稳定 code，不返回依赖语言的展示文案。地区允许 `not_region_specific/cn/eu/us`；平台允许 `platform_agnostic/google_trends/google_merchant_center/google_analytics_4/google_search/google_ads`；业务阶段允许 `market_research/product_operations/content_operations/traffic_acquisition/conversion_optimization/financial_management/customer_service/compliance_governance`。这些字段用于管理端题目治理，不在本阶段改变学生选题，也不能把 `not_region_specific` 解释为适用于所有司法辖区。

每行 `choiceMetrics` 固定按当前题库选项顺序返回公开 `choiceId`、`choiceText`、`selectedCount` 和 `selectedRate`，零样本题也返回完整选项零值。选择身份优先读取结构化 evidence 的 `rubric_hits.selectedChoiceId`，缺失时只回退同一 probe 的不可变 `contextSnapshot.selectedChoiceId`。`selectedRate` 的分母是能够映射到当前选项的有效选择总数；缺失或未知 ID 单独计入 `unmappedChoiceCount`，不得伪造到某个选项。该分布只用于观察干扰项表现，不自动形成题目优劣、难度或区分度结论。

该数组不返回 `correctChoiceId`、`isCorrect`、参考答案、量规、课程角度码、审核人详情或学生身份。选项顺序、次数、占比和适用范围 code 不能解释为答案提示；旧客户端可忽略全部加法字段。

阶段 AO 当时基于 v8/64 题完成工程验收：零样本返回完整选项零值，未知选择只进入 `unmappedChoiceCount`，旧响应缺少新增字段时前端安全回退；管理端桌面与 390×844 仅在原 DataTable 内横向滚动，学生 Chat 结构化选择仍只调用独立 `answer-choice`，不创建 conversation message、不触发 Agent/SSE 且 composer 保持为空。该段 `0/64` 是阶段 AO 的历史验收口径，不是当前题库状态。

当前 v13 在不改变本接口契约的前提下扩为 100 题。新增 14 题依据实际核对的 Google Search Central 内容可靠性指南、GA4 官方电商事件文档及 Australian Government 定价指南原创编写，区分通用信息可靠性与平台专用规则；旧题与答案继续按原快照恢复。当前公司教研审核与生产批准均为 0/100，发布清单仍为 `approvals=[]`，真实班级阶段 I 仍待执行。

### `GET /api/admin/learning/students/{userId}`

单个学生可解释的掌握状态与证据摘要。无画像时返回稳定空结构，不报 404。

```json
{
  "userId": "uuid",
  "displayId": "id",
  "displayName": "name",
  "schoolName": "",
  "teamName": "",
  "coverageRate": 0.0,
  "observedKnowledgeCount": 0,
  "masteryStatus": "unobserved",
  "assessmentSummary": {
    "assessedKnowledgeCount": 0,
    "assessmentCount": 0,
    "distinctAssessmentQuestionCount": 0,
    "multiAngleKnowledgeCount": 0,
    "demonstratedKnowledgeCount": 0,
    "retainedKnowledgeCount": 0,
    "correctCount": 0,
    "partialCount": 0,
    "incorrectCount": 0,
    "needsAttentionKnowledgeCount": 0
  },
  "enabled": true,
  "lastAnalyzedAt": null,
  "masterySummary": [
    {
      "knowledgeCode": "market",
      "status": "unobserved",
      "strength": null
    }
  ],
  "evidenceSummary": [
    {
      "id": "uuid",
      "knowledgeCode": "market",
      "evidenceType": "explanation",
      "direction": "positive",
      "strength": 0.6,
      "summary": "…",
      "probeId": null,
      "questionText": null,
      "selectedChoiceId": null,
      "selectedChoiceText": null,
      "assessmentOutcome": "observed_unscored",
      "createdAt": "2026-08-19T00:00:00Z"
    }
  ]
}
```

`masterySummary` 固定按 `ecommerce-knowledge-framework.v1.json` 的 8 个领域返回；`status` 支持 `unobserved` / `observed` 及画像中写入的状态值；`strength` 仅当画像中存在时返回。

`assessmentSummary` 与学生端使用同一聚合口径。结构化 evidence 的 `questionText` 与 `selectedChoiceText` 用于管理端明确展示题目和学生实际选择；普通 Chat 观察和无法补齐的历史记录允许为 `null`。管理端仍不返回正确答案、参考答案和内部量规。

## `POST /api/app/learning/probes/{probeId}/answer-choice`

在原 Chat 助手回复下的轻题卡内提交单选答案。要求 app JWT；服务端以 JWT UUID 校验 probe 归属，不接受客户端提供用户、领域、题目、分数或答案键。

请求：

```json
{
  "choiceId": "stable-choice-id"
}
```

成功响应复用 `LearningProbeResultResponse`，包含 `id/status/canContinue/outcome/score/feedbackCode/selectedChoiceId/selectedChoiceText/demonstratedPoints/nextPoint/referenceExplanation/sourceLabels`，并加法返回 `correctChoiceText`、`answerComparisonCode` 和 `knowledgePoints`。

- 仅支持 `questionType=single_choice` 且未过期的 active/offered probe。
- `choiceId` 必须存在于该 probe 的题目快照；正确答案仅由服务端版本化题库和快照确定。
- `selectedChoiceId/selectedChoiceText` 只在首次选择已成功入档后回显学生自己的提交；响应仍不返回 `correctChoiceId` 或正确选项标记。
- `correctChoiceText` 只在本人 probe 已回答、且服务端能从该 probe 的不可变题库版本恢复单选题答案时返回；未答题查询不会泄露答案键。
- `answerComparisonCode` 为 `matches_reference` / `differs_from_reference` / `not_available`，供客户端解释学生选择与参考答案的关系；`knowledgePoints` 来自该题 rubric criteria 的课程概念并去重。客户端不得自行推断或改写正确答案。
- `referenceExplanation` 保留版本化题库 `referenceAnswer` 原文，并由服务端确定性补充 rubric 核心判断条件和本次选择对照：答错时指出误选没有完整覆盖的条件，答对时说明迁移判断方法。客户端应将学生选择、正确选择、参考原理、核心知识点和实践复盘分层展示，而不是把它们拼成一行短提示；客户端不得调用 LLM 临时改写答案解析。
- 首次提交形成一条与 `probeId` 唯一关联的 evidence；结构化答案不创建 conversation message，也不触发普通 Chat、多智能体或 SSE。
- 重复请求（包括两个不同 `choiceId` 的并发请求）幂等返回首次已提交结果，不能用第二个 choice 覆盖首次答案；并发竞争不会产生第二条 evidence 或第二次画像更新。
- 旧 `engage`、普通 Chat 回答和开放题路径保持兼容。
