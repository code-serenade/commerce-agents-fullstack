import { Suspense } from "react"
import type { ReactNode } from "react"
import type { RouteObject } from "react-router"
import { PageLoading, RouteErrorBoundary } from "@workspace/app-kit"
import AppLayout from "@/layouts/AppLayout"
import {
  ButtonGuidePage,
  DataTableGuidePage,
  DisplayGuidePage,
  FeedbackGuidePage,
  FormGuidePage,
  NavigationGuidePage,
  ThemeGuidePage,
} from "@/routes/lazy/appPages"

function withSuspense(node: ReactNode) {
  return <Suspense fallback={<PageLoading />}>{node}</Suspense>
}

export const appRoutes: RouteObject = {
  path: "/",
  element: <AppLayout />,
  errorElement: <RouteErrorBoundary />,
  children: [
    {
      index: true,
      element: withSuspense(<DataTableGuidePage />),
    },
    {
      path: "theme",
      element: withSuspense(<ThemeGuidePage />),
    },
    {
      path: "button",
      element: withSuspense(<ButtonGuidePage />),
    },
    {
      path: "forms",
      element: withSuspense(<FormGuidePage />),
    },
    {
      path: "feedback",
      element: withSuspense(<FeedbackGuidePage />),
    },
    {
      path: "navigation",
      element: withSuspense(<NavigationGuidePage />),
    },
    {
      path: "display",
      element: withSuspense(<DisplayGuidePage />),
    },
  ],
}
