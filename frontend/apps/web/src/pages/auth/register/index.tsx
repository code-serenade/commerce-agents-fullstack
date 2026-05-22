import { AuthRegisterPage } from "@workspace/app-kit/register"
import { CommerceAgentLogo } from "@/components/brand"

export default function RegisterPage() {
  return <AuthRegisterPage brandLogo={<CommerceAgentLogo size="lg" />} />
}
