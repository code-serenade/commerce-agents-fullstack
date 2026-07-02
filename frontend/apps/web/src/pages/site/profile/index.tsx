import { AuthProfilePage } from "@workspace/app-kit/profile"
import { AppProfileSection } from "./components/app-profile-section"
import { useProfilePage } from "./use-profile-page"

export default function ProfilePage() {
  const page = useProfilePage()

  return (
    <div className="min-h-full bg-background">
      <AuthProfilePage />
      <div className="w-full px-4 pb-8 sm:px-6 lg:px-8">
        <div className="mx-auto w-full max-w-6xl">
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
