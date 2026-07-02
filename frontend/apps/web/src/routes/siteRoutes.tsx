import { Suspense, type ReactNode } from "react"
import type { RouteObject } from "react-router"
import { RouteErrorBoundary } from "@workspace/app-kit"
import AppLayout from "@/layouts/AppLayout"
import {
  AgentCapabilitiesPage,
  AgentChatPage,
  AgentOutputPage,
  ApprovalsPage,
  CampaignsPage,
  CommerceOverviewPage,
  FilesPage,
  HomePage,
  OperationsPage,
  PlatformConnectPage,
  ProductsPage,
  QuestionIdeasPage,
  ProfilePage,
  WorkspacesPage,
} from "@/routes/lazy/sitePages"
import { RequireAuth } from "@/routes/RequireAuth"
import { RouteLoadingFallback } from "@/routes/RouteLoadingFallback"

function withSuspense(node: ReactNode) {
  return <Suspense fallback={<RouteLoadingFallback />}>{node}</Suspense>
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
      path: "question-ideas",
      element: withSuspense(<QuestionIdeasPage />),
    },
    {
      element: <RequireAuth />,
      children: [
        {
          path: "overview",
          element: withSuspense(<CommerceOverviewPage />),
        },
        {
          path: "materials",
          element: withSuspense(<FilesPage />),
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
