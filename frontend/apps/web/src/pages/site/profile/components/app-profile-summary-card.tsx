import { useTranslation } from "react-i18next"
import { Badge } from "@workspace/ui-components/stable/badge"
import { Button } from "@workspace/ui-components/stable/button"
import type { AppProfile } from "../profile-data"

export interface AppProfileSummaryCardProps {
  profile: AppProfile | null
  schoolName: string
  teamName: string
  isLoading: boolean
  loadError: string
  onManage: () => void
}

export function AppProfileSummaryCard({
  profile,
  schoolName,
  teamName,
  isLoading,
  loadError,
  onManage,
}: AppProfileSummaryCardProps) {
  const { t } = useTranslation()
  const isBound = Boolean(profile?.isSchoolTeamBound)
  const normalizedSchoolName = schoolName.trim()
  const normalizedTeamName = teamName.trim()

  return (
    <section className="rounded-xl border border-(--document-card-border) bg-(image:--document-card-background) p-5 shadow-(--document-card-shadow) backdrop-blur-md">
      <div className="flex flex-wrap items-start justify-between gap-3">
        <div className="min-w-0 space-y-1">
          <h2 className="text-lg font-semibold text-(--app-text)">
            {t("profile.appProfile.summaryTitle")}
          </h2>
          <p className="text-sm text-(--app-muted-text)">
            {t("profile.appProfile.summaryDescription")}
          </p>
        </div>
        <Badge variant={isBound ? "secondary" : "outline"}>
          {isBound
            ? t("profile.appProfile.boundStatus")
            : t("profile.appProfile.unboundStatus")}
        </Badge>
      </div>

      {isLoading ? (
        <p className="mt-4 text-sm text-(--app-muted-text)">
          {t("profile.appProfile.loading")}
        </p>
      ) : (
        <div className="mt-5 space-y-4">
          {loadError ? (
            <p className="text-sm text-(--app-muted-text)">{loadError}</p>
          ) : (
            <div className="grid gap-3 text-sm sm:grid-cols-2">
              <ProfileSummaryField
                label={t("profile.appProfile.schoolName")}
                value={
                  normalizedSchoolName || t("profile.appProfile.unsetValue")
                }
              />
              <ProfileSummaryField
                label={t("profile.appProfile.teamName")}
                value={normalizedTeamName || t("profile.appProfile.unsetValue")}
              />
            </div>
          )}

          <div className="flex flex-wrap items-center gap-3">
            <Button onClick={onManage}>
              {isBound
                ? t("profile.appProfile.manageTeamSchool")
                : t("profile.appProfile.setupTeamSchool")}
            </Button>
            {!isBound && !loadError ? (
              <p className="text-sm text-(--app-muted-text)">
                {t("profile.appProfile.summaryHelp")}
              </p>
            ) : null}
          </div>
        </div>
      )}
    </section>
  )
}

function ProfileSummaryField({
  label,
  value,
}: {
  label: string
  value: string
}) {
  return (
    <div className="rounded-lg border border-(--document-card-border) bg-(--agent-chat-content-surface) px-3 py-2">
      <div className="text-xs font-medium text-(--app-muted-text)">{label}</div>
      <div className="mt-1 truncate font-medium text-(--app-text)">{value}</div>
    </div>
  )
}
