import type { ReactNode } from "react"
import {
  BarChart3,
  Gauge,
  LayoutDashboard,
  ShieldCheck,
  UsersRound,
} from "lucide-react"

export interface NavigationSubItemConfig {
  id: string
  permissionCode: string
  labelKey: string
  href: string
  matcher?: (pathname: string) => boolean
}

export interface NavigationItemConfig {
  id: string
  permissionCode?: string
  labelKey: string
  path?: string
  icon?: ReactNode
  subItems?: NavigationSubItemConfig[]
}

export interface NavigationSectionConfig {
  id: string
  labelKey: string
  items: NavigationItemConfig[]
}

export const navigationSections: NavigationSectionConfig[] = [
  {
    id: "platform",
    labelKey: "admin.navigation.platform",
    items: [
      {
        id: "dashboard",
        permissionCode: "dashboard",
        labelKey: "admin.navigation.dashboard",
        path: "/",
        icon: <LayoutDashboard />,
      },
      {
        id: "account-management",
        labelKey: "admin.navigation.accountManagement",
        icon: <UsersRound />,
        subItems: [
          {
            id: "admin-users",
            permissionCode: "accounts:admin_users",
            labelKey: "admin.navigation.adminUsers",
            href: "/accounts/admin-users",
            matcher: (pathname) => pathname.startsWith("/accounts/admin-users"),
          },
          {
            id: "app-users",
            permissionCode: "accounts:app_users",
            labelKey: "admin.navigation.appUsers",
            href: "/accounts/app-users",
            matcher: (pathname) => pathname.startsWith("/accounts/app-users"),
          },
        ],
      },
      {
        id: "access-control",
        labelKey: "admin.navigation.accessControl",
        icon: <ShieldCheck />,
        subItems: [
          {
            id: "roles",
            permissionCode: "access_control:roles",
            labelKey: "admin.navigation.roles",
            href: "/access/roles",
            matcher: (pathname) => pathname.startsWith("/access/roles"),
          },
          {
            id: "role-permissions",
            permissionCode: "access_control:role_permissions",
            labelKey: "admin.navigation.rolePermissions",
            href: "/access/role-permissions",
            matcher: (pathname) =>
              pathname.startsWith("/access/role-permissions"),
          },
          {
            id: "app-roles",
            permissionCode: "access_control:app_roles",
            labelKey: "admin.navigation.appRoles",
            href: "/access/app-roles",
            matcher: (pathname) => pathname.startsWith("/access/app-roles"),
          },
          {
            id: "app-role-permissions",
            permissionCode: "access_control:app_role_permissions",
            labelKey: "admin.navigation.appRolePermissions",
            href: "/access/app-role-permissions",
            matcher: (pathname) =>
              pathname.startsWith("/access/app-role-permissions"),
          },
        ],
      },
      {
        id: "analytics",
        labelKey: "admin.navigation.analytics",
        icon: <BarChart3 />,
        subItems: [
          {
            id: "access-analytics",
            permissionCode: "analytics:overview",
            labelKey: "admin.navigation.accessAnalytics",
            href: "/analytics/access",
            matcher: (pathname) => pathname.startsWith("/analytics/access"),
          },
        ],
      },
      {
        id: "usage-management",
        labelKey: "admin.navigation.usageManagement",
        icon: <Gauge />,
        subItems: [
          {
            id: "team-usage",
            permissionCode: "usage:team_usage",
            labelKey: "admin.navigation.teamUsage",
            href: "/usage/team-usage",
            matcher: (pathname) => pathname.startsWith("/usage/team-usage"),
          },
        ],
      },
    ],
  },
]
