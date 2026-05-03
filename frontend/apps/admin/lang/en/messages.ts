const messages = {
  admin: {
    navigation: {
      platform: "Platform",
      dashboard: "Dashboard",
      accountManagement: "Accounts",
      adminUsers: "Admin Users",
      appUsers: "App Users",
      accessControl: "Access Control",
      roles: "Roles",
      rolePermissions: "Role Permissions",
      appRoles: "App Roles",
      appRolePermissions: "App Role Permissions",
    },
    authLayout: {
        eyebrow: "Workspace Admin",
        title: "Admin Entry",
    },
    shell: {
      brand: {
        eyebrow: "Workspace",
        title: "Admin",
        description: "Enterprise",
      },
      navigation: {
        mobileLabel: "Primary navigation",
      },
      notifications: {
        label: "Notifications",
        title: "Notifications",
        empty: "You're all caught up.",
        actions: {
          markAllRead: "Mark all as read",
          markRead: "Mark as read",
          dismiss: "Dismiss",
        },
        items: {
          weeklyReports: {
            title: "14 new weekly reports arrived",
            description:
              "Review the latest regional summaries and flagged changes before the afternoon sync.",
            time: "3 hours ago",
          },
          reply: {
            title: "Zhu Pianyou replied to you",
            description:
              "The access review note now includes the rollout window and the owner confirmation.",
            time: "Just now",
          },
          comment: {
            title: "Qu Lili commented on your update",
            description:
              "She asked whether the dashboard prototype should keep the current chart grouping.",
            time: "2024-01-01",
          },
          taskReminder: {
            title: "Task reminder",
            description:
              "The finance export checklist is still waiting for your final verification.",
            time: "1 day ago",
          },
        },
      },
      workspaceTabs: {
        clear: "Close other pages",
      },
      account: {
        fallbackName: "Admin",
        fallbackInitial: "A",
        fallbackId: "workspace-admin@example.com",
        logout: "Log out",
        actions: {
          profile: "Profile",
        },
      },
    },
    dashboard: {
      heading: {
        title: "Dashboard",
        description:
          "Today’s operating metrics, business trends, and system status.",
      },
      metrics: {
        revenue: "Today Revenue",
        newUsers: "New Members",
        pendingTickets: "Pending Tickets",
        availability: "Availability",
      },
      days: {
        mon: "Mon",
        tue: "Tue",
        wed: "Wed",
        thu: "Thu",
        fri: "Fri",
        sat: "Sat",
        sun: "Sun",
      },
      charts: {
        revenueTrend: {
          title: "Revenue Trend",
          description: "Revenue and order volume over the last 7 days.",
        },
        userSegments: {
          title: "User Segments",
          description: "Member activity status distribution.",
        },
        channels: {
          title: "Channel Revenue",
          description: "This week’s contribution by channel.",
        },
        series: {
          revenue: "Revenue",
          orders: "Orders",
          channelShare: "Share",
        },
      },
      channels: {
        directStore: "Direct Store",
        partner: "Partners",
        miniProgram: "Mini Program",
        offline: "Offline Stores",
      },
      userSegments: {
        new: "New Users",
        active: "Active Users",
        silent: "Inactive Users",
      },
      orders: {
        title: "Latest Orders",
        description: "A real-time view of high-frequency business activity.",
        columns: {
          id: "Order ID",
          source: "Source",
          status: "Status",
          amount: "Amount",
        },
      },
      orderSources: {
        eastChina: "East China",
        directStore: "Direct Store",
        partner: "Partners",
        miniProgram: "Mini Program Store",
      },
      orderStatuses: {
        reviewing: "Reviewing",
        shipped: "Shipped",
        pendingPayment: "Pending Payment",
        completed: "Completed",
      },
      riskMonitor: {
        title: "Risk Monitor",
        description: "Common backlog and risk indicators for admin systems.",
      },
      risks: {
        refundBacklog: "Refund Backlog",
        permissionReview: "Permission Review",
        inventoryDelay: "Inventory Sync Delay",
      },
      servicesPanel: {
        title: "Service Status",
        description: "Health status of core services.",
        columns: {
          service: "Service",
          status: "Status",
          metric: "Metric",
        },
      },
      services: {
        apiGateway: "API Gateway",
        jobQueue: "Job Queue",
        notifications: "Notifications",
      },
      serviceStatuses: {
        normal: "Normal",
        watching: "Watching",
      },
    },
    accounts: {
      adminUsers: {
        data: {
          fallbackDisplayName: "Display name not set",
        },
        metrics: {
          all: {
            label: "Admin Users",
            tail: "Admin accounts that can sign in to the console.",
          },
          enabled: {
            label: "Enabled",
            tail: "Accounts with active admin access.",
          },
          disabled: {
            label: "Disabled",
            tail: "Accounts disabled pending review or restoration.",
          },
          multiRole: {
            label: "Multi-role",
            tail: "Accounts assigned to more than one role.",
          },
        },
        table: {
          columns: {
            id: "ID",
            displayName: "Display Name",
            remark: "Remark",
            status: "Status",
            roles: "Roles",
            roleCount: "Role Count",
          },
          empty: {
            remark: "No remark",
            roles: "No roles assigned",
          },
          status: {
            enabled: "Enabled",
            disabled: "Disabled",
          },
          query: {
            keyword: {
              label: "Keyword",
              placeholder: "Search display name, ID, remark, or role",
            },
            status: {
              label: "Status",
              placeholder: "Status",
            },
          },
          actions: {
            edit: {
              label: "Edit",
              title: "Edit admin user {{id}}",
              description: "Only the remark can be edited here.",
              success: "Admin user updated",
            },
            delete: {
              label: "Delete",
              title: "Delete admin user {{id}}",
              description:
                "Delete admin user {{name}} ({{id}})? This account will no longer be able to access the admin console.",
              confirm: "Delete",
              success: "Admin user deleted",
            },
            toggle: {
              enable: "Enable",
              disable: "Disable",
              enabledSuccess: "Admin user enabled",
              disabledSuccess: "Admin user disabled",
            },
            editRoles: "Edit roles",
          },
        },
        create: {
          label: "New Admin User",
          title: "Create Admin User",
          description:
            "Add an existing auth account to admin access, or register a new account and create admin access immediately.",
          mode: {
            label: "Creation Mode",
            existing: "Use existing account",
            new: "Create new account",
          },
          fields: {
            account: {
              label: "Account",
              placeholder: "Enter username, email, or display ID",
            },
            username: {
              label: "Username",
              placeholder: "Enter new username",
            },
            password: {
              label: "Password",
              placeholder: "Enter initial password",
            },
            displayName: {
              label: "Display Name",
              placeholder: "Enter display name",
            },
            email: {
              label: "Email (optional)",
              placeholder: "Enter email",
            },
            remark: {
              label: "Remark",
              placeholder: "Enter remark (optional)",
            },
          },
          errors: {
            identifierRequired: "Enter an account.",
            usernameRequired: "Enter a username.",
            passwordRequired: "Enter a password.",
            displayNameRequired: "Enter a display name.",
          },
        },
        edit: {
          fields: {
            id: "ID",
            displayName: "Display Name",
            remark: "Remark",
          },
          remarkPlaceholder: "Enter remark",
        },
        rolesDialog: {
          title: "Edit roles{{name}}",
          id: "ID {{id}}",
          description: "Only user roles are changed here. User information is not affected.",
          loading: "Loading roles.",
          empty: "No configurable roles.",
          selected: "Selected",
          cancel: "Cancel",
          reset: "Reset",
          save: "Save Roles",
          success: "User roles updated",
        },
      },
      appUsers: {
        data: {
          fallbackDisplayName: "Display name not set",
        },
        metrics: {
          all: {
            label: "App Users",
            tail: "Total App users returned by the server.",
          },
          enabled: {
            label: "Enabled",
            tail: "Accounts with active App access.",
          },
          disabled: {
            label: "Disabled",
            tail: "Accounts disabled pending review or restoration.",
          },
          multiRole: {
            label: "Multi-role",
            tail: "Accounts assigned to more than one role.",
          },
        },
        table: {
          columns: {
            id: "ID",
            displayName: "Display Name",
            remark: "Remark",
            status: "Status",
            roles: "Roles",
            roleCount: "Role Count",
          },
          empty: {
            remark: "No remark",
            roles: "No roles assigned",
          },
          status: {
            enabled: "Enabled",
            disabled: "Disabled",
          },
          query: {
            keyword: {
              label: "Keyword",
              placeholder: "Search display name, ID, remark, or role",
            },
            status: {
              label: "Status",
              placeholder: "Status",
            },
          },
          actions: {
            edit: {
              label: "Edit",
              title: "Edit App user {{id}}",
              description: "Only the remark can be edited here.",
              success: "App user updated",
            },
            delete: {
              label: "Delete",
              title: "Delete App user {{id}}",
              description:
                "Delete App user {{name}} ({{id}})? This account will no longer be able to use App permissions.",
              confirm: "Delete",
              success: "App user deleted",
            },
            toggle: {
              enable: "Enable",
              disable: "Disable",
              enabledSuccess: "App user enabled",
              disabledSuccess: "App user disabled",
            },
            editRoles: "Edit roles",
          },
        },
        edit: {
          fields: {
            id: "ID",
            displayName: "Display Name",
            remark: "Remark",
          },
          remarkPlaceholder: "Enter remark",
        },
        rolesDialog: {
          title: "Edit roles{{name}}",
          id: "ID {{id}}",
          description: "Only user roles are changed here. User information is not affected.",
          loading: "Loading roles.",
          empty: "No configurable roles.",
          selected: "Selected",
          cancel: "Cancel",
          reset: "Reset",
          save: "Save Roles",
          success: "User roles updated",
        },
      },
    },
    access: {
      roles: {
        table: {
          columns: {
            id: "ID",
            name: "Role Name",
            code: "Code",
          },
          query: {
            keyword: {
              label: "Keyword",
              placeholder: "Search role name, ID, or code",
            },
          },
          actions: {
            delete: {
              label: "Delete",
              title: "Delete role {{name}}",
              description:
                "Delete role {{name}} ({{code}})? Make sure no users or permissions are still linked to it.",
              confirm: "Delete",
              success: "Role deleted",
            },
          },
        },
        create: {
          label: "New Role",
          title: "Create Role",
          description:
            "The code is the unique role identifier. Configure permissions on the Role Permissions page after creating it.",
          fields: {
            name: {
              label: "Role Name",
              placeholder: "Enter role name",
            },
            code: {
              label: "Code",
              placeholder: "Enter a unique code, for example ops_manager",
            },
          },
          errors: {
            nameRequired: "Enter a role name.",
            codeRequired: "Enter a role code.",
          },
          success: "Role created",
        },
        note: {
          title: "Role Creation Notes",
          line1:
            "Create roles such as Operations or Support Lead. The code is a unique identifier used by the system to store and distinguish roles.",
          line2:
            "New roles have no permissions by default. Grant permissions on the Role Permissions page before the role takes effect.",
        },
      },
      appRoles: {
        table: {
          columns: {
            id: "ID",
            name: "Role Name",
            code: "Code",
          },
          query: {
            keyword: {
              label: "Keyword",
              placeholder: "Search role name, ID, or code",
            },
          },
          actions: {
            delete: {
              label: "Delete",
              title: "Delete App role {{name}}",
              description:
                "Delete App role {{name}} ({{code}})? Make sure no users or permissions are still linked to it.",
              confirm: "Delete",
              success: "App role deleted",
            },
          },
        },
        create: {
          label: "New App Role",
          title: "Create App Role",
          description:
            "The code is the unique App role identifier. Configure permissions on the App Role Permissions page after creating it.",
          fields: {
            name: {
              label: "Role Name",
              placeholder: "Enter role name",
            },
            code: {
              label: "Code",
              placeholder: "Enter a unique code, for example ops_manager",
            },
          },
          errors: {
            nameRequired: "Enter a role name.",
            codeRequired: "Enter a role code.",
          },
          success: "App role created",
        },
        note: {
          title: "App Role Creation Notes",
          line1:
            "Create App roles such as Member or Content Author. The code is a unique identifier used by the system to store and distinguish roles.",
          line2:
            "New roles have no permissions by default. Grant permissions on the App Role Permissions page before the role takes effect.",
        },
      },
      rolePermissions: {
        empty: {
          title: "No Roles Available",
          description:
            "Create permission roles, then configure their permission coverage here.",
        },
        editor: {
          codePrefix: "Code: ",
          descriptionSuffix:
            ". The server returns the full permission tree and marks the nodes already granted to this role.",
          reset: "Reset Selection",
          save: "Save Role Permissions",
          searchPlaceholder: "Search permissions",
          loading: "Loading permission tree.",
          empty: "No matching permissions.",
          success: "Role permissions saved",
        },
        summary: {
          group: "Groups",
          action: "Actions",
          selected: "Selected",
        },
      },
      appRolePermissions: {
        empty: {
          title: "No Roles Available",
          description:
            "Create App roles, then configure their permission coverage here.",
        },
        editor: {
          codePrefix: "Code: ",
          descriptionSuffix:
            ". The server returns the full permission tree and marks the nodes already granted to this role.",
          reset: "Reset Selection",
          save: "Save App Role Permissions",
          searchPlaceholder: "Search permissions",
          loading: "Loading permission tree.",
          empty: "No matching permissions.",
          success: "App role permissions saved",
        },
        summary: {
          group: "Groups",
          action: "Actions",
          selected: "Selected",
        },
      },
    },
  },
  login: {
    brand: "Workspace Admin",
    hero: {
      titleLine1: "Permission first,",
      titleLine2: "admin controlled.",
      desc1:
        "The admin entry restores admin identity, menu permissions, and console access only.",
      desc2:
        "Business app user initialization does not run here, keeping the permission models separate.",
    },
    welcome: "ADMIN SIGN IN",
    title: "Sign in to Admin",
    subtitle: "Enter your account identifier and password to enter the admin shell.",
    notice: {
      title: "Local Backend Ready",
      description:
        "When the local backend is running, login, session restore, and admin permission APIs are forwarded through the dev proxy.",
    },
    form: {
      password: {
        label: "Password",
        placeholder: "Enter password",
      },
      submit: "Sign in",
      submitting: "Signing in...",
      identifier: {
        label: "Identifier",
        placeholder: "Enter identifier",
      },
    },
  },
  register: {
    brand: "AI STANDARDIZED FRAMEWORK",
    hero: {
      titleLine1: "Create your account,",
      titleLine2: "start AI development.",
      desc1:
        "After registering, you can let AI implement tasks with unified framework standards.",
      desc2:
        "Theme mode, i18n, and modular structure are ready for scalable iteration.",
    },
    welcome: "CREATE ACCOUNT",
    title: "Register your account",
    subtitle: "Fill in your information to get started.",
    form: {
      password: {
        label: "Password",
        placeholder: "Enter password",
      },
      confirmPassword: {
        label: "Confirm password",
        placeholder: "Enter password again",
      },
      submit: "Register",
      submitting: "Registering...",
      username: {
        label: "Username",
        placeholder: "Enter username",
      },
      displayName: {
        label: "Display name (optional)",
        placeholder: "Enter display name",
      },
      email: {
        label: "Email (optional)",
        placeholder: "Enter email",
      },
    },
    footer: {
      toLoginPrefix: "Already have an account?",
      toLoginAction: "Sign in",
    },
    error: {
      passwordMismatch: "Passwords do not match.",
    },
  },
} as const

export default messages
