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
  const { contentClassName, ...profileOptions } = props
  const { profile, loading, loadError } = useAuthProfilePage(profileOptions)

  if (loading) {
    return <PageLoading label={t("profile.loading")} />
  }

  if (loadError) {
    return (
      <PageErrorState title={t("profile.errorTitle")} message={loadError} />
    )
  }

  return (
    <AuthProfileView
      contentClassName={contentClassName}
      profile={profile}
      labels={labels}
    />
  )
}

export function AuthProfileView({
  contentClassName,
  profile,
  labels,
}: AuthProfileViewProps) {
  return (
    <main
      className={["w-full text-(--app-text)", contentClassName]
        .filter(Boolean)
        .join(" ")}
    >
      <div className={contentClassName ? "w-full" : "mx-auto w-full max-w-4xl"}>
        <header className="border-b border-(--app-border)/70 pb-4">
          <h1 className="text-2xl/8 font-semibold tracking-normal">
            {labels.title}
          </h1>
        </header>

        <div className="grid gap-6 py-6 lg:grid-cols-[minmax(0,1fr)_17rem] lg:items-stretch">
          <div className="min-w-0 lg:h-full">
            <ProfileNameSection profile={profile} labels={labels} />
          </div>
          <ProfileAvatarSection profile={profile} labels={labels} />
          <div className="min-w-0 lg:col-span-2">
            <ProfileEmailSection profile={profile} labels={labels} />
          </div>
          <div className="min-w-0 lg:col-span-2">
            <ProfilePasswordSection profile={profile} labels={labels} />
          </div>
        </div>
      </div>
    </main>
  )
}
