import { useState } from "react"
import { useDispatch } from "react-redux"
import { useTranslation } from "react-i18next"
import {
  Dialog,
  DialogContent,
  DialogHeader,
  DialogTitle,
} from "@workspace/ui-components"
import type { AuthLoginSubmitValues } from "@workspace/app-kit/login"
import { ensureCurrentAppUserPermissions, loginApi, meApi } from "@/api"
import { resetAccess, setAccess } from "@/store/accessSlice"
import { loginSuccess, logout } from "@/store/authSlice"
import { MinimalLoginForm } from "@/components/auth/MinimalLoginForm"
import { MinimalAuthSurface } from "@/layouts/auth/MinimalAuthSurface"
import { MinimalAuthTools } from "@/layouts/auth/MinimalAuthTools"
import { toRequestError } from "@workspace/services/errors/request-error"
import {
  loginErrorCopyKey,
  type LoginStage,
} from "@/pages/auth/login-error-copy"

export function LoginDialog({
  open,
  onOpenChange,
  onSuccess,
}: {
  open: boolean
  onOpenChange: (open: boolean) => void
  onSuccess: () => void
}) {
  const { t } = useTranslation()
  const dispatch = useDispatch()
  const [loginError, setLoginError] = useState<string | null>(null)
  const helpSteps = [
    t("login.help.steps.0"),
    t("login.help.steps.1"),
    t("login.help.steps.2"),
  ]

  const handleLogin = async (values: AuthLoginSubmitValues) => {
    setLoginError(null)
    let stage: LoginStage = "credentials"
    try {
      const result = await loginApi(values)
      localStorage.setItem("token", result.accessToken)
      localStorage.setItem("refreshToken", result.refreshToken)
      stage = "profile"
      const user = await meApi()
      stage = "workspace"
      const access = await ensureCurrentAppUserPermissions(user)
      dispatch(loginSuccess({ token: result.accessToken, user }))
      dispatch(
        setAccess({
          roleCodes: access.roleCodes,
          permissionCodes: access.permissionCodes,
        })
      )
      onSuccess()
    } catch (error) {
      const requestError = toRequestError(error)
      localStorage.removeItem("token")
      localStorage.removeItem("refreshToken")
      dispatch(logout())
      dispatch(resetAccess())
      setLoginError(t(loginErrorCopyKey(requestError, stage)))
    }
  }

  return (
    <Dialog open={open} onOpenChange={onOpenChange}>
      <DialogContent className="w-[min(520px,calc(100vw-2rem))] max-w-none gap-0 overflow-visible rounded-2xl border-0 bg-white p-0 shadow-[0_24px_68px_rgba(38,76,160,0.18)] ring-0 *:data-[slot=dialog-close]:hidden sm:max-w-none dark:bg-[#0b1224]">
        <MinimalAuthSurface className="rounded-2xl">
          <DialogHeader className="border-b border-[rgba(209,220,240,0.56)] p-5 pr-36 text-left dark:border-[rgba(159,179,217,0.16)]">
            <DialogTitle>{t("login.title")}</DialogTitle>
          </DialogHeader>

          <div className="absolute top-3 right-3 z-30 flex gap-2">
            <MinimalAuthTools
              helpLabel={t("login.help.label")}
              helpSteps={helpSteps}
            />
          </div>

          <div className="relative px-8 pt-9 pb-7">
            <MinimalLoginForm
              notice={
                loginError
                  ? { title: t("login.error.title"), description: loginError }
                  : undefined
              }
              onRegisterClick={() => onOpenChange(false)}
              onSubmit={handleLogin}
            />
          </div>
        </MinimalAuthSurface>
      </DialogContent>
    </Dialog>
  )
}
