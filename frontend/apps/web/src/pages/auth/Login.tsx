import { Link, useLocation, useNavigate } from "react-router"
import { useDispatch } from "react-redux"
import { useTranslation } from "react-i18next"
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
import { setAccess } from "@/store/accessSlice"
import { loginSuccess } from "@/store/authSlice"

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
  const from = location.state?.from?.pathname ?? "/home"

  const handleLogin = async (values: AuthLoginSubmitValues) => {
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
      console.error("Login failed:", err)
    }
  }

  return (
    <AuthLoginView
      labels={createLoginLabels(t)}
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
