import { useMemo } from "react"
import { useTranslation } from "react-i18next"
import type { AuthRegisterPageProps } from "@workspace/app-kit/register"
import { savePendingRegistrationProfile } from "@/api"

export function useRegisterPage() {
  const { t } = useTranslation()
  const extraFields = useMemo<AuthRegisterPageProps["extraFields"]>(
    () => [
      {
        name: "schoolName",
        label: t("register.profile.schoolName.label"),
        placeholder: t("register.profile.schoolName.placeholder"),
        maxLength: 128,
      },
      {
        name: "teamName",
        label: t("register.profile.teamName.label"),
        placeholder: t("register.profile.teamName.placeholder"),
        maxLength: 128,
      },
    ],
    [t]
  )

  const onRegistered: AuthRegisterPageProps["onRegistered"] = (values) => {
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
