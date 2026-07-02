import { Suspense, type ReactNode } from "react"
import type { RouteObject } from "react-router"
import { RouteErrorBoundary } from "@workspace/app-kit"
import AuthLayout from "@/layouts/AuthLayout"
import { LoginPage, RegisterPage } from "@/routes/lazy/authPages"
import { RouteLoadingFallback } from "@/routes/RouteLoadingFallback"

function withSuspense(node: ReactNode) {
  return <Suspense fallback={<RouteLoadingFallback />}>{node}</Suspense>
}

export const authRoutes: RouteObject = {
  path: "/",
  element: <AuthLayout />,
  errorElement: <RouteErrorBoundary />,
  children: [
    {
      path: "login",
      element: withSuspense(<LoginPage />),
    },
    {
      path: "register",
      element: withSuspense(<RegisterPage />),
    },
  ],
}
