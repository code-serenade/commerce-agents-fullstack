import { lazy } from "react"

export const LoginPage = lazy(() => import("@/pages/auth/Login"))
export const RegisterPage = lazy(async () => {
  const { AuthRegisterPage } = await import("@workspace/app-kit/register")
  return { default: AuthRegisterPage }
})
