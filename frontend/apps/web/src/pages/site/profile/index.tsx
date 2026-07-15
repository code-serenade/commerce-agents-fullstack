import { AuthProfilePage } from "@workspace/app-kit/profile"
import { useNavigate } from "react-router"
import { AppProfileSummaryCard } from "./components/app-profile-summary-card"
import { useProfilePage } from "./use-profile-page"

export default function ProfilePage() {
  const page = useProfilePage()
  const navigate = useNavigate()

  return (
    <div className="min-h-full bg-background">
      <div className="mx-auto w-full max-w-4xl px-4 py-5 text-(--app-text) sm:px-6 lg:px-8">
        <AuthProfilePage contentClassName="px-0 py-0" />
        <div className="pt-2 pb-8">
          <AppProfileSummaryCard
            profile={page.appProfile.profile}
            schoolName={page.appProfile.schoolName}
            teamName={page.appProfile.teamName}
            isLoading={page.appProfile.isLoading}
            loadError={page.appProfile.loadError}
            onManage={() => navigate("/team-school")}
          />
        </div>
      </div>
    </div>
  )
}
