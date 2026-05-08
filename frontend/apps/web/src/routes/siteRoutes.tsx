import { Suspense, type ReactNode } from "react"
import type { RouteObject } from "react-router"
import { PageLoading, RouteErrorBoundary } from "@workspace/app-kit"
import AppLayout from "@/layouts/AppLayout"
import {
  AgentChatPage,
  AgentOutputPage,
  ApprovalsPage,
  CampaignsPage,
  CommerceOverviewPage,
  HomePage,
  OperationsPage,
  PackagesPage,
  PlatformConnectPage,
  ProductsPage,
  ProfilePage,
  WorkspacesPage,
} from "@/routes/lazy/sitePages"
import { RequireAuth } from "@/routes/RequireAuth"

function withSuspense(node: ReactNode) {
  return <Suspense fallback={<PageLoading />}>{node}</Suspense>
}

export const siteRoutes: RouteObject = {
  path: "/",
  element: <AppLayout />,
  errorElement: <RouteErrorBoundary />,
  children: [
    {
      index: true,
      element: withSuspense(<AgentChatPage />),
    },
    {
      path: "home",
      element: withSuspense(<HomePage />),
    },
    {
      path: "commerce",
      element: withSuspense(<AgentChatPage />),
    },
    {
      path: "agent",
      element: withSuspense(<AgentChatPage />),
    },
    {
      element: <RequireAuth />,
      children: [
        {
          path: "overview",
          element: withSuspense(<CommerceOverviewPage />),
        },
        {
          path: "products",
          element: withSuspense(<ProductsPage />),
        },
        {
          path: "campaigns",
          element: withSuspense(<CampaignsPage />),
        },
        {
          path: "workspaces",
          element: withSuspense(<WorkspacesPage />),
        },
        {
          path: "platform-connect",
          element: withSuspense(<PlatformConnectPage />),
        },
        {
          path: "approvals",
          element: withSuspense(<ApprovalsPage />),
        },
        {
          path: "operations",
          element: withSuspense(<OperationsPage />),
        },
        {
          path: "packages",
          element: withSuspense(<PackagesPage />),
        },
        {
          path: "agent-output",
          element: withSuspense(<AgentOutputPage />),
        },
        {
          path: "profile",
          element: withSuspense(<ProfilePage />),
        },
      ],
    },
  ],
}
