const permissions = {
  dashboard: {
    label: "控制台",
  },
  accounts: {
    label: "账号管理",
    admin_users: {
      label: "后台账号",
    },
    app_users: {
      label: "App 用户",
    },
  },
  access_control: {
    label: "权限中心",
    roles: {
      label: "权限角色",
    },
    role_permissions: {
      label: "角色授权",
    },
    app_roles: {
      label: "App 角色",
    },
    app_role_permissions: {
      label: "App 角色授权",
    },
  },
  profile: {
    label: "个人中心",
    view: {
      label: "查看个人资料",
    },
    update: {
      label: "更新个人资料",
    },
  },
} as const

export default permissions
