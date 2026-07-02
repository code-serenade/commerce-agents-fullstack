import { useTranslation } from "react-i18next"
import { Badge } from "@workspace/ui-components/stable/badge"
import { Button } from "@workspace/ui-components/stable/button"
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
  onSave: () => void
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

  return (
    <section className="rounded-lg border border-(--app-border) bg-(--app-surface) p-5">
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

      <div className="mt-4 rounded-md border border-(--app-border) bg-muted/30 px-3 py-2 text-sm text-(--app-muted-text)">
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
            <div className="flex flex-wrap items-center gap-3">
              <Button onClick={onSave} disabled={isSaving}>
                {isSaving
                  ? t("profile.appProfile.saving")
                  : t("profile.appProfile.save")}
              </Button>
              {loadError || saveStatus ? (
                <p className="text-sm text-(--app-muted-text)">
                  {loadError || saveStatus}
                </p>
              ) : null}
            </div>
          </FieldGroup>
        </div>
      )}
    </section>
  )
}
