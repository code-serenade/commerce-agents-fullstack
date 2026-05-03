import { Suspense, type ReactNode } from "react"
import type { RouteObject } from "react-router"
import { PageLoading, RouteErrorBoundary } from "@workspace/app-kit"
import AppLayout from "@/layouts/AppLayout"
import {
  AgentChatPage,
  AgentOutputPage,
  HomePage,
  ProfilePage,
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
      index: true, // Default path `/`
      element: withSuspense(<HomePage />),
    },
    {
      path: "home",
      element: withSuspense(<HomePage />),
    },
    {
      path: "agent-output",
      element: withSuspense(<AgentOutputPage />),
    },
    {
      element: <RequireAuth />,
      children: [
        {
          path: "agent",
          element: withSuspense(<AgentChatPage />),
        },
        {
          path: "profile",
          element: withSuspense(<ProfilePage />),
        },
      ],
    },
  ],
}
