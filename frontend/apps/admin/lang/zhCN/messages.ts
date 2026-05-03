const messages = {
  admin: {
    navigation: {
      platform: "平台",
      dashboard: "控制台",
      accountManagement: "账号管理",
      adminUsers: "后台账号",
      appUsers: "App 用户",
      accessControl: "权限中心",
      roles: "权限角色",
      rolePermissions: "角色授权",
      appRoles: "App 角色",
      appRolePermissions: "App 角色授权",
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
        description: "今日关键经营指标、运营趋势和系统状态概览。",
      },
      metrics: {
        revenue: "今日成交额",
        newUsers: "新增会员",
        pendingTickets: "待处理工单",
        availability: "系统可用率",
      },
      days: {
        mon: "周一",
        tue: "周二",
        wed: "周三",
        thu: "周四",
        fri: "周五",
        sat: "周六",
        sun: "周日",
      },
      charts: {
        revenueTrend: {
          title: "成交趋势",
          description: "最近 7 天成交额与订单量走势。",
        },
        userSegments: {
          title: "用户结构",
          description: "会员活跃状态占比。",
        },
        channels: {
          title: "渠道成交",
          description: "本周各渠道成交贡献。",
        },
        series: {
          revenue: "成交额",
          orders: "订单量",
          channelShare: "成交占比",
        },
      },
      channels: {
        directStore: "直营商城",
        partner: "渠道伙伴",
        miniProgram: "小程序",
        offline: "线下门店",
      },
      userSegments: {
        new: "新用户",
        active: "活跃用户",
        silent: "沉默用户",
      },
      orders: {
        title: "最新订单流",
        description: "高频业务动作的实时视图。",
        columns: {
          id: "单号",
          source: "来源",
          status: "状态",
          amount: "金额",
        },
      },
      orderSources: {
        eastChina: "华东大区",
        directStore: "直营商城",
        partner: "渠道伙伴",
        miniProgram: "小程序商城",
      },
      orderStatuses: {
        reviewing: "待复核",
        shipped: "已发货",
        pendingPayment: "待付款",
        completed: "已完成",
      },
      riskMonitor: {
        title: "风险监控",
        description: "后台系统常见待办和风险指标。",
      },
      risks: {
        refundBacklog: "退款工单积压",
        permissionReview: "权限变更待审",
        inventoryDelay: "库存同步延迟",
      },
      servicesPanel: {
        title: "服务状态",
        description: "核心服务的健康状态。",
        columns: {
          service: "服务",
          status: "状态",
          metric: "指标",
        },
      },
      services: {
        apiGateway: "API 网关",
        jobQueue: "任务队列",
        notifications: "消息通知",
      },
      serviceStatuses: {
        normal: "正常",
        watching: "观察中",
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
            remark: "备注",
            status: "状态",
            roles: "角色",
            roleCount: "角色数",
          },
          empty: {
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
              placeholder: "输入初始密码",
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
            label: "App 用户总数",
            tail: "服务端返回的 App 用户总量。",
          },
          enabled: {
            label: "启用账号",
            tail: "具备正常 App 权限访问能力的账号。",
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
            remark: "备注",
            status: "状态",
            roles: "角色",
            roleCount: "角色数",
          },
          empty: {
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
              title: "编辑 App 用户 {{id}}",
              description: "当前可编辑备注，保存后会同步到服务端。",
              success: "App 用户已更新",
            },
            delete: {
              label: "删除",
              title: "删除 App 用户 {{id}}",
              description:
                "确认删除 App 用户 {{name}}（{{id}}）？删除后该账号将无法使用 App 权限。",
              confirm: "删除",
              success: "App 用户已删除",
            },
            toggle: {
              enable: "启用",
              disable: "停用",
              enabledSuccess: "App 用户已启用",
              disabledSuccess: "App 用户已停用",
            },
            editRoles: "编辑角色",
          },
        },
        edit: {
          fields: {
            id: "ID",
            displayName: "显示名称",
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
    access: {
      roles: {
        table: {
          columns: {
            id: "ID",
            name: "角色名",
            code: "编码",
          },
          query: {
            keyword: {
              label: "关键字",
              placeholder: "搜索角色名称、ID 或编码",
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
          description: "编码只是角色的唯一编号。创建后请到角色授权页面配置权限。",
          fields: {
            name: {
              label: "角色名",
              placeholder: "输入角色名",
            },
            code: {
              label: "编码",
              placeholder: "输入唯一角色编码，例如 ops_manager",
            },
          },
          errors: {
            nameRequired: "请输入角色名。",
            codeRequired: "请输入角色编码。",
          },
          success: "角色已创建",
        },
        note: {
          title: "角色创建说明",
          line1:
            "这里用于创建角色，例如“运营人员”或“客服主管”。编码只是这个角色的唯一编号，方便系统保存和区分角色。",
          line2:
            "创建后，角色默认没有权限；需要到“角色授权”页面勾选权限后，这个角色才会生效。",
        },
      },
      appRoles: {
        table: {
          columns: {
            id: "ID",
            name: "角色名",
            code: "编码",
          },
          query: {
            keyword: {
              label: "关键字",
              placeholder: "搜索角色名称、ID 或编码",
            },
          },
          actions: {
            delete: {
              label: "删除",
              title: "删除 App 角色 {{name}}",
              description:
                "确认删除 App 角色 {{name}}（{{code}}）？删除前请确认该角色没有仍在使用的用户或权限关联。",
              confirm: "删除",
              success: "App 角色已删除",
            },
          },
        },
        create: {
          label: "新增 App 角色",
          title: "创建 App 角色",
          description:
            "编码只是 App 角色的唯一编号。创建后请到 App 角色授权页面配置权限。",
          fields: {
            name: {
              label: "角色名",
              placeholder: "输入角色名",
            },
            code: {
              label: "编码",
              placeholder: "输入唯一角色编码，例如 ops_manager",
            },
          },
          errors: {
            nameRequired: "请输入角色名。",
            codeRequired: "请输入角色编码。",
          },
          success: "App 角色已创建",
        },
        note: {
          title: "App 角色创建说明",
          line1:
            "这里用于创建 App 角色，例如“会员”或“内容作者”。编码只是这个角色的唯一编号，方便系统保存和区分角色。",
          line2:
            "创建后，角色默认没有权限；需要到“App 角色授权”页面勾选权限后，这个角色才会生效。",
        },
      },
      rolePermissions: {
        empty: {
          title: "暂无可授权角色",
          description: "创建权限角色后，可以在这里配置该角色的权限覆盖集。",
        },
        editor: {
          codePrefix: "编码：",
          descriptionSuffix: "。服务端返回完整权限树，并标记当前角色已授权节点。",
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
          description: "创建 App 角色后，可以在这里配置该角色的权限覆盖集。",
        },
        editor: {
          codePrefix: "编码：",
          descriptionSuffix: "。服务端返回完整权限树，并标记当前角色已授权节点。",
          reset: "重置勾选",
          save: "保存 App 角色授权",
          searchPlaceholder: "搜索权限名称",
          loading: "权限树加载中。",
          empty: "没有匹配到权限项。",
          success: "App 角色授权已保存",
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
