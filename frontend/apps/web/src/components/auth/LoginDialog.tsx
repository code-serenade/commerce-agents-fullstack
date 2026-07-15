import { useEffect, useState } from "react"
import { useTranslation } from "react-i18next"
import {
  Dialog,
  DialogContent,
  DialogHeader,
  DialogTitle,
} from "@workspace/ui-components"
import type { AuthLoginSubmitValues } from "@workspace/app-kit/login"
import { AuthRegisterPage } from "@workspace/app-kit/register"
import { CommerceAgentLogo } from "@/components/brand"
import { MinimalLoginForm } from "@/components/auth/MinimalLoginForm"
import {
  type AuthLoginFlowError,
  useAuthLoginFlow,
} from "@/components/auth/use-auth-login-flow"
import { MinimalAuthSurface } from "@/layouts/auth/MinimalAuthSurface"
import { MinimalAuthTools } from "@/layouts/auth/MinimalAuthTools"
import { loginErrorCopyKey } from "@/pages/auth/login-error-copy"
import { useRegisterPage } from "@/pages/auth/register/use-register-page"

type AuthDialogMode = "login" | "register"

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
  const completeLogin = useAuthLoginFlow()
  const registerPage = useRegisterPage()
  const [mode, setMode] = useState<AuthDialogMode>("login")
  const [loginError, setLoginError] = useState<string | null>(null)
  const helpSteps = [
    t("login.help.steps.0"),
    t("login.help.steps.1"),
    t("login.help.steps.2"),
  ]

  useEffect(() => {
    if (open) {
      setMode("login")
      setLoginError(null)
    }
  }, [open])

  const handleLogin = async (values: AuthLoginSubmitValues) => {
    setLoginError(null)
    try {
      await completeLogin(values)
      onSuccess()
    } catch (error) {
      const loginFlowError = error as AuthLoginFlowError
      setLoginError(
        t(loginErrorCopyKey(loginFlowError.requestError, loginFlowError.stage))
      )
    }
  }

  return (
    <Dialog open={open} onOpenChange={onOpenChange}>
      <DialogContent className="w-[min(520px,calc(100vw-2rem))] max-w-none gap-0 overflow-visible rounded-2xl border-0 bg-white p-0 shadow-[0_24px_68px_rgba(38,76,160,0.18)] ring-0 *:data-[slot=dialog-close]:hidden sm:max-w-none dark:bg-[#0b1224]">
        <MinimalAuthSurface className="rounded-2xl">
          <DialogHeader className="border-b border-[rgba(209,220,240,0.56)] p-5 pr-36 text-left dark:border-[rgba(159,179,217,0.16)]">
            <DialogTitle>
              {mode === "login"
                ? t("login.title")
                : t("authRegister.title", { ns: "pages" })}
            </DialogTitle>
          </DialogHeader>

          <div className="absolute top-3 right-3 z-30 flex gap-2">
            <MinimalAuthTools
              helpLabel={t("login.help.label")}
              helpSteps={helpSteps}
            />
          </div>

          <div className="relative px-8 pt-9 pb-7">
            {mode === "login" ? (
              <MinimalLoginForm
                showBackToChat={false}
                notice={
                  loginError
                    ? { title: t("login.error.title"), description: loginError }
                    : undefined
                }
                onRegisterClick={() => {
                  setLoginError(null)
                  setMode("register")
                }}
                onSubmit={handleLogin}
              />
            ) : (
              <AuthRegisterPage
                brandLogo={<CommerceAgentLogo className="size-18" size="lg" />}
                showProfileFields={false}
                redirectAfterRegister={false}
                variant="minimal"
                onLoginClick={() => setMode("login")}
                onRegistered={async (values) => {
                  await registerPage.onRegistered?.(values)
                  try {
                    await completeLogin({
                      identifier: values.username,
                      password: values.password,
                    })
                    onSuccess()
                  } catch (error) {
                    const loginFlowError = error as AuthLoginFlowError
                    setMode("login")
                    setLoginError(
                      t(
                        loginErrorCopyKey(
                          loginFlowError.requestError,
                          loginFlowError.stage
                        )
                      )
                    )
                  }
                }}
              />
            )}
          </div>
        </MinimalAuthSurface>
      </DialogContent>
    </Dialog>
  )
}
