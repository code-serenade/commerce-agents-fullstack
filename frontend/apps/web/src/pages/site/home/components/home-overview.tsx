import {
  ArrowRight,
  BrainCircuit,
  ClipboardCheck,
  FileText,
  LogIn,
  Megaphone,
  PackageSearch,
  ShieldCheck,
  UserRound,
  type LucideIcon,
} from "lucide-react"
import { useTranslation } from "react-i18next"
import { useNavigate } from "react-router"
import { Avatar } from "@workspace/ui-components/stable/avatar"
import { Button } from "@workspace/ui-components/stable/button"
import type { UserInfo } from "@/models/userModel"

interface StudentPathItem {
  icon: LucideIcon
  titleKey: string
  descriptionKey: string
  actionKey: string
  path: string
  requiresLogin: boolean
}

const studentPathItems: StudentPathItem[] = [
  {
    icon: BrainCircuit,
    titleKey: "home.studentPath.steps.assistant.title",
    descriptionKey: "home.studentPath.steps.assistant.description",
    actionKey: "home.studentPath.steps.assistant.action",
    path: "/commerce",
    requiresLogin: false,
  },
  {
    icon: PackageSearch,
    titleKey: "home.studentPath.steps.products.title",
    descriptionKey: "home.studentPath.steps.products.description",
    actionKey: "home.studentPath.steps.products.action",
    path: "/products",
    requiresLogin: true,
  },
  {
    icon: Megaphone,
    titleKey: "home.studentPath.steps.campaigns.title",
    descriptionKey: "home.studentPath.steps.campaigns.description",
    actionKey: "home.studentPath.steps.campaigns.action",
    path: "/campaigns",
    requiresLogin: true,
  },
  {
    icon: ClipboardCheck,
    titleKey: "home.studentPath.steps.tasks.title",
    descriptionKey: "home.studentPath.steps.tasks.description",
    actionKey: "home.studentPath.steps.tasks.action",
    path: "/workspaces",
    requiresLogin: true,
  },
  {
    icon: FileText,
    titleKey: "home.studentPath.steps.reports.title",
    descriptionKey: "home.studentPath.steps.reports.description",
    actionKey: "home.studentPath.steps.reports.action",
    path: "/agent-output",
    requiresLogin: true,
  },
]

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
  const navigateToPath = (path: string, requiresLogin = false) => {
    if (requiresLogin && !isLogin) {
      navigate("/login", { state: { from: { pathname: path } } })
      return
    }

    navigate(path)
  }

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

          <div className="flex shrink-0 flex-col gap-2 sm:flex-row">
            <Button onClick={() => navigateToPath("/commerce")}>
              <BrainCircuit className="size-4" />
              {t("home.actions.assistant")}
            </Button>
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
        </div>
      </section>

      <section className="space-y-4">
        <div className="max-w-3xl space-y-2">
          <p className="text-sm font-medium text-(--app-muted-text)">
            {t("home.studentPath.eyebrow")}
          </p>
          <h2 className="text-xl font-semibold text-(--app-text)">
            {t("home.studentPath.title")}
          </h2>
          <p className="text-sm/6 text-(--app-muted-text)">
            {t("home.studentPath.description")}
          </p>
        </div>

        <div className="grid gap-3 sm:grid-cols-2 lg:grid-cols-5">
          {studentPathItems.map((item, index) => {
            const StepIcon = item.icon
            return (
              <button
                key={item.path}
                type="button"
                className="group flex min-h-44 flex-col justify-between rounded-lg border border-(--app-border) bg-(--app-surface) p-4 text-left transition hover:border-ring/60 hover:bg-(--app-panel)"
                onClick={() => navigateToPath(item.path, item.requiresLogin)}
              >
                <span className="space-y-3">
                  <span className="flex items-center justify-between gap-2">
                    <span className="flex size-9 items-center justify-center rounded-md bg-(--app-active-bg) text-(--app-active-text)">
                      <StepIcon className="size-4" />
                    </span>
                    <span className="text-xs font-semibold text-(--app-muted-text)">
                      {String(index + 1).padStart(2, "0")}
                    </span>
                  </span>
                  <span className="block text-sm font-semibold text-(--app-text)">
                    {t(item.titleKey)}
                  </span>
                  <span className="block text-xs/5 text-(--app-muted-text)">
                    {t(item.descriptionKey)}
                  </span>
                </span>
                <span className="mt-4 flex items-center gap-1 text-xs font-medium text-(--app-text)">
                  {t(item.actionKey)}
                  <ArrowRight className="size-3.5 transition group-hover:translate-x-0.5" />
                </span>
              </button>
            )
          })}
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
