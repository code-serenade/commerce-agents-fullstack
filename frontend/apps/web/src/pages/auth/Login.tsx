import { useLocation, useNavigate } from "react-router"
import { useDispatch } from "react-redux"
import { useState } from "react"
import { useTranslation } from "react-i18next"
import { toRequestError } from "@workspace/services/errors/request-error"
import type { AuthLoginSubmitValues } from "@workspace/app-kit/login"
import { ensureCurrentAppUserPermissions, loginApi, meApi } from "@/api"
import { resetAccess, setAccess } from "@/store/accessSlice"
import { loginSuccess, logout } from "@/store/authSlice"
import { MinimalLoginForm } from "@/components/auth/MinimalLoginForm"
import { trackProductEvent } from "@/analytics/product-analytics"
import { loginErrorCopyKey, type LoginStage } from "./login-error-copy"

export default function LoginPage() {
  const { t } = useTranslation()
  const dispatch = useDispatch()
  const navigate = useNavigate()
  const location = useLocation()
  const [loginError, setLoginError] = useState<string | null>(null)
  const from = location.state?.from?.pathname ?? "/commerce"

  const handleLogin = async (values: AuthLoginSubmitValues) => {
    setLoginError(null)
    trackProductEvent({
      eventType: "login_click",
      featureKey: "auth.login.submit",
    })
    let loginStage: LoginStage = "credentials"
    try {
      const res = await loginApi(values)
      const token = res.accessToken
      const refreshToken = res.refreshToken

      localStorage.setItem("token", token)
      localStorage.setItem("refreshToken", refreshToken)

      loginStage = "profile"
      const user = await meApi()
      loginStage = "workspace"
      const access = await ensureCurrentAppUserPermissions(user)

      dispatch(loginSuccess({ token, user }))
      dispatch(
        setAccess({
          roleCodes: access.roleCodes,
          permissionCodes: access.permissionCodes,
        })
      )
      trackProductEvent({
        eventType: "login_success",
        featureKey: "auth.login.success",
      })

      navigate(from, { replace: true })
    } catch (err) {
      const requestError = toRequestError(err)
      console.error("Login failed:", requestError)
      localStorage.removeItem("token")
      localStorage.removeItem("refreshToken")
      dispatch(logout())
      dispatch(resetAccess())
      setLoginError(t(loginErrorCopyKey(requestError, loginStage)))
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
