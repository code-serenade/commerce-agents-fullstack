const permissions = {
  dashboard: {
    label: "Dashboard",
  },
  accounts: {
    label: "Accounts",
    admin_users: {
      label: "Admin Users",
    },
    app_users: {
      label: "Product Users",
    },
  },
  access_control: {
    label: "Access Control",
    roles: {
      label: "Roles",
    },
    role_permissions: {
      label: "Role Permissions",
    },
    app_roles: {
      label: "User Roles",
    },
    app_role_permissions: {
      label: "User Role Permissions",
    },
  },
  usage: {
    label: "Usage",
    team_usage: {
      label: "Account Usage",
    },
  },
  analytics: {
    label: "Analytics",
    overview: {
      label: "Access Overview",
    },
    accounts: {
      label: "Account Analytics",
    },
    school_teams: {
      label: "School / Team Analytics",
    },
  },
  profile: {
    label: "Profile",
    view: {
      label: "View Profile",
    },
    update: {
      label: "Update Profile",
    },
  },
} as const

export default permissions
