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
