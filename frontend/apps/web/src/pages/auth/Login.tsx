import { Link, useLocation, useNavigate } from "react-router"
import { useDispatch } from "react-redux"
import { useState } from "react"
import { useTranslation } from "react-i18next"
import { toRequestError } from "@workspace/services/errors/request-error"
import {
  AuthLoginView,
  type AuthLoginSubmitValues,
  type AuthLoginViewLabels,
} from "@workspace/app-kit/login"
import {
  ensureCurrentAppUserPermissions,
  loginApi,
  meApi,
} from "@/api"
import { resetAccess, setAccess } from "@/store/accessSlice"
import { loginSuccess, logout } from "@/store/authSlice"

const authLinkClassName =
  "font-medium text-(--app-text) underline-offset-4 transition hover:underline"

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
  const [loginError, setLoginError] = useState<string | null>(null)
  const from = location.state?.from?.pathname ?? "/commerce"

  const handleLogin = async (values: AuthLoginSubmitValues) => {
    setLoginError(null)
    try {
      const res = await loginApi(values)
      const token = res.accessToken
      const refreshToken = res.refreshToken

      localStorage.setItem("token", token)
      localStorage.setItem("refreshToken", refreshToken)

      const user = await meApi()
      const access = await ensureCurrentAppUserPermissions(user)

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
      setLoginError(t(loginErrorCopyKey(requestError.status)))
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
          : undefined
      }
      footerLink={{
        prefix: t("login.footer.toRegisterPrefix"),
        action: (
          <Link to="/register" className={authLinkClassName}>
            {t("login.footer.toRegisterAction")}
          </Link>
        ),
      }}
      onSubmit={handleLogin}
    />
  )
}

function loginErrorCopyKey(status?: number) {
  if (status === 400 || status === 401 || status === 403) {
    return "login.error.invalidCredentials"
  }

  return "login.error.workspaceUnavailable"
}
