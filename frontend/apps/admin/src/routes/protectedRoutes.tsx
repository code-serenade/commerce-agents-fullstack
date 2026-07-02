import { Suspense, type ReactNode } from "react"
import type { RouteObject } from "react-router"
import { Navigate } from "react-router"
import { RouteErrorBoundary } from "@workspace/app-kit"
import AppLayout from "@/layouts/AppLayout"
import {
  AdminUsersPage,
  AccessAnalyticsPage,
  AppRolePermissionsPage,
  AppRolesPage,
  AppUsersPage,
  DashboardPage,
  ProfilePage,
  RolePermissionsPage,
  RolesPage,
  TeamUsagePage,
} from "@/routes/lazy/protectedPages"
import { RouteLoadingFallback } from "@/routes/RouteLoadingFallback"

function withSuspense(node: ReactNode) {
  return <Suspense fallback={<RouteLoadingFallback />}>{node}</Suspense>
}

export const protectedRoutes: RouteObject = {
  path: "/",
  element: <AppLayout />,
  errorElement: <RouteErrorBoundary />,
  children: [
    {
      index: true, // Default path `/`
      element: withSuspense(<DashboardPage />),
    },
    {
      path: "access",
      element: <Navigate to="/access/roles" replace />,
    },
    {
      path: "accounts",
      element: <Navigate to="/accounts/admin-users" replace />,
    },
    {
      path: "usage",
      element: <Navigate to="/usage/team-usage" replace />,
    },
    {
      path: "analytics",
      element: <Navigate to="/analytics/access" replace />,
    },
    {
      path: "accounts/admin-users",
      element: withSuspense(<AdminUsersPage />),
    },
    {
      path: "access/roles",
      element: withSuspense(<RolesPage />),
    },
    {
      path: "access/role-permissions",
      element: withSuspense(<RolePermissionsPage />),
    },
    {
      path: "accounts/app-users",
      element: withSuspense(<AppUsersPage />),
    },
    {
      path: "profile",
      element: withSuspense(<ProfilePage />),
    },
    {
      path: "access/app-roles",
      element: withSuspense(<AppRolesPage />),
    },
    {
      path: "access/app-role-permissions",
      element: withSuspense(<AppRolePermissionsPage />),
    },
    {
      path: "usage/team-usage",
      element: withSuspense(<TeamUsagePage />),
    },
    {
      path: "analytics/access",
      element: withSuspense(<AccessAnalyticsPage />),
    },
  ],
}
