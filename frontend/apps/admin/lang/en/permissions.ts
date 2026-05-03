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
      label: "App Users",
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
      label: "App Roles",
    },
    app_role_permissions: {
      label: "App Role Permissions",
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
