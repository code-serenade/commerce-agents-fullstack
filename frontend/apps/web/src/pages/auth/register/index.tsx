import { useNavigate } from "react-router"
import { AuthRegisterPage } from "@workspace/app-kit/register"
import { AuthBackToChatLink } from "@/components/auth/AuthBackToChatLink"
import {
  type AuthLoginFlowError,
  useAuthLoginFlow,
} from "@/components/auth/use-auth-login-flow"
import { CommerceAgentLogo } from "@/components/brand"
import { trackProductEvent } from "@/analytics/product-analytics"
import { useRegisterPage } from "./use-register-page"

export default function RegisterPage() {
  const navigate = useNavigate()
  const completeLogin = useAuthLoginFlow()
  const page = useRegisterPage()

  return (
    <>
      <AuthBackToChatLink />
      <AuthRegisterPage
        brandLogo={<CommerceAgentLogo className="size-18" size="lg" />}
        showProfileFields={false}
        redirectAfterRegister={false}
        variant="minimal"
        onRegistered={async (values) => {
          await page.onRegistered?.(values)
          try {
            await completeLogin({
              identifier: values.username,
              password: values.password,
            })
            trackProductEvent({
              eventType: "login_success",
              featureKey: "auth.login.success",
            })
            navigate("/commerce", { replace: true })
          } catch (error) {
            const loginFlowError = error as AuthLoginFlowError
            console.error("Auto login after register failed:", loginFlowError)
            navigate("/login", { replace: true })
          }
        }}
      />
    </>
  )
}
