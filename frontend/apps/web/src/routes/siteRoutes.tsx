import { Suspense, type ReactNode } from "react"
import type { RouteObject } from "react-router"
import { PageLoading, RouteErrorBoundary } from "@workspace/app-kit"
import AppLayout from "@/layouts/AppLayout"
import {
  AgentCapabilitiesPage,
  AgentChatPage,
  AgentOutputPage,
  ApprovalsPage,
  CampaignsPage,
  CommerceOverviewPage,
  HomePage,
  OperationsPage,
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
      path: "agent-capabilities",
      element: withSuspense(<AgentCapabilitiesPage />),
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
