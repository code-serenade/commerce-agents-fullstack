import { lazy } from "react"

export const AdminUsersPage = lazy(
  () => import("@/pages/protected/accounts/admin-users")
)
export const AppRolePermissionsPage = lazy(
  () => import("@/pages/protected/access/app-role-permissions")
)
export const AppRolesPage = lazy(
  () => import("@/pages/protected/access/app-roles")
)
export const AppUsersPage = lazy(
  () => import("@/pages/protected/accounts/app-users")
)
export const AccessAnalyticsPage = lazy(
  () => import("@/pages/protected/analytics/access-analytics")
)
export const DashboardPage = lazy(() => import("@/pages/protected/dashboard"))
export const ProfilePage = lazy(() =>
  import("@/pages/protected/shared-pages").then((module) => ({
    default: module.ProfilePage,
  }))
)
export const RolePermissionsPage = lazy(
  () => import("@/pages/protected/access/role-permissions")
)
export const RolesPage = lazy(() => import("@/pages/protected/access/roles"))
export const TeamUsagePage = lazy(
  () => import("@/pages/protected/usage/team-usage")
)
