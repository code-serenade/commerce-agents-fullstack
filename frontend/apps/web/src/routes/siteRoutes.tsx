import { Suspense, type ReactNode } from "react"
import type { RouteObject } from "react-router"
import { PageLoading, RouteErrorBoundary } from "@workspace/app-kit"
import AppLayout from "@/layouts/AppLayout"
import { AgentChatPage } from "@/routes/lazy/sitePages"
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
      element: <RequireAuth />,
      children: [
        {
          index: true,
          element: withSuspense(<AgentChatPage />),
        },
        {
          path: "home",
          element: withSuspense(<AgentChatPage />),
        },
        {
          path: "agent",
          element: withSuspense(<AgentChatPage />),
        },
        {
          path: "agent-output",
          element: withSuspense(<AgentChatPage />),
        },
        {
          path: "profile",
          element: withSuspense(<AgentChatPage />),
        },
      ],
    },
  ],
}
