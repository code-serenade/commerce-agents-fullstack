import { Suspense, type ReactNode } from "react"
import { Navigate, type RouteObject } from "react-router"
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
      index: true,
      element: <Navigate to="/login" replace />,
    },
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
