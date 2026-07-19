import type { AuthRegisterPageProps } from "@workspace/app-kit/register"
import { trackProductEvent } from "@/analytics/product-analytics"
import { savePendingRegistrationProfile } from "@/api"

export function useRegisterPage() {
  const onRegistered: AuthRegisterPageProps["onRegistered"] = (values) => {
    trackProductEvent({
      eventType: "register_success",
      featureKey: "auth.register.success",
    })
    savePendingRegistrationProfile({
      username: values.username,
      displayName: values.displayName,
      email: values.email,
      schoolName: "",
      teamName: "",
    })
  }

  return {
    onRegistered,
  }
}
