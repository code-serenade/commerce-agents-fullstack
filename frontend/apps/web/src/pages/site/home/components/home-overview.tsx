import {
  ArrowRight,
  BrainCircuit,
  GraduationCap,
  LogIn,
  ShieldCheck,
  UserRound,
} from "lucide-react"
import type { CSSProperties } from "react"
import { useTranslation } from "react-i18next"
import { useNavigate } from "react-router"
import { Button } from "@workspace/ui-components/stable/button"
import { CommerceAgentLogo } from "@/components/brand"
import type { UserInfo } from "@/models/userModel"
import { HomeStudentPath } from "./home-student-path"
import { useAuthDialog } from "@/components/auth/AuthDialogProvider"

const homeGlassPanelStyle: CSSProperties = {
  background:
    "linear-gradient(112deg, color-mix(in oklch, var(--background) 97%, var(--primary) 3%) 0%, color-mix(in oklch, var(--background) 93%, var(--primary) 7%) 54%, color-mix(in oklch, var(--background) 95%, var(--primary-outline, var(--primary)) 5%) 100%)",
  borderColor: "color-mix(in oklch, var(--background) 98%, transparent)",
  boxShadow:
    "0 22px 64px color-mix(in oklch, var(--primary) 9%, transparent), inset 0 1px 0 color-mix(in oklch, var(--background) 98%, transparent)",
}

const homeSoftButtonStyle: CSSProperties = {
  background:
    "linear-gradient(135deg, color-mix(in oklch, var(--background) 96%, var(--primary) 4%) 0%, color-mix(in oklch, var(--background) 88%, var(--primary) 12%) 100%)",
  borderColor: "color-mix(in oklch, var(--background) 96%, transparent)",
  boxShadow:
    "0 10px 24px color-mix(in oklch, var(--primary) 8%, transparent), inset 0 1px 0 color-mix(in oklch, var(--background) 96%, transparent)",
}

const homePrimaryButtonStyle: CSSProperties = {
  background:
    "linear-gradient(135deg, color-mix(in oklch, var(--background) 90%, var(--primary) 10%) 0%, color-mix(in oklch, var(--background) 70%, var(--primary) 30%) 100%)",
  borderColor: "color-mix(in oklch, var(--background) 82%, var(--primary))",
  boxShadow:
    "0 12px 28px color-mix(in oklch, var(--primary) 14%, transparent), inset 0 1px 0 color-mix(in oklch, var(--background) 82%, transparent)",
  color: "var(--primary-outline, var(--primary))",
}

const homeEntryCardStyle: CSSProperties = {
  background:
    "linear-gradient(135deg, color-mix(in oklch, var(--background) 96%, var(--primary) 4%) 0%, color-mix(in oklch, var(--background) 88%, var(--primary) 12%) 100%)",
  borderColor: "color-mix(in oklch, var(--background) 98%, transparent)",
  boxShadow:
    "0 18px 44px color-mix(in oklch, var(--primary) 8%, transparent), inset 0 1px 0 color-mix(in oklch, var(--background) 98%, transparent)",
}

export interface HomeOverviewProps {
  user: UserInfo | null
  displayName: string
  displayId: string
}

export function HomeOverview({
  user,
  displayName,
  displayId,
}: HomeOverviewProps) {
  const { t } = useTranslation()
  const navigate = useNavigate()
  const { requestLogin } = useAuthDialog()
  const isLogin = Boolean(user)
  const navigateToPath = (path: string, requiresLogin = false) => {
    if (requiresLogin && !isLogin) {
      requestLogin({ pathname: path })
      return
    }

    navigate(path)
  }

  return (
    <div className="relative flex min-h-[calc(100vh-73px)] w-full flex-col gap-4 overflow-hidden px-5 pt-3 pb-4 sm:px-8 lg:px-10">
      <div
        aria-hidden="true"
        className="pointer-events-none absolute inset-0 -z-10 bg-[radial-gradient(circle_at_20%_48%,color-mix(in_oklch,var(--primary)_10%,transparent)_0,color-mix(in_oklch,var(--primary)_4%,transparent)_30%,transparent_58%),radial-gradient(circle_at_82%_30%,color-mix(in_oklch,var(--primary-outline,var(--primary))_5%,transparent)_0,color-mix(in_oklch,var(--primary-outline,var(--primary))_2%,transparent)_30%,transparent_58%),linear-gradient(135deg,color-mix(in_oklch,var(--background)_98%,var(--primary)_2%)_0%,var(--background)_46%,color-mix(in_oklch,var(--background)_97%,var(--primary)_3%)_100%)]"
      />

      <section
        className="mx-auto w-full max-w-[760px] rounded-xl border border-[color-mix(in_oklch,var(--background)_98%,transparent)] bg-[linear-gradient(112deg,color-mix(in_oklch,var(--background)_97%,var(--primary)_3%)_0%,color-mix(in_oklch,var(--background)_93%,var(--primary)_7%)_54%,color-mix(in_oklch,var(--background)_95%,var(--primary-outline,var(--primary))_5%)_100%)] px-5 py-4 shadow-[0_22px_64px_color-mix(in_oklch,var(--primary)_9%,transparent),inset_0_1px_0_color-mix(in_oklch,var(--background)_98%,transparent)] backdrop-blur-xl"
        style={homeGlassPanelStyle}
      >
        <div className="flex flex-col gap-4 sm:flex-row sm:items-center sm:justify-between">
          <div className="flex min-w-0 items-center gap-4">
            <CommerceAgentLogo className="size-12 rounded-xl" size="lg" />
            <div className="min-w-0 space-y-1">
              <p className="text-xs font-medium text-(--app-muted-text)">
                {t(isLogin ? "home.welcome" : "home.publicWelcome")}
              </p>
              <h1 className="truncate text-xl font-semibold text-(--app-text)">
                {isLogin
                  ? displayName || t("home.fallbackName")
                  : t("home.publicTitle")}
              </h1>
              {isLogin ? (
                <p className="truncate text-xs text-(--app-muted-text)">
                  {displayId}
                </p>
              ) : (
                <p className="text-xs text-(--app-muted-text)">
                  {t("home.publicSubtitle")}
                </p>
              )}
            </div>
          </div>

          <div className="flex shrink-0 flex-wrap gap-2">
            <Button
              classNameMode="replace"
              className="inline-flex h-8 cursor-pointer items-center gap-1.5 rounded-lg border border-[color-mix(in_oklch,var(--background)_96%,transparent)] bg-[linear-gradient(135deg,color-mix(in_oklch,var(--background)_96%,var(--primary)_4%)_0%,color-mix(in_oklch,var(--background)_88%,var(--primary)_12%)_100%)] px-3 text-xs font-semibold text-(--app-text) shadow-[0_10px_24px_color-mix(in_oklch,var(--primary)_8%,transparent),inset_0_1px_0_color-mix(in_oklch,var(--background)_96%,transparent)] transition hover:border-[color-mix(in_oklch,var(--background)_82%,var(--primary))]"
              size="sm"
              style={homeSoftButtonStyle}
              onClick={() => navigateToPath("/commerce")}
            >
              <BrainCircuit className="size-4" />
              {t("home.actions.assistant")}
            </Button>
            {isLogin ? (
              <Button
                classNameMode="replace"
                className="inline-flex h-8 cursor-pointer items-center gap-1.5 rounded-lg border border-[color-mix(in_oklch,var(--background)_82%,var(--primary))] bg-[linear-gradient(135deg,color-mix(in_oklch,var(--background)_90%,var(--primary)_10%)_0%,color-mix(in_oklch,var(--background)_70%,var(--primary)_30%)_100%)] px-3 text-xs font-semibold text-primary shadow-[0_12px_28px_color-mix(in_oklch,var(--primary)_14%,transparent),inset_0_1px_0_color-mix(in_oklch,var(--background)_82%,transparent)] transition hover:border-[color-mix(in_oklch,var(--background)_70%,var(--primary))]"
                variant="outline"
                size="sm"
                style={homePrimaryButtonStyle}
                onClick={() => navigate("/profile")}
              >
                <UserRound className="size-4" />
                {t("home.actions.profile")}
              </Button>
            ) : (
              <Button
                classNameMode="replace"
                className="inline-flex h-8 cursor-pointer items-center gap-1.5 rounded-lg border border-[color-mix(in_oklch,var(--background)_82%,var(--primary))] bg-[linear-gradient(135deg,color-mix(in_oklch,var(--background)_90%,var(--primary)_10%)_0%,color-mix(in_oklch,var(--background)_70%,var(--primary)_30%)_100%)] px-3 text-xs font-semibold text-primary shadow-[0_12px_28px_color-mix(in_oklch,var(--primary)_14%,transparent),inset_0_1px_0_color-mix(in_oklch,var(--background)_82%,transparent)] transition hover:border-[color-mix(in_oklch,var(--background)_70%,var(--primary))]"
                variant="outline"
                size="sm"
                style={homePrimaryButtonStyle}
                onClick={() => requestLogin()}
              >
                <LogIn className="size-4" />
                {t("home.actions.login")}
              </Button>
            )}
          </div>
        </div>
      </section>

      <HomeStudentPath t={t} navigateToPath={navigateToPath} />

      <section className="mx-auto grid w-full max-w-[760px] gap-4 sm:grid-cols-2">
        <button
          type="button"
          className="group cursor-pointer rounded-xl border border-[color-mix(in_oklch,var(--background)_98%,transparent)] bg-[linear-gradient(135deg,color-mix(in_oklch,var(--background)_96%,var(--primary)_4%)_0%,color-mix(in_oklch,var(--background)_88%,var(--primary)_12%)_100%)] p-4 text-left shadow-[0_18px_44px_color-mix(in_oklch,var(--primary)_8%,transparent),inset_0_1px_0_color-mix(in_oklch,var(--background)_98%,transparent)] backdrop-blur-xl transition hover:border-[color-mix(in_oklch,var(--background)_82%,var(--primary))]"
          style={homeEntryCardStyle}
          onClick={() =>
            isLogin
              ? navigateToPath("/workspaces")
              : requestLogin({ pathname: "/workspaces" })
          }
        >
          <div className="flex items-start gap-3">
            <div className="flex size-9 shrink-0 items-center justify-center rounded-lg bg-(--app-active-bg) text-(--app-active-text)">
              <ShieldCheck className="size-5" />
            </div>
            <div className="min-w-0 space-y-1">
              <h2 className="text-sm font-semibold text-(--app-text)">
                {t(isLogin ? "home.account.title" : "home.publicAccount.title")}
              </h2>
              <p className="text-xs/5 text-(--app-muted-text)">
                {t(
                  isLogin
                    ? "home.account.description"
                    : "home.publicAccount.description"
                )}
              </p>
            </div>
            <ArrowRight className="mt-1 size-4 shrink-0 text-(--app-muted-text) transition group-hover:translate-x-0.5" />
          </div>
        </button>

        <button
          type="button"
          className="group cursor-pointer rounded-xl border border-[color-mix(in_oklch,var(--background)_98%,transparent)] bg-[linear-gradient(135deg,color-mix(in_oklch,var(--background)_96%,var(--primary)_4%)_0%,color-mix(in_oklch,var(--background)_88%,var(--primary)_12%)_100%)] p-4 text-left shadow-[0_18px_44px_color-mix(in_oklch,var(--primary)_8%,transparent),inset_0_1px_0_color-mix(in_oklch,var(--background)_98%,transparent)] backdrop-blur-xl transition hover:border-[color-mix(in_oklch,var(--background)_82%,var(--primary))]"
          style={homeEntryCardStyle}
          onClick={() =>
            isLogin
              ? navigate("/profile")
              : requestLogin({ pathname: "/profile" })
          }
        >
          <div className="flex items-start gap-3">
            <div className="flex size-9 shrink-0 items-center justify-center rounded-lg bg-(--app-active-bg) text-(--app-active-text)">
              <UserRound className="size-5" />
            </div>
            <div className="min-w-0 space-y-1">
              <h2 className="text-sm font-semibold text-(--app-text)">
                {t(isLogin ? "home.profile.title" : "home.publicProfile.title")}
              </h2>
              <p className="text-xs/5 text-(--app-muted-text)">
                {t(
                  isLogin
                    ? "home.profile.description"
                    : "home.publicProfile.description"
                )}
              </p>
            </div>
            <ArrowRight className="mt-1 size-4 shrink-0 text-(--app-muted-text) transition group-hover:translate-x-0.5" />
          </div>
        </button>
      </section>

      <section
        className="mx-auto w-full max-w-[760px] cursor-pointer rounded-xl border border-[color-mix(in_oklch,var(--background)_98%,transparent)] bg-[linear-gradient(135deg,color-mix(in_oklch,var(--background)_96%,var(--primary)_4%)_0%,color-mix(in_oklch,var(--background)_88%,var(--primary)_12%)_100%)] p-4 text-left shadow-[0_18px_44px_color-mix(in_oklch,var(--primary)_8%,transparent),inset_0_1px_0_color-mix(in_oklch,var(--background)_98%,transparent)] backdrop-blur-xl transition hover:border-[color-mix(in_oklch,var(--background)_82%,var(--primary))]"
        style={homeEntryCardStyle}
        onClick={() =>
          isLogin
            ? navigateToPath("/learning-progress")
            : requestLogin({ pathname: "/learning-progress" })
        }
      >
        <div className="flex items-start gap-3">
          <div className="flex size-9 shrink-0 items-center justify-center rounded-lg bg-(--app-active-bg) text-(--app-active-text)">
            <GraduationCap className="size-5" />
          </div>
          <div className="min-w-0 flex-1 space-y-1">
            <h2 className="text-sm font-semibold text-(--app-text)">
              {t("home.learning.title")}
            </h2>
            <p className="text-xs/5 text-(--app-muted-text)">
              {t("home.learning.description")}
            </p>
          </div>
          <ArrowRight className="mt-1 size-4 shrink-0 text-(--app-muted-text) transition group-hover:translate-x-0.5" />
        </div>
      </section>
    </div>
  )
}
