import { Suspense, type ReactNode } from "react"
import type { RouteObject } from "react-router"
import { PageLoading, RouteErrorBoundary } from "@workspace/app-kit"
import AuthLayout from "@/layouts/AuthLayout"
import { LoginPage, RegisterPage } from "@/routes/lazy/authPages"

function withSuspense(node: ReactNode) {
  return <Suspense fallback={<PageLoading />}>{node}</Suspense>
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
