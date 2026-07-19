import { useLocation, useNavigate } from "react-router"
import { useState } from "react"
import { useTranslation } from "react-i18next"
import type { AuthLoginSubmitValues } from "@workspace/app-kit/login"
import { MinimalLoginForm } from "@/components/auth/MinimalLoginForm"
import {
  type AuthLoginFlowError,
  useAuthLoginFlow,
} from "@/components/auth/use-auth-login-flow"
import { trackProductEvent } from "@/analytics/product-analytics"
import { loginErrorCopyKey } from "./login-error-copy"

export default function LoginPage() {
  const { t } = useTranslation()
  const navigate = useNavigate()
  const location = useLocation()
  const completeLogin = useAuthLoginFlow()
  const [loginError, setLoginError] = useState<string | null>(null)
  const from = location.state?.from?.pathname ?? "/commerce"

  const handleLogin = async (values: AuthLoginSubmitValues) => {
    setLoginError(null)
    trackProductEvent({
      eventType: "login_click",
      featureKey: "auth.login.submit",
    })
    try {
      await completeLogin(values)
      trackProductEvent({
        eventType: "login_success",
        featureKey: "auth.login.success",
      })

      navigate(from, { replace: true })
    } catch (err) {
      const loginFlowError = err as AuthLoginFlowError
      console.error("Login failed:", loginFlowError.requestError)
      setLoginError(
        t(loginErrorCopyKey(loginFlowError.requestError, loginFlowError.stage))
      )
    }
  }

  return (
    <MinimalLoginForm
      notice={
        loginError
          ? {
              title: t("login.error.title"),
              description: loginError,
            }
          : undefined
      }
      onSubmit={handleLogin}
    />
  )
}
