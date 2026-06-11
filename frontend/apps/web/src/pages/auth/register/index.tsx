import { AuthRegisterPage } from "@workspace/app-kit/register"
import { CommerceAgentLogo } from "@/components/brand"
import { useRegisterPage } from "./use-register-page"

export default function RegisterPage() {
  const page = useRegisterPage()

  return (
    <AuthRegisterPage
      brandLogo={<CommerceAgentLogo size="lg" />}
      extraFields={page.extraFields}
      onRegistered={page.onRegistered}
    />
  )
}
