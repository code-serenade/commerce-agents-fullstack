import { AuthRegisterPage } from "@workspace/app-kit/register"
import { CommerceAgentLogo } from "@/components/brand"
import { useRegisterPage } from "./use-register-page"

export default function RegisterPage() {
  const page = useRegisterPage()

  return (
    <AuthRegisterPage
      brandLogo={<CommerceAgentLogo className="size-18" size="lg" />}
      extraFields={page.extraFields}
      variant="minimal"
      onRegistered={page.onRegistered}
    />
  )
}
