const messages = {
  admin: {
    navigation: {
      platform: "Platform",
      dashboard: "Dashboard",
      accountManagement: "Accounts",
      adminUsers: "Admin Users",
      appUsers: "Product Users",
      accessControl: "Access Control",
      roles: "Roles",
      rolePermissions: "Role Permissions",
      appRoles: "User Roles",
      appRolePermissions: "User Role Permissions",
      analytics: "Analytics",
      accessAnalytics: "Access Analytics",
      usageManagement: "Usage",
      teamUsage: "Account Usage",
    },
    loading: {
      app: "Loading Commerce Multi-Agent Admin",
      route: "Preparing admin page",
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
          "Product traffic and AI usage from {{from}} to {{to}}. Data status: {{status}}.",
        loading: "loading",
        ready: "updated",
      },
      metrics: {
        pv: {
          label: "Product Visits",
          tail: "{{sessions}} visit sessions in this range.",
        },
        questions: {
          label: "AI Questions",
          tail: "About {{tokens}} tokens used in this range.",
        },
        accounts: {
          label: "Product Accounts",
          tail: "{{enabled}} accounts are enabled.",
        },
        teams: {
          label: "Schools / Teams",
          tail: "{{active}} accounts were active in this range.",
        },
      },
      charts: {
        trafficTrend: {
          title: "Daily Visits and Questions",
          description: "Product visits, visitors, and AI questions by day.",
        },
        productMix: {
          title: "Product Usage Summary",
          description: "A quick view of whether users are actively using the product.",
        },
        series: {
          pv: "Visits",
          uv: "Visitors",
          questions: "AI Questions",
        },
      },
      productMix: {
        activeAccounts: "Active Accounts",
        avgSession: "Average Session",
        bounceRate: "Bounce Rate",
        errors: "Frontend / API Errors",
      },
      topPages: {
        title: "Popular Product Pages",
        description: "The product pages managers should watch most closely.",
        columns: {
          page: "Page",
          path: "Raw Path",
          pv: "Visits",
          uv: "Visitors",
        },
      },
      schoolTeams: {
        title: "School / Team Activity",
        description: "Visits, active accounts, and AI questions by school and team.",
        rowDetail: "{{accounts}} active accounts, {{questions}} AI questions",
        empty: "No school or team traffic yet.",
      },
      usageAlerts: {
        title: "Question Quota",
        description: "Accounts with less remaining quota are shown first.",
        columns: {
          account: "Account",
          schoolTeam: "School / Team",
          used: "Questions Used",
          remaining: "Remaining",
        },
      },
      dataStatus: {
        title: "Data Connection Status",
        description: "Shows which real admin data modules are loaded.",
        columns: {
          name: "Data Module",
          status: "Status",
          detail: "Details",
        },
      },
      statusRows: {
        ready: "Loaded",
        loading: "Loading",
        analytics: {
          name: "Access Analytics",
          detail: "From product visits, sources, pages, and daily traffic APIs.",
        },
        accounts: {
          name: "Account Statistics",
          detail: "From product account and status metrics APIs.",
        },
        usage: {
          name: "School / Team and Quota",
          detail: "From school/team analytics and account quota APIs.",
        },
      },
      empty: {
        schoolTeam: "No school / team",
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
            schoolName: "School",
            teamName: "Team",
            remark: "Remark",
            status: "Status",
            roles: "Roles",
            roleCount: "Role Count",
          },
          empty: {
            schoolName: "No school",
            teamName: "No team",
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
            schoolName: "School",
            teamName: "Team",
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
            label: "Product Users",
            tail: "Total product user accounts.",
          },
          enabled: {
            label: "Enabled",
            tail: "Accounts with active product access.",
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
            schoolName: "School",
            teamName: "Team",
            remark: "Remark",
            status: "Status",
            roles: "Roles",
            roleCount: "Role Count",
          },
          empty: {
            schoolName: "No school",
            teamName: "No team",
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
              title: "Edit product user {{id}}",
              description: "Only the remark can be edited here.",
              success: "Product user updated",
            },
            delete: {
              label: "Delete",
              title: "Delete product user {{id}}",
              description:
                "Delete product user {{name}} ({{id}})? This account will no longer be able to use product features.",
              confirm: "Delete",
              success: "Product user deleted",
            },
            toggle: {
              enable: "Enable",
              disable: "Disable",
              enabledSuccess: "Product user enabled",
              disabledSuccess: "Product user disabled",
            },
            editRoles: "Edit roles",
          },
        },
        edit: {
          fields: {
            id: "ID",
            displayName: "Display Name",
            schoolName: "School",
            teamName: "Team",
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
    usage: {
      teamUsage: {
        metrics: {
          all: {
            label: "Accounts",
            tail: "Total accounts returned by the server query.",
          },
          questionUsed: {
            label: "Current page questions",
            tail: "Recorded question usage across accounts on this page.",
          },
          exhausted: {
            label: "Exhausted on page",
            tail: "Enabled accounts on this page with no remaining questions.",
          },
          disabled: {
            label: "Disabled on page",
            tail: "Accounts disabled by admins on this page.",
          },
        },
        table: {
          columns: {
            displayId: "Account ID",
            displayName: "Display Name",
            schoolName: "School",
            teamName: "Team",
            questionUsed: "Questions Used",
            questionLimit: "Question Limit",
            questionRemaining: "Remaining",
            questionProgress: "Progress",
            tokenUsed: "Tokens Used",
            status: "Status",
            remark: "Remark",
            lastUsedAt: "Last Used",
          },
          empty: {
            remark: "No remark",
          },
          status: {
            enabled: "Enabled",
            disabled: "Disabled",
          },
          query: {
            keyword: {
              label: "Keyword",
              placeholder: "Search account, school, team, or remark",
            },
            status: {
              label: "Status",
              placeholder: "Status",
            },
          },
          actions: {
            edit: {
              label: "Edit quota",
              title: "Edit account quota {{account}}",
              description: "Adjust this account's question limit, status, and remark.",
              success: "Account quota updated",
            },
            toggle: {
              enable: "Enable",
              disable: "Disable",
              enabledSuccess: "Account questions enabled",
              disabledSuccess: "Account questions disabled",
            },
          },
        },
        edit: {
          fields: {
            account: "Account",
            schoolName: "School",
            teamName: "Team",
            questionLimit: "Question Limit",
            status: "Status",
            remark: "Remark",
          },
          remarkPlaceholder: "Enter admin remark",
        },
      },
    },
    analytics: {
      heading: {
        title: "Access Analytics",
        description:
          "Review product traffic, sources, daily flow, accounts, schools, and team usage.",
      },
      filters: {
        from: "Start date",
        to: "End date",
        rangePlaceholder: "Choose analytics date range",
        rangeSummary: "Current range: {{from}} to {{to}}",
        rangeHint:
          "The selected range applies to overview, accounts, and school/team tabs.",
        refresh: "Refresh",
        presets: {
          label: "Quick analytics range",
          today: "Today",
          last7Days: "Last 7 days",
          last30Days: "Last 30 days",
        },
      },
      tabs: {
        overview: "Overview",
        accounts: "Accounts",
        schoolTeams: "Schools / Teams",
      },
      metrics: {
        pv: {
          label: "PV",
          tail: "Page views in the selected range.",
        },
        uv: {
          label: "UV",
          tail: "Deduplicated anonymous visitors and signed-in accounts.",
        },
        questions: {
          label: "AI Questions",
          tail: "Estimated tokens {{tokens}}.",
        },
        quality: {
          label: "Avg Session",
          tail: "Bounce {{bounce}}, errors {{errors}}.",
        },
      },
      charts: {
        traffic: {
          title: "Daily Traffic",
          description: "PV, UV, and AI questions by day.",
        },
        sources: {
          title: "Source Types",
          description: "Page views grouped by source type.",
        },
        pages: {
          title: "Top Pages",
          description: "Product pages with the most traffic.",
        },
        series: {
          pv: "PV",
          uv: "UV",
          sessions: "Sessions",
          questions: "AI Questions",
        },
      },
      sources: {
        kinds: {
          direct: "Direct",
          search: "Search",
          referral: "Referral",
          social: "Social",
          ad: "Ads / Campaigns",
          internal: "Internal",
          unknown: "Unknown",
        },
      },
      guides: {
        accounts: {
          title: "How to read account analytics",
          description:
            "Each row summarizes one product account: visits, sessions, active days, AI questions, and frequently used pages.",
        },
        schoolTeams: {
          title: "How to read school / team analytics",
          description:
            "Accounts under the same school and team are aggregated so managers can compare overall activity and AI usage.",
        },
      },
      routes: {
        unknown: {
          label: "Unknown Page",
          description:
            "No manager-friendly description is configured for this path yet. Use the raw path for traceability.",
        },
        known: {
          commerce: {
            label: "Commerce Copilot Workspace",
            description:
              "Main product page for commerce analysis and AI conversations.",
          },
          login: {
            label: "Sign-in Page",
            description: "Page where users enter account credentials.",
          },
          register: {
            label: "Registration Page",
            description: "Page where new users create product accounts.",
          },
          home: {
            label: "Home",
            description: "Product home or default entry page.",
          },
          overview: {
            label: "Business Overview",
            description: "Overview page for commerce operating metrics.",
          },
          materials: {
            label: "Materials Center",
            description: "Page for images, videos, copy, and commerce assets.",
          },
          products: {
            label: "Product Management",
            description: "Page for product data, selling points, and knowledge.",
          },
          campaigns: {
            label: "Campaigns",
            description: "Page for marketing activities and growth plans.",
          },
          workspaces: {
            label: "Workspaces",
            description: "Page for AI outputs, task materials, and artifacts.",
          },
          platformConnect: {
            label: "Platform Connections",
            description: "Page for commerce platform authorization status.",
          },
          approvals: {
            label: "Approvals",
            description: "Page for tool, publishing, or business action approvals.",
          },
          operations: {
            label: "Operations",
            description: "Page for operating tasks, todos, and results.",
          },
          agentOutput: {
            label: "AI Outputs",
            description: "Page for generated plans, reports, and reusable content.",
          },
          agentCapabilities: {
            label: "AI Capabilities",
            description: "Page explaining what the commerce agents can handle.",
          },
          questionIdeas: {
            label: "Question Ideas",
            description: "Business question examples users can ask the AI.",
          },
          profile: {
            label: "Profile",
            description: "Page where users view or edit personal information.",
          },
        },
      },
      tables: {
        common: {
          pv: "PV",
          uv: "UV",
          sessions: "Sessions",
          agentQuestions: "AI Questions",
          tokenUsed: "Tokens",
          lastSeenAt: "Last Seen",
        },
        pages: {
          title: "Page Details",
          description: "Entry, exit, duration, and error metrics by page.",
          columns: {
            pageLabel: "Page Name / Description",
            routePath: "Raw Path",
            avgDuration: "Avg Duration",
            entryCount: "Entries",
            exitCount: "Exits",
            errors: "Errors",
          },
        },
        accounts: {
          topPagePv: "{{pv}} visits",
          columns: {
            displayId: "Account ID",
            displayName: "Display Name",
            schoolName: "School",
            teamName: "Team",
            activeDays: "Active Days",
            topPages: "Top Pages",
          },
          query: {
            keyword: {
              label: "Keyword",
              placeholder: "Search account, name, school, or team",
            },
            schoolName: {
              label: "School",
              placeholder: "Filter by school",
            },
            teamName: {
              label: "Team",
              placeholder: "Filter by team",
            },
          },
        },
        schoolTeams: {
          columns: {
            schoolName: "School",
            teamName: "Team",
            accountCount: "Accounts",
            activeAccounts: "Active Accounts",
          },
          query: {
            schoolName: {
              label: "School",
              placeholder: "Filter by school",
            },
            teamName: {
              label: "Team",
              placeholder: "Filter by team",
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
              title: "Delete user role {{name}}",
              description:
                "Delete user role {{name}} ({{code}})? Make sure no users or permissions are still linked to it.",
              confirm: "Delete",
              success: "User role deleted",
            },
          },
        },
        create: {
          label: "New User Role",
          title: "Create User Role",
          description:
            "The code is the unique product user role identifier. Configure permissions on the User Role Permissions page after creating it.",
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
          success: "User role created",
        },
        note: {
          title: "User Role Creation Notes",
          line1:
            "Create product user roles such as Standard User or School Admin. The code is a unique identifier used by the system to distinguish roles.",
          line2:
            "New roles have no permissions by default. Grant permissions on the User Role Permissions page before the role takes effect.",
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
            "Create user roles, then configure their product feature permissions here.",
        },
        editor: {
          codePrefix: "Code: ",
          descriptionSuffix:
            ". The server returns the full permission tree and marks the nodes already granted to this role.",
          reset: "Reset Selection",
          save: "Save User Role Permissions",
          searchPlaceholder: "Search permissions",
          loading: "Loading permission tree.",
          empty: "No matching permissions.",
          success: "User role permissions saved",
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
    error: {
      title: "Sign-in failed",
      invalidCredentials: "The account or password is incorrect. Please try again.",
      serviceUnavailable: "The admin auth service is temporarily unavailable. Please try again later.",
      permissionUnavailable: "Admin permissions could not be loaded. Please try again or contact an administrator.",
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
