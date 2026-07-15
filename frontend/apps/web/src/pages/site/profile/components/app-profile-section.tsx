import { useEffect } from "react"
import { useTranslation } from "react-i18next"
import { Badge } from "@workspace/ui-components/stable/badge"
import {
  Field,
  FieldDescription,
  FieldGroup,
  FieldLabel,
} from "@workspace/ui-components/stable/field"
import { Input } from "@workspace/ui-components/stable/input"
import type { AppProfile } from "../profile-data"

export interface AppProfileSectionProps {
  profile: AppProfile | null
  schoolName: string
  teamName: string
  isLoading: boolean
  isSaving: boolean
  loadError: string
  saveStatus: string
  onSchoolNameChange: (value: string) => void
  onTeamNameChange: (value: string) => void
  onSave: () => Promise<void> | void
}

export function AppProfileSection({
  profile,
  schoolName,
  teamName,
  isLoading,
  isSaving,
  loadError,
  saveStatus,
  onSchoolNameChange,
  onTeamNameChange,
  onSave,
}: AppProfileSectionProps) {
  const { t } = useTranslation()
  const isBound = Boolean(profile?.isSchoolTeamBound)
  const savedSchoolName = profile?.schoolName ?? ""
  const savedTeamName = profile?.teamName ?? ""
  const hasProfileChange =
    schoolName.trim() !== savedSchoolName || teamName.trim() !== savedTeamName

  useEffect(() => {
    if (!profile || isLoading || isSaving || loadError || !hasProfileChange) {
      return
    }

    const timer = window.setTimeout(() => {
      void onSave()
    }, 800)

    return () => window.clearTimeout(timer)
  }, [
    hasProfileChange,
    isLoading,
    isSaving,
    loadError,
    onSave,
    profile,
    schoolName,
    teamName,
  ])

  return (
    <section className="rounded-xl border border-(--document-card-border) bg-(image:--document-card-background) p-5 shadow-(--document-card-shadow) backdrop-blur-md">
      <div className="flex flex-wrap items-start justify-between gap-3">
        <div className="min-w-0 space-y-1">
          <h2 className="text-lg font-semibold text-(--app-text)">
            {t("profile.appProfile.title")}
          </h2>
          <p className="text-sm text-(--app-muted-text)">
            {t("profile.appProfile.description")}
          </p>
        </div>
        <Badge variant={isBound ? "secondary" : "outline"}>
          {isBound
            ? t("profile.appProfile.boundStatus")
            : t("profile.appProfile.unboundStatus")}
        </Badge>
      </div>

      <div className="mt-4 rounded-lg border border-(--document-card-border) bg-(--agent-chat-content-surface) px-3 py-2 text-sm text-(--app-muted-text)">
        {t("profile.appProfile.quotaRule")}
      </div>

      {isLoading ? (
        <p className="mt-4 text-sm text-(--app-muted-text)">
          {t("profile.appProfile.loading")}
        </p>
      ) : (
        <div className="mt-5">
          <FieldGroup>
            <Field>
              <FieldLabel>{t("profile.appProfile.schoolName")}</FieldLabel>
              <Input
                value={schoolName}
                onValueChange={onSchoolNameChange}
                placeholder={t("profile.appProfile.schoolPlaceholder")}
                disabled={isSaving}
              />
            </Field>
            <Field>
              <FieldLabel>{t("profile.appProfile.teamName")}</FieldLabel>
              <Input
                value={teamName}
                onValueChange={onTeamNameChange}
                placeholder={t("profile.appProfile.teamPlaceholder")}
                disabled={isSaving}
              />
              <FieldDescription>
                {t("profile.appProfile.fieldHelp")}
              </FieldDescription>
            </Field>
            {isSaving || loadError || saveStatus ? (
              <div className="flex flex-wrap items-center gap-3">
                <p className="text-sm text-(--app-muted-text)">
                  {isSaving
                    ? t("profile.appProfile.saving")
                    : loadError || saveStatus}
                </p>
              </div>
            ) : null}
          </FieldGroup>
        </div>
      )}
    </section>
  )
}
