import { useMemo } from "react"
import { useTranslation } from "react-i18next"
import type { AuthRegisterPageProps } from "@workspace/app-kit/register"
import { trackProductEvent } from "@/analytics/product-analytics"
import { savePendingRegistrationProfile } from "@/api"

export function useRegisterPage() {
  const { t } = useTranslation()
  const extraFields = useMemo<AuthRegisterPageProps["extraFields"]>(
    () => [
      {
        name: "schoolName",
        label: t("register.profile.schoolName.label"),
        placeholder: t("register.profile.schoolName.placeholder"),
        helpText: t("register.profile.schoolName.help"),
        maxLength: 128,
        required: false,
      },
      {
        name: "teamName",
        label: t("register.profile.teamName.label"),
        placeholder: t("register.profile.teamName.placeholder"),
        helpText: t("register.profile.teamName.help"),
        maxLength: 128,
        required: false,
      },
    ],
    [t]
  )

  const onRegistered: AuthRegisterPageProps["onRegistered"] = (values) => {
    trackProductEvent({
      eventType: "register_success",
      featureKey: "auth.register.success",
    })
    savePendingRegistrationProfile({
      username: values.username,
      displayName: values.displayName,
      email: values.email,
      schoolName: values.extraFields.schoolName,
      teamName: values.extraFields.teamName,
    })
  }

  return {
    extraFields,
    onRegistered,
  }
}
