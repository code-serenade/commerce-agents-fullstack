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
      label: "产品用户",
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
      label: "用户角色",
    },
    app_role_permissions: {
      label: "用户角色授权",
    },
  },
  usage: {
    label: "用量管理",
    team_usage: {
      label: "账号用量",
    },
  },
  analytics: {
    label: "数据统计",
    overview: {
      label: "访问统计总览",
    },
    accounts: {
      label: "账号访问统计",
    },
    school_teams: {
      label: "学校/队伍统计",
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
