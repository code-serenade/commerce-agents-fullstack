import { AuthRegisterPage } from "@workspace/app-kit/register"
import { TaskMateLogo } from "@/components/brand"

export default function RegisterPage() {
  return <AuthRegisterPage brandLogo={<TaskMateLogo size="lg" />} />
}
