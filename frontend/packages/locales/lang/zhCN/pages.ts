const pages = {
  authRegister: {
    brand: "AI 标准化开发框架",
    hero: {
      titleLine1: "创建账号，",
      titleLine2: "开始 AI 开发。",
      desc1: "这里只创建共享账号；登录具体业务时再初始化该业务身份。",
      desc2: "同样支持主题切换、多语言和模块化结构，便于持续迭代。",
    },
    welcome: "创建新账号",
    title: "注册你的账号",
    subtitle: "创建一次共享账号即可；每个业务会在登录时初始化自己的用户记录。",
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
  profile: {
    title: "个人资料",
    loading: "正在加载个人资料...",
    errorTitle: "个人资料加载失败",
    sections: {
      account: "账号信息",
      email: "邮箱",
      security: "修改密码",
    },
    fields: {
      id: "ID",
      email: "邮箱",
      name: "名称",
      currentPassword: "当前密码",
      newPassword: "新密码",
      confirmPassword: "确认新密码",
    },
    placeholders: {
      name: "请输入显示名称",
      email: "请输入邮箱地址",
      currentPassword: "请输入当前密码",
      newPassword: "请输入新密码",
      confirmPassword: "请再次输入新密码",
    },
    help: {
      name: "你的名称会显示在应用中与你相关的贡献、提及和资料位置。",
      emailNotSet: "设置邮箱后可用于账号找回和通知。",
      newPassword: "新密码至少需要 8 个字符。",
    },
    avatar: {
      title: "头像",
      edit: "编辑",
      upload: "上传照片...",
      uploading: "上传中...",
      remove: "移除头像",
      removeConfirm: {
        title: "移除头像？",
        description: "当前头像会从你的账号资料中移除。",
        cancel: "取消",
        action: "移除头像",
      },
    },
    password: {
      configured: "已配置",
    },
    email: {
      notSet: "未设置，请添加邮箱。",
    },
    actions: {
      save: "保存资料",
      saving: "保存中...",
      setEmail: "设置邮箱",
      changeEmail: "修改邮箱",
      hideEmail: "隐藏",
      updateEmail: "更新邮箱",
      savingEmail: "保存中...",
      changePassword: "修改密码",
      hidePassword: "隐藏",
      updatePassword: "更新密码",
      changingPassword: "修改中...",
    },
    status: {
      saved: "资料已保存。",
      failed: "资料更新失败。",
      avatarSaved: "头像已更新。",
      avatarRemoved: "头像已移除。",
      emailSaved: "邮箱已更新。",
      passwordChanged: "密码已修改。",
      passwordMismatch: "两次输入的新密码不一致。",
    },
    fallback: {
      id: "未知 ID",
      email: "未设置邮箱",
      name: "用户",
    },
  },
} as const

export default pages
