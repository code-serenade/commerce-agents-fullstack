const messages = {
  admin: {
    navigation: {
      platform: "平台",
      dashboard: "控制台",
      accountManagement: "账号管理",
      adminUsers: "后台账号",
      appUsers: "产品用户",
      accessControl: "权限中心",
      roles: "权限角色",
      rolePermissions: "角色授权",
      appRoles: "用户角色",
      appRolePermissions: "用户角色授权",
      analytics: "数据统计",
      accessAnalytics: "访问统计",
      learning: "学习评价",
      usageManagement: "用量管理",
      teamUsage: "账号用量",
    },
    loading: {
      app: "正在进入电商多智能体管理后台",
      route: "正在准备后台页面",
    },
    authLayout: {
      eyebrow: "Workspace Admin",
      title: "管理台入口",
    },
    shell: {
      brand: {
        eyebrow: "Workspace",
        title: "管理后台",
        description: "企业版",
      },
      navigation: {
        mobileLabel: "主导航",
      },
      notifications: {
        label: "通知",
        title: "通知",
        empty: "暂时没有新的通知。",
        actions: {
          markAllRead: "全部标记为已读",
          markRead: "标记已读",
          dismiss: "忽略通知",
        },
        items: {
          weeklyReports: {
            title: "收到了 14 份新周报",
            description: "请在下午同步前查看最新的大区摘要和重点变更。",
            time: "3 小时前",
          },
          reply: {
            title: "朱偏右 回复了你",
            description: "权限审核备注里已经补充了发布时间窗口和负责人确认。",
            time: "刚刚",
          },
          comment: {
            title: "曲丽丽 评论了你",
            description: "她在确认控制台原型是否继续保留现在这组图表分区。",
            time: "2024-01-01",
          },
          taskReminder: {
            title: "待办提醒",
            description: "财务导出检查单还在等待你的最终确认。",
            time: "1 天前",
          },
        },
      },
      workspaceTabs: {
        clear: "仅保留当前页",
      },
      account: {
        fallbackName: "后台账号",
        fallbackInitial: "管",
        fallbackId: "workspace-admin@example.com",
        logout: "退出登录",
        actions: {
          profile: "个人资料",
        },
      },
    },
    dashboard: {
      heading: {
        title: "控制台",
        description:
          "当前展示 {{from}} 至 {{to}} 的产品访问和 AI 使用概览，数据状态：{{status}}。",
        loading: "正在加载",
        ready: "已更新",
      },
      metrics: {
        pv: {
          label: "产品访问量",
          tail: "本期共 {{sessions}} 次访问会话。",
        },
        questions: {
          label: "AI 提问次数",
          tail: "本期约消耗 {{tokens}} Token。",
        },
        accounts: {
          label: "产品用户数",
          tail: "其中 {{enabled}} 个账号处于启用状态。",
        },
        teams: {
          label: "学校/队伍",
          tail: "本期 {{active}} 个账号有访问或问答行为。",
        },
      },
      charts: {
        trafficTrend: {
          title: "每日访问与提问趋势",
          description: "按天查看产品访问量、访客数和 AI 提问次数。",
        },
        productMix: {
          title: "产品使用概况",
          description: "帮助管理人员快速判断用户是否真的在使用产品。",
        },
        series: {
          pv: "访问量",
          uv: "访客数",
          questions: "AI 提问",
        },
      },
      productMix: {
        activeAccounts: "活跃账号",
        avgSession: "平均会话时长",
        bounceRate: "跳出率",
        errors: "前端/API 错误",
      },
      topPages: {
        title: "热门功能页面",
        description: "展示管理人员最需要关注的产品页面访问情况。",
        columns: {
          page: "页面说明",
          path: "原始路径",
          pv: "访问量",
          uv: "访客数",
        },
      },
      schoolTeams: {
        title: "学校/队伍活跃情况",
        description: "按学校和队伍查看本期访问量、活跃账号和 AI 提问。",
        rowDetail: "{{accounts}} 个活跃账号，{{questions}} 次 AI 提问",
        empty: "暂无学校或队伍访问数据。",
      },
      usageAlerts: {
        title: "账号问答额度",
        description: "优先展示剩余额度较少的账号，便于管理人员提前处理。",
        columns: {
          account: "账号",
          schoolTeam: "学校/队伍",
          used: "已提问",
          remaining: "剩余额度",
        },
      },
      dataStatus: {
        title: "数据接入状态",
        description: "显示控制台当前读取了哪些真实后台数据。",
        columns: {
          name: "数据模块",
          status: "状态",
          detail: "说明",
        },
      },
      statusRows: {
        ready: "已读取",
        loading: "读取中",
        analytics: {
          name: "访问统计",
          detail: "来自产品访问、来源、页面和每日流量接口。",
        },
        accounts: {
          name: "账号统计",
          detail: "来自产品用户账号和启停状态统计接口。",
        },
        usage: {
          name: "学校/队伍与额度",
          detail: "来自学校队伍访问统计和账号问答额度接口。",
        },
      },
      empty: {
        schoolTeam: "未填写学校/队伍",
      },
    },
    accounts: {
      adminUsers: {
        data: {
          fallbackDisplayName: "未设置显示名称",
        },
        metrics: {
          all: {
            label: "后台账号总数",
            tail: "当前已注册为后台可登录账号的后台用户数量。",
          },
          enabled: {
            label: "启用账号",
            tail: "具备正常后台访问能力的账号。",
          },
          disabled: {
            label: "停用账号",
            tail: "已被禁用，需要人工恢复或复核。",
          },
          multiRole: {
            label: "多角色账号",
            tail: "同时挂载多个角色的重点账号。",
          },
        },
        table: {
          columns: {
            id: "ID",
            displayName: "显示名称",
            schoolName: "学校",
            teamName: "队伍",
            remark: "备注",
            status: "状态",
            roles: "角色",
            roleCount: "角色数",
          },
          empty: {
            schoolName: "未填写学校",
            teamName: "未填写队伍",
            remark: "无备注",
            roles: "未分配角色",
          },
          status: {
            enabled: "启用",
            disabled: "停用",
          },
          query: {
            keyword: {
              label: "关键字",
              placeholder: "搜索显示名称、ID、备注或角色",
            },
            status: {
              label: "状态",
              placeholder: "状态",
            },
          },
          actions: {
            edit: {
              label: "编辑",
              title: "编辑后台账号 {{id}}",
              description: "当前可编辑备注，保存后会同步到服务端。",
              success: "后台账号已更新",
            },
            delete: {
              label: "删除",
              title: "删除后台账号 {{id}}",
              description:
                "确认删除后台账号 {{name}}（{{id}}）？删除后该账号将无法进入后台。",
              confirm: "删除",
              success: "后台账号已删除",
            },
            toggle: {
              enable: "启用",
              disable: "停用",
              enabledSuccess: "后台账号已启用",
              disabledSuccess: "后台账号已停用",
            },
            editRoles: "编辑角色",
          },
        },
        create: {
          label: "新增后台账号",
          title: "创建后台账号",
          description:
            "支持两种方式：直接将已有 auth 账号加入后台，或先注册新账号，再立即创建后台账号。",
          mode: {
            label: "创建方式",
            existing: "已有账号转后台账号",
            new: "新建账号并创建后台账号",
          },
          fields: {
            account: {
              label: "账号",
              placeholder: "输入用户名、邮箱或展示 ID",
            },
            username: {
              label: "用户名",
              placeholder: "输入新用户名",
            },
            password: {
              label: "密码",
              placeholder: "8-128位，字母/数字/符号至少2种",
            },
            displayName: {
              label: "显示名称",
              placeholder: "输入显示名称",
            },
            email: {
              label: "邮箱（可选）",
              placeholder: "输入邮箱",
            },
            remark: {
              label: "备注",
              placeholder: "输入备注（可选）",
            },
          },
          errors: {
            identifierRequired: "请输入账号。",
            usernameRequired: "请输入用户名。",
            passwordRequired: "请输入密码。",
            displayNameRequired: "请输入显示名称。",
          },
        },
        edit: {
          fields: {
            id: "ID",
            displayName: "显示名称",
            schoolName: "学校",
            teamName: "队伍",
            remark: "备注",
          },
          remarkPlaceholder: "输入备注",
        },
        rolesDialog: {
          title: "编辑角色{{name}}",
          id: "ID {{id}}",
          description: "这里只修改用户角色，不会影响用户信息。",
          loading: "角色加载中。",
          empty: "暂无可配置角色。",
          selected: "已选择",
          cancel: "取消",
          reset: "重置",
          save: "保存角色",
          success: "用户角色已更新",
        },
      },
      appUsers: {
        data: {
          fallbackDisplayName: "未设置显示名称",
        },
        metrics: {
          all: {
            label: "产品用户总数",
            tail: "当前已注册并可使用产品的用户账号总量。",
          },
          enabled: {
            label: "启用账号",
            tail: "具备正常产品访问能力的账号。",
          },
          disabled: {
            label: "停用账号",
            tail: "已被禁用，需要人工恢复或复核。",
          },
          multiRole: {
            label: "多角色账号",
            tail: "同时挂载多个角色的重点账号。",
          },
        },
        table: {
          columns: {
            id: "ID",
            displayName: "显示名称",
            schoolName: "学校",
            teamName: "队伍",
            remark: "备注",
            status: "状态",
            roles: "角色",
            roleCount: "角色数",
          },
          empty: {
            schoolName: "未填写学校",
            teamName: "未填写队伍",
            remark: "无备注",
            roles: "未分配角色",
          },
          status: {
            enabled: "启用",
            disabled: "停用",
          },
          query: {
            keyword: {
              label: "关键字",
              placeholder: "搜索显示名称、ID、备注或角色",
            },
            status: {
              label: "状态",
              placeholder: "状态",
            },
          },
          actions: {
            edit: {
              label: "编辑",
              title: "编辑产品用户 {{id}}",
              description: "当前可编辑备注，保存后会同步到服务端。",
              success: "产品用户已更新",
            },
            delete: {
              label: "删除",
              title: "删除产品用户 {{id}}",
              description:
                "确认删除产品用户 {{name}}（{{id}}）？删除后该账号将无法使用产品功能。",
              confirm: "删除",
              success: "产品用户已删除",
            },
            toggle: {
              enable: "启用",
              disable: "停用",
              enabledSuccess: "产品用户已启用",
              disabledSuccess: "产品用户已停用",
            },
            editRoles: "编辑角色",
          },
        },
        edit: {
          fields: {
            id: "ID",
            displayName: "显示名称",
            schoolName: "学校",
            teamName: "队伍",
            remark: "备注",
          },
          remarkPlaceholder: "输入备注",
        },
        rolesDialog: {
          title: "编辑角色{{name}}",
          id: "ID {{id}}",
          description: "这里只修改用户角色，不会影响用户信息。",
          loading: "角色加载中。",
          empty: "暂无可配置角色。",
          selected: "已选择",
          cancel: "取消",
          reset: "重置",
          save: "保存角色",
          success: "用户角色已更新",
        },
      },
    },
    usage: {
      teamUsage: {
        metrics: {
          all: {
            label: "账号总数",
            tail: "服务端分页查询返回的全部账号数量。",
          },
          questionUsed: {
            label: "当前页问答数",
            tail: "当前页账号已记录的问答消耗总和。",
          },
          exhausted: {
            label: "当前页已耗尽",
            tail: "当前页剩余额度为 0 的启用账号。",
          },
          disabled: {
            label: "当前页停用",
            tail: "当前页已被后台停用的账号。",
          },
        },
        table: {
          columns: {
            displayId: "账号 ID",
            displayName: "显示名称",
            schoolName: "学校",
            teamName: "队伍",
            questionUsed: "已问答",
            questionLimit: "问答上限",
            questionRemaining: "剩余",
            questionProgress: "进度",
            tokenUsed: "Token 已用",
            status: "状态",
            remark: "备注",
            lastUsedAt: "最近使用",
          },
          empty: {
            remark: "无备注",
          },
          status: {
            enabled: "启用",
            disabled: "停用",
          },
          query: {
            keyword: {
              label: "关键字",
              placeholder: "搜索账号、学校、队伍或备注",
            },
            status: {
              label: "状态",
              placeholder: "状态",
            },
          },
          actions: {
            edit: {
              label: "编辑额度",
              title: "编辑账号额度 {{account}}",
              description: "调整该账号的问答上限、状态和备注。",
              success: "账号额度已更新",
            },
            toggle: {
              enable: "启用",
              disable: "停用",
              enabledSuccess: "账号问答已启用",
              disabledSuccess: "账号问答已停用",
            },
          },
        },
        edit: {
          fields: {
            account: "账号",
            schoolName: "学校",
            teamName: "队伍",
            questionLimit: "问答上限",
            status: "状态",
            remark: "备注",
          },
          remarkPlaceholder: "输入后台备注",
        },
      },
    },
    analytics: {
      learning: {
        heading: {
          title: "学习评价",
          description:
            "分别查看学生的对话关注和结构化答题评价，不把交流观察直接作为掌握结论。",
        },
        tabs: {
          students: "学生评价",
          questions: "题目质量",
        },
        metrics: {
          total: "学生总数",
          enabled: "已开启追踪",
          observed: "已观察学生",
          avgCoverage: "平均覆盖率",
          assessed: "已答题学生",
          demonstrated: "理解已体现",
          needsAttention: "待巩固学生",
          avgCoverageTail: "对话关注平均覆盖 {{rate}}%",
          retainedTail: "跨时段验证 {{count}} 人",
        },
        pilotMetrics: {
          title: "轻题参与概况",
          issued: "已投放题卡",
          issuedTail: "当前待答 {{active}}，已替换 {{superseded}}",
          answered: "已完成作答",
          answeredTail: "作答率 {{rate}}%，平均 {{seconds}} 秒",
          skipped: "主动跳过",
          skippedTail: "跳过率 {{rate}}%，过期 {{expired}}",
          correct: "正确作答",
          correctTail: "可判定正确率 {{rate}}%，自愿续题 {{continuations}}",
        },
        questionGovernance: {
          version: "题库版本",
          versionTail: "当前运行题库",
          total: "题目总数",
          totalTail: "当前合成目录",
          angles: "课程角度",
          anglesTail: "其中 {{count}} 个仅 1 道题",
          anglesUnavailableTail: "角度深度数据尚不可用",
          approved: "生产批准",
          approvedTail: "仅按生产清单统计",
          pending: "待教研",
          pendingTail: "等待公司实名复核",
          validThrough: "最近有效期",
          validThroughTail: "全库最早来源截止日",
          coverageGaps: "题库覆盖缺口",
          coverageGapsTail: "最高频领域 {{domain}}，共 {{count}} 个回合",
          coverageGapsEmptyTail: "暂无已记录的相关题缺口",
        },
        questionTable: {
          notAvailable: "未知",
          releaseStatuses: {
            production_approved: "生产已批准",
            curriculum_pending: "待教研",
            internal_pilot: "内部灰度",
            unknown: "状态未知",
          },
          columns: {
            domain: "领域",
            question: "题目",
            applicabilityScope: "适用范围",
            choiceDistribution: "选项分布",
            releaseStatus: "发布状态",
            sources: "来源机构",
            validThrough: "有效至",
            issued: "投放",
            issuedDetail: "待答 {{active}} · 已替换 {{superseded}}",
            answered: "作答",
            skipped: "跳过",
            expiredDetail: "已过期 {{expired}}",
            answerRate: "作答率",
            correctRate: "可判定正确率",
            averageSeconds: "平均作答",
            secondsValue: "{{count}} 秒",
          },
          scopeLabels: {
            jurisdiction: "地区",
            platform: "平台",
            businessStage: "阶段",
          },
          jurisdictions: {
            not_region_specific: "非特定地区",
            cn: "中国",
            eu: "欧盟",
            us: "美国",
            unknown: "范围未知",
          },
          platforms: {
            platform_agnostic: "平台无关",
            google_trends: "Google Trends",
            google_merchant_center: "Google Merchant Center",
            google_analytics_4: "Google Analytics 4",
            google_search: "Google 搜索",
            google_ads: "Google Ads",
            unknown: "平台未知",
          },
          businessStages: {
            market_research: "市场研究",
            product_operations: "商品运营",
            content_operations: "内容运营",
            traffic_acquisition: "流量获取",
            conversion_optimization: "转化优化",
            financial_management: "财务管理",
            customer_service: "客户服务",
            compliance_governance: "合规治理",
            unknown: "阶段未知",
          },
          choiceSelection: "{{count}} 次 · {{rate}}%",
          noChoiceResponses: "暂无有效选项作答",
          unmappedChoices: "另有 {{count}} 条历史选择无法映射",
          query: {
            keyword: {
              label: "搜索题目",
              placeholder: "搜索题干、题目 ID 或适用范围",
            },
            domain: {
              label: "领域",
              placeholder: "全部领域",
            },
            releaseStatus: {
              label: "发布状态",
              placeholder: "全部状态",
            },
          },
        },
        table: {
          student: "学生",
          school: "学校 / 团队",
          coverageRate: "对话关注覆盖",
          observed: "观察领域",
          structuredAssessment: "结构化评价",
          assessedDomains: "{{count}} 个领域",
          assessmentCount: "{{count}} 次作答",
          evaluation: "评价状态",
          mastery: "掌握状态",
          evaluationStatus: {
            unassessed: "尚未答题",
            accumulating: "评价积累中",
            demonstrated: "理解已体现",
            retained: "跨时段验证",
            needs_attention: "需要巩固",
          },
          enabled: "追踪状态",
          enabledYes: "已开启",
          enabledNo: "已关闭",
          lastAnalyzedAt: "最近分析",
          query: {
            keyword: {
              label: "搜索",
              placeholder: "搜索学生姓名或 ID",
            },
          },
          actions: {
            viewDetail: "查看详情",
          },
        },
        detail: {
          back: "返回学习评价",
          heading: {
            title: "学生详情",
            description: "分开查看对话关注、结构化答题评价与领域证据。",
          },
          metrics: {
            coverage: "对话关注覆盖",
            coverageTail: "观察到 {{count}} 个领域",
            observed: "已观察知识点",
            mastery: "掌握状态",
            assessed: "已评价领域",
            assessmentTail: "累计 {{count}} 次作答",
            demonstrated: "理解已体现",
            retained: "跨时段验证",
            needsAttention: "需要巩固",
            tracking: "追踪状态",
            trackingOn: "已开启",
            trackingOff: "已关闭",
          },
          sections: {
            mastery: {
              title: "领域证据摘要",
              description:
                "按知识领域展示观察和评价形成的证据信号，不将单次交流直接作为掌握结论。",
              empty: "暂无领域证据记录。",
            },
            evidence: {
              title: "证据摘要",
              description:
                "结构化答题显示题目与学生选择；对话观察仅显示摘要，不展示完整消息正文。",
              empty: "暂无学习证据。",
            },
          },
          fields: {
            question: "题目",
            selectedChoice: "学生选择",
          },
          domains: {
            market: "市场洞察",
            product: "商品策略",
            content: "内容表达",
            traffic: "流量运营",
            conversion: "转化优化",
            finance: "经营核算",
            service: "客户服务",
            compliance: "合规经营",
          },
          masteryStatus: {
            unobserved: "未观察",
            developing: "发展中",
            proficient: "掌握",
          },
          domainEvidenceStatus: {
            unobserved: "尚无证据",
            observed: "已有观察",
            developing: "证据积累中",
            proficient: "理解证据较强",
            assessed: "已有评价",
            needs_attention: "需要巩固",
          },
          evidenceKind: {
            conversationObservation: "对话观察",
            structuredAssessment: "答题评价",
            historical: "历史记录",
          },
          evidenceDirection: {
            positive: "正向记录",
            neutral: "修正记录",
            negative: "待纠正记录",
          },
          assessmentOutcome: {
            correct: "评价完整",
            partial: "部分完整",
            incorrect: "需要巩固",
            unscorable: "暂不判定",
            observed_unscored: "交流中观察到",
            legacy_unscored: "历史未评分",
          },
          evidenceSignal:
            "证据方向：{{direction}}；本条记录强度：{{strength}}%",
        },
      },
      heading: {
        title: "访问统计",
        description:
          "查看产品访问量、来源、每日流量、账号、学校和队伍维度的使用数据。",
      },
      filters: {
        from: "开始日期",
        to: "结束日期",
        rangePlaceholder: "选择统计日期范围",
        rangeSummary: "当前统计范围：{{from}} 至 {{to}}",
        rangeHint:
          "快捷范围和日期选择会同时影响总览、账号、学校/队伍三个页签。",
        refresh: "刷新",
        presets: {
          label: "快捷统计范围",
          today: "今天",
          last7Days: "近 7 天",
          last30Days: "近 30 天",
        },
      },
      tabs: {
        overview: "总览",
        accounts: "账号",
        schoolTeams: "学校/队伍",
      },
      metrics: {
        pv: {
          label: "访问量 PV",
          tail: "统计范围内的页面访问次数。",
        },
        uv: {
          label: "访客 UV",
          tail: "匿名访客与登录账号合并去重。",
        },
        questions: {
          label: "AI 提问",
          tail: "Token 估算 {{tokens}}。",
        },
        quality: {
          label: "平均会话",
          tail: "跳出率 {{bounce}}，错误 {{errors}} 次。",
        },
      },
      charts: {
        traffic: {
          title: "每日流量",
          description: "按日期查看 PV、UV 和 AI 提问变化。",
        },
        sources: {
          title: "来源类型",
          description: "按来源类型聚合访问量。",
        },
        pages: {
          title: "热门页面",
          description: "访问量最高的产品页面。",
        },
        series: {
          pv: "PV",
          uv: "UV",
          sessions: "会话",
          questions: "AI 提问",
        },
      },
      sources: {
        kinds: {
          direct: "直接访问",
          search: "搜索",
          referral: "外部引荐",
          social: "社交",
          ad: "广告/活动",
          internal: "站内",
          unknown: "未知",
        },
      },
      guides: {
        accounts: {
          title: "怎么看账号统计",
          description:
            "这里按单个产品账号汇总访问量、会话次数、活跃天数、AI 提问和常用页面，适合查看哪些账号真正使用了产品。",
        },
        schoolTeams: {
          title: "怎么看学校/队伍统计",
          description:
            "这里把同一学校和队伍下的账号合并统计，适合比较不同学校、不同队伍的整体活跃度和 AI 使用情况。",
        },
      },
      routes: {
        unknown: {
          label: "未识别页面",
          description: "系统还没有为这个路径配置中文说明，请结合原始路径判断。",
        },
        known: {
          commerce: {
            label: "电商小参谋工作台",
            description: "用户进入产品后进行电商经营分析和 AI 问答的主页面。",
          },
          login: {
            label: "登录页",
            description: "用户输入账号和密码进入产品的页面。",
          },
          register: {
            label: "注册页",
            description: "新用户创建产品账号的页面。",
          },
          home: {
            label: "首页",
            description: "产品首页或默认入口页面。",
          },
          overview: {
            label: "经营概览",
            description: "查看店铺、商品、内容和经营指标概览的页面。",
          },
          materials: {
            label: "素材中心",
            description: "查看和管理图片、视频、文案等电商素材的页面。",
          },
          products: {
            label: "商品管理",
            description: "查看商品资料、卖点和商品知识的页面。",
          },
          campaigns: {
            label: "活动与投放",
            description: "查看营销活动、投放计划和增长建议的页面。",
          },
          workspaces: {
            label: "工作空间",
            description: "查看 AI 生成结果、任务资料和协作产物的页面。",
          },
          platformConnect: {
            label: "平台连接",
            description: "连接或查看电商平台授权状态的页面。",
          },
          approvals: {
            label: "审批处理",
            description: "处理工具调用、内容发布或业务动作审批的页面。",
          },
          operations: {
            label: "运营任务",
            description: "查看运营动作、待办事项和执行结果的页面。",
          },
          agentOutput: {
            label: "AI 产出",
            description: "集中查看 AI 生成的方案、报告和可复用内容的页面。",
          },
          agentCapabilities: {
            label: "AI 能力说明",
            description: "了解电商多智能体可处理问题和能力边界的页面。",
          },
          questionIdeas: {
            label: "提问灵感",
            description: "为用户提供可直接向 AI 提问的业务问题示例。",
          },
          profile: {
            label: "个人中心",
            description: "用户查看或修改个人资料的页面。",
          },
        },
      },
      tables: {
        common: {
          pv: "PV",
          uv: "UV",
          sessions: "会话",
          agentQuestions: "AI 提问",
          tokenUsed: "Token",
          lastSeenAt: "最近访问",
        },
        pages: {
          title: "页面明细",
          description: "入口页、退出页、停留和错误等页面指标。",
          columns: {
            pageLabel: "页面名称/说明",
            routePath: "原始路径",
            avgDuration: "平均时长",
            entryCount: "入口",
            exitCount: "退出",
            errors: "错误",
          },
        },
        accounts: {
          topPagePv: "{{pv}} 次访问",
          columns: {
            displayId: "账号 ID",
            displayName: "显示名称",
            schoolName: "学校",
            teamName: "队伍",
            activeDays: "活跃天数",
            topPages: "主要页面",
          },
          query: {
            keyword: {
              label: "关键字",
              placeholder: "搜索账号、姓名、学校或队伍",
            },
            schoolName: {
              label: "学校",
              placeholder: "按学校过滤",
            },
            teamName: {
              label: "队伍",
              placeholder: "按队伍过滤",
            },
          },
        },
        schoolTeams: {
          columns: {
            schoolName: "学校",
            teamName: "队伍",
            accountCount: "账号数",
            activeAccounts: "活跃账号",
          },
          query: {
            schoolName: {
              label: "学校",
              placeholder: "按学校过滤",
            },
            teamName: {
              label: "队伍",
              placeholder: "按队伍过滤",
            },
          },
        },
      },
    },
    access: {
      roles: {
        table: {
          columns: {
            id: "ID",
            name: "角色名",
            code: "角色编号",
          },
          query: {
            keyword: {
              label: "关键字",
              placeholder: "搜索角色名称、ID 或角色编号",
            },
          },
          actions: {
            delete: {
              label: "删除",
              title: "删除角色 {{name}}",
              description:
                "确认删除角色 {{name}}（{{code}}）？删除前请确认该角色没有仍在使用的用户或权限关联。",
              confirm: "删除",
              success: "角色已删除",
            },
          },
        },
        create: {
          label: "新增角色",
          title: "创建角色",
          description:
            "角色编号用于系统识别角色。创建后请到角色授权页面配置权限。",
          fields: {
            name: {
              label: "角色名",
              placeholder: "输入角色名",
            },
            code: {
              label: "角色编号",
              placeholder: "输入唯一角色编号，例如 ops_manager",
            },
          },
          errors: {
            nameRequired: "请输入角色名。",
            codeRequired: "请输入角色编号。",
          },
          success: "角色已创建",
        },
        note: {
          title: "角色创建说明",
          line1:
            "这里用于创建角色，例如“运营人员”或“客服主管”。角色编号只是系统保存和区分角色用的唯一编号。",
          line2:
            "创建后，角色默认没有权限；需要到“角色授权”页面勾选权限后，这个角色才会生效。",
        },
      },
      appRoles: {
        table: {
          columns: {
            id: "ID",
            name: "角色名",
            code: "角色编号",
          },
          query: {
            keyword: {
              label: "关键字",
              placeholder: "搜索角色名称、ID 或角色编号",
            },
          },
          actions: {
            delete: {
              label: "删除",
              title: "删除用户角色 {{name}}",
              description:
                "确认删除用户角色 {{name}}（{{code}}）？删除前请确认该角色没有仍在使用的用户或权限关联。",
              confirm: "删除",
              success: "用户角色已删除",
            },
          },
        },
        create: {
          label: "新增用户角色",
          title: "创建用户角色",
          description:
            "角色编号用于系统识别产品用户角色。创建后请到用户角色授权页面配置权限。",
          fields: {
            name: {
              label: "角色名",
              placeholder: "输入角色名",
            },
            code: {
              label: "角色编号",
              placeholder: "输入唯一角色编号，例如 ops_manager",
            },
          },
          errors: {
            nameRequired: "请输入角色名。",
            codeRequired: "请输入角色编号。",
          },
          success: "用户角色已创建",
        },
        note: {
          title: "用户角色创建说明",
          line1:
            "这里用于创建产品用户角色，例如“普通用户”或“学校管理员”。角色编号只是系统保存和区分角色用的唯一编号。",
          line2:
            "创建后，角色默认没有权限；需要到“用户角色授权”页面勾选权限后，这个角色才会生效。",
        },
      },
      rolePermissions: {
        empty: {
          title: "暂无可授权角色",
          description: "创建权限角色后，可以在这里配置该角色的权限覆盖集。",
        },
        editor: {
          codePrefix: "角色编号：",
          descriptionSuffix:
            "。服务端返回完整权限树，并标记当前角色已授权节点。",
          reset: "重置勾选",
          save: "保存角色授权",
          searchPlaceholder: "搜索权限名称",
          loading: "权限树加载中。",
          empty: "没有匹配到权限项。",
          success: "角色授权已保存",
        },
        summary: {
          group: "分组",
          action: "操作",
          selected: "已选",
        },
      },
      appRolePermissions: {
        empty: {
          title: "暂无可授权角色",
          description: "创建用户角色后，可以在这里配置该角色的产品功能权限。",
        },
        editor: {
          codePrefix: "角色编号：",
          descriptionSuffix:
            "。服务端返回完整权限树，并标记当前角色已授权节点。",
          reset: "重置勾选",
          save: "保存用户角色授权",
          searchPlaceholder: "搜索权限名称",
          loading: "权限树加载中。",
          empty: "没有匹配到权限项。",
          success: "用户角色授权已保存",
        },
        summary: {
          group: "分组",
          action: "操作",
          selected: "已选",
        },
      },
    },
  },
  login: {
    brand: "Workspace Admin",
    hero: {
      titleLine1: "权限先行，",
      titleLine2: "后台可控。",
      desc1: "后台入口只负责恢复后台身份、菜单权限和管理台访问能力。",
      desc2: "业务 App 的用户初始化流程不在这里执行，避免混淆两套权限模型。",
    },
    welcome: "管理台登录",
    title: "登录管理后台",
    subtitle: "请输入账号标识和密码后进入后台壳层。",
    notice: {
      title: "本地服务端联调",
      description:
        "本地服务端启动后，登录、会话恢复和后台权限接口会通过 dev proxy 转发。",
    },
    error: {
      title: "登录失败",
      invalidCredentials: "账号或密码不正确，请检查后重试。",
      serviceUnavailable: "后台认证服务暂时不可用，请稍后再试。",
      permissionUnavailable: "后台权限加载失败，请稍后重试或联系管理员。",
    },
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
  },
  register: {
    brand: "AI 标准化开发框架",
    hero: {
      titleLine1: "创建账号，",
      titleLine2: "开始 AI 开发。",
      desc1: "注册后即可在统一规范下，让 AI 按框架要求完成开发任务。",
      desc2: "同样支持主题切换、多语言和模块化结构，便于持续迭代。",
    },
    welcome: "创建新账号",
    title: "注册你的账号",
    subtitle: "填写信息后即可进入框架体验。",
    form: {
      password: {
        label: "密码",
        placeholder: "8-128位，字母/数字/符号至少2种",
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
      passwordMinLength: "密码需8-128位，且字母、数字、符号至少2种",
    },
  },
} as const

export default messages
