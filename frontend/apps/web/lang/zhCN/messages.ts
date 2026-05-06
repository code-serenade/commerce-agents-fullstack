const messages = {
  web: {
    shell: {
      brand: {
        eyebrow: "Web App",
        title: "Web",
        description: "用户工作台",
      },
      navigation: {
        mobileLabel: "主导航",
        workspace: "工作区",
        home: "首页",
        agent: "Agent",
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
    welcomeMessage: "你好，我是当前项目里的 Agent。你可以直接输入问题，我会用流式回复把结果显示在这里。",
    streaming: "正在思考...",
    error: "Agent 回复失败，请稍后重试。",
    errorBadge: "回复失败",
    activity: {
      title: "执行过程",
      kinds: {
        session: "会话",
        memory: "记忆",
        commander: "指挥",
        worker: "智能体",
        collaboration: "协作",
        task_graph: "任务图",
        task_node: "任务",
        output: "输出",
        error: "错误",
        other: "事件",
      },
      events: {
        started: "会话开始",
        memoryLoaded: "加载记忆",
        memoryPersisted: "保存记忆",
        memoryCompacted: "压缩记忆",
        commanderThinking: "指挥智能体思考",
        workerStarted: "智能体开始执行",
        workerFinished: "智能体完成执行",
        collaborationStarted: "协作开始",
        collaborationWorkerStarted: "协作智能体开始",
        collaborationWorkerFinished: "协作智能体完成",
        collaborationFinished: "协作完成",
        taskGraphPlanned: "规划任务图",
        taskNodeReady: "任务就绪",
        taskNodeStarted: "任务开始",
        taskNodeReported: "任务提交报告",
        taskNodeEvaluated: "任务评估完成",
        taskNodeReworkRequested: "任务要求返工",
        taskNodePassed: "任务通过",
        taskNodeFailed: "任务失败",
        taskGraphFinished: "任务图完成",
        output: "生成最终输出",
        failed: "执行失败",
        finished: "会话完成",
        error: "流式错误",
        done: "流式结束",
        other: "智能体事件",
      },
    },
    sidebar: {
      newChat: "新对话",
      search: "搜索历史记录",
      recents: "最近",
      chat: "聊天",
      more: "更多",
      empty: "还没有历史对话",
    },
    composer: {
      placeholder: "给 Agent 发送消息...",
      loginPlaceholder: "请先登录后再调用 Agent",
      hint: "Enter 发送，Shift + Enter 换行",
      send: "发送",
      stop: "停止",
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
      dispatchTitle: "智能调度",
      dispatchDescription:
        "主 Agent：运营专家；协作角色：内容创作者、财务分析师。",
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
