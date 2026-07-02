import { useLocation, useNavigate } from "react-router"
import { useDispatch } from "react-redux"
import { useState } from "react"
import { useTranslation } from "react-i18next"
import {
  AuthLoginView,
  type AuthLoginSubmitValues,
  type AuthLoginViewLabels,
} from "@workspace/app-kit/login"
import { toRequestError } from "@workspace/services/errors/request-error"
import { getCurrentUserPermissionsApi, loginApi, meApi } from "@/api"
import { resetAccess, setAccess } from "@/store/accessSlice"
import { loginSuccess, logout } from "@/store/authSlice"

function createLoginLabels(
  t: ReturnType<typeof useTranslation>["t"]
): AuthLoginViewLabels {
  return {
    brand: t("login.brand"),
    heroTitleLine1: t("login.hero.titleLine1"),
    heroTitleLine2: t("login.hero.titleLine2"),
    heroDescriptions: [t("login.hero.desc1"), t("login.hero.desc2")],
    welcome: t("login.welcome"),
    title: t("login.title"),
    subtitle: t("login.subtitle"),
    identifierLabel: t("login.form.identifier.label"),
    identifierPlaceholder: t("login.form.identifier.placeholder"),
    passwordLabel: t("login.form.password.label"),
    passwordPlaceholder: t("login.form.password.placeholder"),
    submit: t("login.form.submit"),
    submitting: t("login.form.submitting"),
  }
}

export default function LoginPage() {
  const { t } = useTranslation()
  const dispatch = useDispatch()
  const navigate = useNavigate()
  const location = useLocation()
  const from = readLoginRedirectTarget(location.state)
  const [loginError, setLoginError] = useState<string | null>(null)

  const handleLogin = async (values: AuthLoginSubmitValues) => {
    setLoginError(null)
    try {
      const res = await loginApi(values)
      const token = res.accessToken
      const refreshToken = res.refreshToken

      localStorage.setItem("token", token)
      localStorage.setItem("refreshToken", refreshToken)

      const [user, access] = await Promise.all([
        meApi(),
        getCurrentUserPermissionsApi(),
      ])

      dispatch(loginSuccess({ token, user }))
      dispatch(
        setAccess({
          roleCodes: access.roleCodes,
          permissionCodes: access.permissionCodes,
        })
      )

      navigate(from, { replace: true })
    } catch (err) {
      const requestError = toRequestError(err)
      console.error("Login failed:", requestError)
      localStorage.removeItem("token")
      localStorage.removeItem("refreshToken")
      dispatch(logout())
      dispatch(resetAccess())
      setLoginError(t(readAdminLoginErrorCopyKey(requestError)))
    }
  }

  return (
    <AuthLoginView
      labels={createLoginLabels(t)}
      notice={
        loginError
          ? {
              title: t("login.error.title"),
              description: loginError,
            }
          : {
              title: t("login.notice.title"),
              description: t("login.notice.description"),
            }
      }
      onSubmit={handleLogin}
    />
  )
}

function readAdminLoginErrorCopyKey(
  error: ReturnType<typeof toRequestError>
) {
  const status = error.status
  if (status === 400 || status === 401 || status === 403) {
    return "login.error.invalidCredentials"
  }

  if (
    error.kind === "network" ||
    status === 502 ||
    status === 503 ||
    status === 504
  ) {
    return "login.error.serviceUnavailable"
  }

  return "login.error.permissionUnavailable"
}

function readLoginRedirectTarget(state: unknown) {
  if (!isLoginRedirectState(state)) return "/"
  const { pathname, search = "", hash = "" } = state.from
  return `${pathname}${search}${hash}`
}

function isLoginRedirectState(state: unknown): state is {
  from: { hash?: string; pathname: string; search?: string }
} {
  if (!state || typeof state !== "object") return false
  const from = (state as { from?: unknown }).from
  if (!from || typeof from !== "object") return false
  return typeof (from as { pathname?: unknown }).pathname === "string"
}
