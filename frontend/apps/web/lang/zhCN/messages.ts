const messages = {
  web: {
    shell: {
      brand: {
        eyebrow: "Commerce AI",
        title: "运营中枢",
        description: "电商智能体经营闭环",
      },
      navigation: {
        mobileLabel: "主导航",
        workspace: "工作区",
        home: "首页",
        agent: "运营智能体",
        agentOutput: "方案展示",
        profile: "个人资料",
      },
      topbar: {
        meta: "个人工作区",
      },
      account: {
        defaultName: "用户",
        defaultId: "member@example.com",
        guestName: "未登录",
        guestId: "游客访问",
        guestFallback: "未",
        login: "登录",
        register: "注册",
        logout: "退出登录",
        actions: {
          profile: "个人资料",
        },
      },
    },
  },
  home: {
    welcome: "欢迎回来",
    publicWelcome: "欢迎",
    publicTitle: "Web 工作区",
    publicSubtitle: "公开页面在登录前后使用同一套应用布局。",
    fallbackName: "用户",
    actions: {
      profile: "编辑个人资料",
      login: "登录",
    },
    account: {
      title: "账号已就绪",
      description: "当前工作区已登录，应用内的私有信息会保留在登录状态之后。",
    },
    publicAccount: {
      title: "公开访问",
      description: "你可以浏览共享页面，账号私有信息只会在登录后展示。",
    },
    profile: {
      title: "个人资料设置",
      description: "你可以在个人资料页面修改显示名称、头像、邮箱和密码。",
    },
    publicProfile: {
      title: "个人功能",
      description: "登录后可以管理个人资料、头像、邮箱和密码。",
    },
  },
  agentChat: {
    welcomeMessage:
      "你好，我是 Commerce 运营智能体。你可以输入一个电商运营目标，我会围绕数据、预算、千川投放、创意和运营方案拆解执行，并在右侧展示进度、过程轨迹和产物。",
    streaming: "正在思考...",
    error: "Agent 回复失败，请稍后重试。",
    errorBadge: "回复失败",
    missingStructuredOutput:
      "上一轮任务未产出完整结构化结果。可以直接重新发送任务，我会重新规划执行计划，并把过程同步到经营工作区。",
    sidebar: {
      newChat: "新任务",
      search: "搜索历史记录",
      recents: "最近",
      chat: "运营智能体",
      more: "更多",
      empty: "还没有历史对话",
    },
    shell: {
      brandTitle: "运营中枢",
      brandDescription: "电商智能体经营闭环",
      topbarTitle: "输入运营目标",
      topbarMeta: "Commerce 运营智能体 · 经营闭环",
      sections: {
        create: "",
        workspace: "运营中枢",
        governance: "管理与复盘",
        recents: "历史会话",
      },
      navigation: {
        newChat: "新建会话",
        smartOps: "智能运营",
        productBoard: "商品看板",
        campaigns: "营销活动",
        platform: "平台连接",
        marketResearch: "智能洞察",
        briefing: "每日简报",
        alerts: "预警中心",
        deliverables: "交付资料",
        taskBoard: "执行清单",
        packages: "通用化管理",
        systemDashboard: "系统看板",
        team: "AI 团队介绍",
        history: "历史会话",
      },
    },
    composer: {
      placeholder: "输入运营目标，例如：五一咖啡抖音转化方案、千川投放复盘、活动预算与创意拆解...",
      loginPlaceholder: "请先登录后再调用 Agent",
      hint: "Enter 发送，Shift + Enter 换行",
      send: "发送",
      stop: "停止",
      attachTitle: "添加文件",
      agentTitle: "切换运营智能体",
      agentName: "Commerce 运营智能体",
      samples: {
        coffee: {
          label: "五一咖啡转化",
          prompt:
            "我是做咖啡运营的，主要营销平台是抖音，现在马上五一了，可以帮我写一个运营方案提升我的转化率，包含千川投放、预算、创意和执行计划",
        },
        qianchuan: {
          label: "千川投放复盘",
          prompt:
            "帮我复盘一个抖音直播间千川投放问题：点击率可以但下单转化低，请按数据诊断、预算、创意、投放和运营执行拆成处理任务",
        },
      },
      chips: {
        products: {
          label: "咖啡商品组",
          title: "后续接商品看板和商品知识",
        },
        campaign: {
          label: "五一营销活动",
          title: "后续接营销活动策划和执行跟踪",
        },
        platform: {
          label: "抖音/千川数据",
          title: "后续接平台连接、数据导入和投放数据",
        },
      },
    },
    workbench: {
      title: "经营工作区",
      expand: "展开经营工作区",
      collapse: "收起经营工作区",
      progress: "进度",
      activity: "本轮动态",
      record: "本轮记录",
      recordCountSuffix: "条",
      businessInput: "业务输入",
      preview: "工作预览",
      turns: "处理轮次",
      handoff: "业务承接",
      futureModules: "后续接真实模块",
      idle: {
        planningTitle: "正在规划执行计划",
        historyTitle: "历史会话，等待新任务",
        waitingTitle: "等待业务目标",
        planningDescription:
          "已接收输入，正在匹配商品、活动、数据和交付步骤。",
        historyDescription:
          "经营工作区只显示本轮过程，历史消息不会计入当前进度。",
        waitingDescription:
          "输入运营目标后，将按商品、活动、平台数据、预警和交付资料组织处理。",
        loopTitle: "01 | 经营闭环",
        previewTitle: "本轮任务预览",
        previewFile: "经营闭环预览",
        pending: "待执行",
        lines: [
          "数据诊断识别指标、归因和经营约束",
          "商品看板提供商品知识、卖点素材和竞品线索",
          "营销活动承接活动策划、执行节奏和复盘结果",
          "平台连接提供抖音、千川等同步状态",
          "预算评估、投放优化、创意产出并行处理",
          "运营方案汇总预算、素材、投放节奏",
          "交付资料沉淀方案、清单和复盘口径",
          "预警中心沉淀预算、投放和转化异常规则",
          "每日简报汇总重点进展和待办事项",
          "通用化管理承接后续流程启停与回滚",
        ],
      },
    },
    thread: {
      agentName: "Commerce 运营智能体",
      welcomeCapabilities: ["拆解经营目标", "组织业务执行", "沉淀交付产物"],
      receivingOutput: "正在接收业务执行输出",
      waitingEvents: "正在等待经营工作区事件",
      completedTasks: "{{completed}}/{{total}} 个任务完成",
      planning: "正在生成执行计划",
      historyNotExecuted: "历史消息未执行",
      executionTitle: "业务执行",
      statuses: {
        executing: "正在执行",
        synced: "本轮执行已同步",
        planning: "正在规划",
        waiting: "等待新一轮执行",
      },
      artifactsTitle: "阶段资料",
      artifacts: {
        updating: "随执行更新",
        preparing: "准备生成",
        pending: "待生成",
      },
      currentProgress: "当前进度 {{completed}}/{{total}}",
      actions: {
        copy: "复制",
        useful: "有用",
        useless: "无用",
      },
      agents: {
        standby: "待命",
        data: {
          name: "数据诊断",
          summary: "先识别指标、归因和经营约束",
        },
        accounting: {
          name: "预算评估",
          summary: "核算预算、SKU 和风险边界",
        },
        douyinAds: {
          name: "千川投放",
          summary: "拆解计划结构、人群包和 ROI 止损",
        },
        creative: {
          name: "创意产出",
          summary: "生成短视频、直播和活动内容方向",
        },
        ops: {
          name: "运营方案",
          summary: "整合上游结论形成执行节奏",
        },
        delivery: {
          name: "交付文档",
          summary: "沉淀方案、清单和后续产物",
        },
      },
      steps: {
        readTarget: "读取目标",
        planReady: "执行计划已生成",
        readingRules: "正在读取经营目标与业务规则",
        targetRules: "经营目标与业务规则",
        assignTask: "分配任务",
        taskRunning: "任务正在执行",
        waitingSync: "等待执行状态同步",
        checkingConstraints: "检查模型、预算与投放约束",
        enrichMaterial: "补充资料",
        enrichByTask: "按任务补充平台资料",
        enrichFallback: "补充行业与平台场景资料",
      },
      artifactSpecs: {
        dataStatus: "数据诊断、指标归因、平台场景资料",
        adsStatus: "千川计划结构、人群包、预算节奏",
        deliveryFile: "运营全流程执行方案.docx",
        deliveryStatus: "最终方案、执行清单、复盘口径",
      },
      handoff: {
        context: {
          label: "上下文",
          detail: "商品看板、平台连接、智能洞察",
        },
        execution: {
          label: "执行",
          detail: "营销活动、执行清单、工作区任务",
        },
        delivery: {
          label: "交付",
          detail: "交付资料、每日简报、复盘历史",
        },
        governance: {
          label: "治理",
          detail: "预警中心、能力包管理、审批回滚",
        },
      },
      processStage: {
        failed: "执行受阻，展开经营工作区查看待处理项",
        finished: "方案已汇总，可查看过程和业务落点",
        running: "执行计划推进中",
        planning: "正在规划执行计划",
        synced: "本轮过程已同步",
        history: "历史会话，展开经营工作区查看本轮过程",
      },
      artifactState: {
        done: "已生成",
        working: "生成中",
        failed: "受阻",
        queued: "排队",
        idle: "待生成",
      },
      inlineState: {
        done: "已完成",
        working: "处理中",
        reworking: "重试中",
        failed: "失败",
        queued: "排队中",
        idle: "待命",
      },
    },
    swarm: {
      lanes: {
        context: { label: "上下文", value: "商品/平台/洞察" },
        execution: { label: "执行", value: "活动/任务/预算" },
        delivery: { label: "交付", value: "资料/简报/复盘" },
        governance: { label: "治理", value: "预警/审批/回滚" },
      },
      agents: {
        data: {
          name: "数据诊断",
          summary: "分析数据、指标、归因和业务约束",
          objective: "分析任务中的数据、指标、归因和约束",
        },
        accounting: {
          name: "预算评估",
          summary: "评估成本、预算、SKU 和经营风险",
          objective: "基于数据诊断评估活动成本、引流 SKU、投流预算和风险",
        },
        douyinAds: {
          name: "千川投放优化",
          summary: "计划结构、人群包、预算节奏和 ROI 止损",
          objective:
            "拆解巨量千川计划结构、人群包、出价预算、ROI 止损线和直播/短视频承接",
        },
        creative: {
          name: "创意产出",
          summary: "产出短视频、直播或活动内容创意",
          objective: "基于数据诊断产出抖音短视频、直播或活动内容创意",
        },
        ops: {
          name: "运营方案",
          summary: "整合财务、千川投放和创意结果，形成可执行方案",
          objective: "整合财务、千川投放和创意结果，形成可执行运营方案",
        },
        delivery: {
          name: "交付文档",
          summary: "沉淀方案结构、执行清单和可下载产物",
          objective: "将运营方案沉淀为客户可复用文档",
        },
      },
      phases: {
        data: {
          title: "阶段一：数据诊断",
          detail: "识别指标、归因、平台场景和已知约束",
        },
        parallel: {
          title: "阶段二：并行处理",
          detail: "财务、千川投放、创意任务并行处理",
        },
        ops: {
          title: "阶段三：交叉校验",
          detail: "合并上游结论，交叉校验预算、素材和投放节奏",
        },
        delivery: {
          title: "阶段四：交付汇总",
          detail: "合成最终策略、执行清单和交付文件",
        },
        dynamicTitle: "阶段 {{index}}：{{title}}",
        dynamicDeliveryTitle: "阶段 {{index}}：交付汇总",
        dynamicDeliveryDetail: "将各任务结果汇总为可交付方案、清单和文档。",
      },
      handoffs: {
        context: {
          title: "上下文",
          detail: "商品看板、平台连接、智能洞察会作为任务输入",
        },
        execution: {
          title: "执行",
          detail: "营销活动、执行清单和工作区任务承接动作",
        },
        delivery: {
          title: "交付",
          detail: "交付资料、每日简报和复盘历史沉淀结果",
        },
        governance: {
          title: "治理",
          detail: "预警中心、能力包管理和审批回滚处理风险",
        },
        review: {
          title: "复盘",
          detail: "对话历史、执行记录和团队能力说明留作复用",
        },
      },
      statePill: {
        done: "已完成",
        working: "正在处理",
        queued: "排队",
        failed: "失败",
        reworking: "重试",
        skipped: "跳过",
        idle: "待命",
      },
      dockState: {
        done: "已完成",
        working: "正在推理",
        reworking: "正在重试",
        failed: "执行失败",
        queued: "等待执行",
        skipped: "已跳过",
        idle: "等待任务",
      },
      stage: {
        waitingInput: "等待任务输入",
        planning: "正在规划执行计划",
        historyWaiting: "历史会话，等待新任务",
        failedAgent: "执行受阻: {{agent}}",
        reworkingAgent: "正在重试: {{agent}}",
        workingAgent: "正在执行: {{agent}}",
        multiTasking: "多任务处理中",
        finished: "方案已汇总",
      },
      timeline: {
        planning: [
          "已收到任务，正在规划执行计划。",
          "处理步骤、依赖和验收规则生成后会同步到经营工作区。",
        ],
        history: [
          "当前展示的是历史对话，本轮过程尚未开始。",
          "发送新任务后会重新规划处理步骤。",
        ],
        empty: [
          "等待客户输入任务，随后会规划处理步骤。",
          "数据诊断先行，预算、投放和创意任务并行。",
          "运营方案负责交叉校验并汇总最终交付。",
        ],
        events: {
          commanderThinking: "正在分析任务并准备执行计划。",
          graphPlanned: "已规划 {{count}} 个处理步骤。",
          nodeStarted: "{{agent}} 开始第 {{attempt}} 次执行。",
          nodeRework: "{{agent}} 进入重试：{{instruction}}",
          nodePassed: "{{agent}} 已通过验收。",
          nodeFailed: "{{agent}} 失败：{{reason}}",
          graphFinished: "执行计划已结束，进入汇总输出。",
        },
      },
      cards: {
        readTarget: {
          title: "读取目标",
          planMeta: "执行计划",
          userMeta: "用户任务",
          plannedDescription: "已将任务拆成 {{count}} 个处理步骤。",
          inputDescription: "读取用户目标并准备生成执行计划。",
          waitingLine: "等待客户输入业务目标。",
        },
        assign: {
          title: "分配任务",
          meta: "等待分配",
          description: "正在准备处理步骤、依赖和验收规则。",
          currentTask: "当前任务：{{agent}}",
          stateLine: "执行状态：{{state}}",
          latestAction: "最近动作：{{action}}",
          latestPlanning: "最近动作：正在拆解任务",
        },
        output: {
          reportTitle: "任务输出",
          materialTitle: "补充资料",
          materialMeta: "补充行业与平台场景资料",
          description: "任务开始输出后，这里会切换为真实的报告摘要、证据和风险。",
          fallbackLines: ["咖啡转化", "千川投放复盘", "运营预算与投放节奏"],
        },
        rework: {
          title: "重新检查",
          description: "验收未通过，系统已触发返工指令。",
        },
        failed: {
          title: "执行受阻",
          description: "当前节点执行失败，需要检查模型服务或上游依赖。",
        },
      },
      screen: {
        acceptanceLine: "验收规则 {{index}}：{{item}}",
        responsibilityLine: "处理职责：{{summary}}",
        objectiveLine: "任务目标：{{request}}",
        reviewLine: "复核意见：{{reason}}",
        reportMode: "报告",
        taskMode: "任务",
        deliveryFilename: "运营全流程执行方案.docx",
      },
      runtimeDescriptions: {
        started: "{{agent}} 开始执行第 {{attempt}} 次尝试。",
        reported: "{{agent}} 已提交报告，等待验收或汇总。",
        passed: "{{agent}} 已通过验收。",
        failed: "{{agent}} 执行失败。",
        reworking: "{{agent}} 正在按反馈重新组织输出。",
      },
      runtimeActions: {
        commanderThinking: "正在拆解任务",
        graphPlanned: "执行计划已生成",
        nodeReady: "任务已排队",
        nodeStarted: "任务开始执行",
        nodeReported: "任务提交结果",
        nodeEvaluated: "结果已复核",
        nodeRework: "触发返工",
        nodePassed: "通过验收",
        nodeFailed: "执行受阻",
        graphFinished: "执行计划结束",
        synced: "状态已同步",
      },
      reportLines: {
        risk: "风险: {{risk}}",
        question: "待确认: {{question}}",
      },
      fallbackResponse: {
        failed: "该任务执行失败，通常是模型服务、工具或上游依赖不可用导致。",
        reworking: "验收未通过，正在按反馈重新组织任务输出。",
        working: "{{agent}} 正在处理请求，结果会在这里更新。",
        done: "{{agent}} 已完成，等待最终汇总。",
        idle: "任务开始后，这里会展示当前请求、处理结果和产物摘要。",
      },
    },
  },
  agentOutput: {
    prompt:
      "我是做咖啡运营的，主要营销平台是抖音，现在马上就五一了，可以帮我像一个运营方案吗提升我的转化率",
    sectionsNavigation: "阅读导航",
    hero: {
      badges: {
        output: "Agent 输出页面",
        category: "咖啡运营",
        platform: "抖音五一转化",
      },
      title: "五一抖音咖啡转化率提升方案",
      description:
        "将 Agent 的回复直接组织成业务页面：先看结论和指标，再看执行路径、内容脚本、财务护栏和风险回滚。",
      promptLabel: "用户输入",
      dispatchTitle: "方案协同",
      dispatchDescription:
        "主线：运营方案；协同支持：内容创意、财务测算。",
      export: "导出方案",
    },
    metrics: {
      target: {
        label: "目标转化率",
        value: "+35%",
        description: "直播间商品点击到下单转化率",
      },
      budget: {
        label: "建议预算",
        value: "1.8 万",
        description: "短视频加热 + 千川测试预算",
      },
      period: {
        label: "执行周期",
        value: "10 天",
        description: "4 月 24 日到 5 月 3 日",
      },
      guardrail: {
        label: "止损线",
        value: "ROI 1.2",
        description: "连续 2 小时低于阈值停投",
      },
    },
    plan: {
      eyebrow: "执行工作流",
      title: "从引流到复盘的 4 步路径",
      actionsTitle: "24 小时内动作",
    },
    steps: {
      sku: {
        title: "引流 SKU",
        owner: "运营",
        timing: "4 月 24 日前",
        detail:
          "上线 9.9 元 3 袋尝鲜装和 19.9 元 5 袋冷萃冻干，限购 1 份，承接五一出行和宅家咖啡场景。",
      },
      content: {
        title: "内容蓄水",
        owner: "内容",
        timing: "4 月 25-28 日",
        detail:
          "发布 10 条五一主题短视频，每条 500 元随心推加热，5 秒完播率低于 35% 的素材当天替换。",
      },
      live: {
        title: "直播承接",
        owner: "直播",
        timing: "4 月 29-5 月 3 日",
        detail:
          "每 20 分钟切一次出行、宅家、值班场景，配合满 3 件减 20、满 5 件送随行杯福利。",
      },
      traffic: {
        title: "投流放大",
        owner: "投放",
        timing: "活动全程",
        detail:
          "千川锁定 18-35 岁一二线城市通勤、旅行、美食兴趣人群，按转化率和真实 ROI 做小时级调价。",
      },
    },
    actions: {
      inventory: "今天先确认咖啡品类、客单价和库存，避免引流 SKU 拉低整体毛利。",
      benefit: "把直播间首屏福利统一成一句话：五一囤咖啡，今天拍 3 件立减 20。",
      hook: "短视频前三秒只讲场景钩子，不先讲品牌，例如“五一爬山景区咖啡 30 一杯，我带这个 3 元一杯”。",
      comment: "评论区置顶“晒出你的五一安排，抽 3 位送整月咖啡”，同时收集用户场景。",
    },
    sections: {
      summary: {
        title: "30 秒结论",
        eyebrow: "执行导向",
        content:
          "五一前不要只做泛折扣，核心是用低价尝鲜装承接抖音节日流量，再用直播间组合购提升客单。先用 10 条场景短视频测出高完播素材，再把预算集中到能带来直播间点击和下单的内容上。",
        bullets: [
          "引流款占整体 SKU 不超过 15%，避免利润被低价款吃掉。",
          "直播间主推组合购，单品尝鲜只负责拉新。",
          "每 2 小时复盘转化率、ROI、退货率，低于阈值立刻回滚。",
        ],
      },
      diagnosis: {
        title: "转化瓶颈判断",
        eyebrow: "当前假设",
        content:
          "咖啡在五一期间有明确消费场景，但用户不是被“便宜咖啡”打动，而是被“出行、宅家、值班都能马上用”的具体画面打动。抖音链路的瓶颈大概率不在曝光，而在内容钩子和直播间承接不一致。",
        bullets: [
          "短视频讲假期场景，直播间却只讲产品参数，会造成跳失。",
          "福利不够具体时，用户没有立即下单理由。",
          "没有限购和组合购设计，引流款容易变成亏损款。",
        ],
      },
      script: {
        title: "可直接复制的短视频脚本",
        eyebrow: "内容创作者补位",
        content:
          "开场：五一出去玩，景区咖啡 30 一杯，我现在包里放这个。正文：一袋冷水也能冲，车上、帐篷、酒店都能喝，平均 3 元一杯。结尾：今天直播间 9.9 元尝鲜，拍 3 件还减 20，五一前囤够。",
        bullets: [
          "画面 1：景区咖啡价签和背包咖啡形成对比。",
          "画面 2：冷水冲泡，展示 10 秒内完成。",
          "画面 3：直播间福利弹窗，强调限时限量。",
        ],
      },
      finance: {
        title: "财务护栏",
        eyebrow: "财务分析师补位",
        content:
          "9.9 元引流款必须设置单客亏损上限，不能按 GMV ROI 直接判断。建议用真实 ROI：扣除退货、佣金、商品成本、物流和赠品成本后再看投流效率。",
        bullets: [
          "投流总费用不超过预估营收的 18%。",
          "退货率超过 42% 时下调引流款曝光。",
          "随行杯赠品只给高客单组合，避免赠品吃掉毛利。",
        ],
      },
    },
  },
  login: {
    brand: "AI 标准化开发框架",
    hero: {
      titleLine1: "说出你的需求，",
      titleLine2: "让 AI 按框架写代码。",
      desc1:
        "这是一个 AI 标准化开发框架，覆盖页面、状态、路由、接口与主题约束。",
      desc2: "你只需要描述需求，AI 会基于项目规范完成实现并保持结构一致。",
    },
    welcome: "欢迎回来",
    title: "登录你的账号",
    subtitle: "请输入标识和密码后继续体验 AI 开发流程。",
    form: {
      password: {
        label: "密码",
        placeholder: "请输入密码",
      },
      submit: "登录",
      submitting: "登录中...",
      identifier: {
        label: "标识",
        placeholder: "请输入标识",
      },
    },
    footer: {
      toRegisterPrefix: "还没有账号？",
      toRegisterAction: "去注册",
    },
    error: {
      invalidCredentials: "登录失败，请检查标识和密码。",
    },
  },
  register: {
    brand: "AI 标准化开发框架",
    hero: {
      titleLine1: "创建账号，",
      titleLine2: "开始 AI 开发。",
      desc1: "这里只创建共享账号；登录当前业务时再初始化当前业务身份。",
      desc2: "同样支持主题切换、多语言和模块化结构，便于持续迭代。",
    },
    welcome: "创建新账号",
    title: "注册你的账号",
    subtitle: "创建一次共享账号即可；当前业务用户会在登录时自动初始化。",
    form: {
      password: {
        label: "密码",
        placeholder: "请输入密码",
      },
      confirmPassword: {
        label: "确认密码",
        placeholder: "请再次输入密码",
      },
      submit: "注册",
      submitting: "注册中...",
      username: {
        label: "用户名",
        placeholder: "请输入用户名",
      },
      displayName: {
        label: "显示名称（可选）",
        placeholder: "请输入显示名称",
      },
      email: {
        label: "邮箱（可选）",
        placeholder: "请输入邮箱",
      },
    },
    footer: {
      toLoginPrefix: "已有账号？",
      toLoginAction: "去登录",
    },
    error: {
      passwordMismatch: "两次输入的密码不一致。",
    },
  },
} as const

export default messages
