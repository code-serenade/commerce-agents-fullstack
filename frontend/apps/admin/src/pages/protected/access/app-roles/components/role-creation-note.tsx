import { useTranslation } from "react-i18next"
import { AppNotice } from "@workspace/app-kit"

export function RoleCreationNote() {
  const { t } = useTranslation()

  return (
    <AppNotice
      title={t("admin.access.appRoles.note.title")}
      variant="warning"
      description={
        <span className="grid gap-1">
          <span>{t("admin.access.appRoles.note.line1")}</span>
          <span>{t("admin.access.appRoles.note.line2")}</span>
        </span>
      }
    />
  )
}
