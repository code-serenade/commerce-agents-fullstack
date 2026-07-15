import { AppProfileSection } from "../profile/components/app-profile-section"
import { useProfilePage } from "../profile/use-profile-page"
import { useTranslation } from "react-i18next"

export default function TeamSchoolPage() {
  const { t } = useTranslation()
  const page = useProfilePage()

  return (
    <div className="min-h-full bg-background">
      <div className="mx-auto w-full max-w-4xl px-4 py-5 text-(--app-text) sm:px-6 lg:px-8">
        <header className="border-b border-(--app-border)/70 pb-4">
          <h1 className="text-2xl/8 font-semibold tracking-normal">
            {t("profile.teamSchoolPage.title")}
          </h1>
          <p className="mt-1 max-w-2xl text-sm text-(--app-muted-text)">
            {t("profile.teamSchoolPage.description")}
          </p>
        </header>
        <div className="py-6">
          <AppProfileSection
            profile={page.appProfile.profile}
            schoolName={page.appProfile.schoolName}
            teamName={page.appProfile.teamName}
            isLoading={page.appProfile.isLoading}
            isSaving={page.appProfile.isSaving}
            loadError={page.appProfile.loadError}
            saveStatus={page.appProfile.saveStatus}
            onSchoolNameChange={page.appProfile.setSchoolName}
            onTeamNameChange={page.appProfile.setTeamName}
            onSave={() => void page.appProfile.save()}
          />
        </div>
      </div>
    </div>
  )
}
