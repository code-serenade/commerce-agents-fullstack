import { ArrowRight, LogIn, ShieldCheck, UserRound } from "lucide-react"
import { useTranslation } from "react-i18next"
import { useNavigate } from "react-router"
import { Avatar } from "@workspace/ui-components/stable/avatar"
import { Button } from "@workspace/ui-components/stable/button"
import type { UserInfo } from "@/models/userModel"

export interface HomeOverviewProps {
  user: UserInfo | null
  displayName: string
  displayId: string
  avatarFallback: string
}

export function HomeOverview({
  user,
  displayName,
  displayId,
  avatarFallback,
}: HomeOverviewProps) {
  const { t } = useTranslation()
  const navigate = useNavigate()
  const isLogin = Boolean(user)

  return (
    <div className="mx-auto flex w-full max-w-5xl flex-col gap-6">
      <section className="rounded-lg border border-(--app-border) bg-(--app-surface) p-6 shadow-(--ui-shadow-soft) sm:p-8">
        <div className="flex flex-col gap-6 sm:flex-row sm:items-center sm:justify-between">
          <div className="flex min-w-0 items-center gap-4">
            <Avatar
              src={user?.avatar}
              alt={displayName}
              fallback={avatarFallback}
            />
            <div className="min-w-0 space-y-1">
              <p className="text-sm font-medium text-(--app-muted-text)">
                {t(isLogin ? "home.welcome" : "home.publicWelcome")}
              </p>
              <h1 className="truncate text-2xl font-semibold text-(--app-text) sm:text-3xl">
                {isLogin ? displayName || t("home.fallbackName") : t("home.publicTitle")}
              </h1>
              {isLogin ? (
                <p className="truncate text-sm text-(--app-muted-text)">
                  {displayId}
                </p>
              ) : (
                <p className="text-sm text-(--app-muted-text)">
                  {t("home.publicSubtitle")}
                </p>
              )}
            </div>
          </div>

          {isLogin ? (
            <Button variant="outline" onClick={() => navigate("/profile")}>
              <UserRound className="size-4" />
              {t("home.actions.profile")}
            </Button>
          ) : (
            <Button variant="outline" onClick={() => navigate("/login")}>
              <LogIn className="size-4" />
              {t("home.actions.login")}
            </Button>
          )}
        </div>
      </section>

      <section className="grid gap-4 sm:grid-cols-2">
        <div className="rounded-lg border border-(--app-border) bg-(--app-surface) p-5">
          <div className="flex items-start gap-3">
            <div className="flex size-9 shrink-0 items-center justify-center rounded-md bg-(--app-active-bg) text-(--app-active-text)">
              <ShieldCheck className="size-5" />
            </div>
            <div className="min-w-0 space-y-2">
              <h2 className="text-base font-semibold text-(--app-text)">
                {t(isLogin ? "home.account.title" : "home.publicAccount.title")}
              </h2>
              <p className="text-sm/6 text-(--app-muted-text)">
                {t(
                  isLogin
                    ? "home.account.description"
                    : "home.publicAccount.description"
                )}
              </p>
            </div>
          </div>
        </div>

        <div className="rounded-lg border border-(--app-border) bg-(--app-surface) p-5">
          <div className="flex items-start justify-between gap-4">
            <div className="space-y-2">
              <h2 className="text-base font-semibold text-(--app-text)">
                {t(isLogin ? "home.profile.title" : "home.publicProfile.title")}
              </h2>
              <p className="text-sm/6 text-(--app-muted-text)">
                {t(
                  isLogin
                    ? "home.profile.description"
                    : "home.publicProfile.description"
                )}
              </p>
            </div>
            <Button
              variant="ghost"
              size="icon"
              onClick={() => navigate(isLogin ? "/profile" : "/login")}
            >
              <ArrowRight className="size-4" />
            </Button>
          </div>
        </div>
      </section>
    </div>
  )
}
