import { useTranslation } from "react-i18next"
import { PageErrorState, PageLoading } from "../../components/feedback"
import { ProfileAvatarSection } from "./profile-avatar-section"
import { ProfileEmailSection } from "./profile-email-section"
import { ProfileNameSection } from "./profile-name-section"
import { ProfilePasswordSection } from "./profile-password-section"
import { useAuthProfileLabels } from "./use-auth-profile-labels"
import { useAuthProfilePage } from "./use-auth-profile-page"
import type { AuthProfilePageProps, AuthProfileViewProps } from "./types"

export function AuthProfilePage(props: AuthProfilePageProps) {
  const { t } = useTranslation("pages")
  const labels = useAuthProfileLabels()
  const { profile, loading, loadError } = useAuthProfilePage(props)

  if (loading) {
    return <PageLoading label={t("profile.loading")} />
  }

  if (loadError) {
    return (
      <PageErrorState title={t("profile.errorTitle")} message={loadError} />
    )
  }

  return <AuthProfileView profile={profile} labels={labels} />
}

export function AuthProfileView({ profile, labels }: AuthProfileViewProps) {
  return (
    <main className="w-full px-4 py-5 text-(--app-text) sm:px-6 lg:px-8">
      <div className="mx-auto w-full max-w-6xl">
        <header className="border-b border-(--app-border) pb-4">
          <h1 className="text-2xl font-semibold tracking-normal">
            {labels.title}
          </h1>
        </header>

        <div className="grid gap-10 py-6 lg:grid-cols-[minmax(0,1fr)_280px] lg:items-start">
          <div className="space-y-8">
            <ProfileNameSection profile={profile} labels={labels} />
            <ProfileEmailSection profile={profile} labels={labels} />
            <ProfilePasswordSection profile={profile} labels={labels} />
          </div>
          <ProfileAvatarSection profile={profile} labels={labels} />
        </div>
      </div>
    </main>
  )
}
