import { Outlet, useLocation } from "react-router"
import { LanguageSwitch, ThemeToggle } from "@workspace/app-kit"
import { ProductAnalyticsTracker } from "@/analytics/ProductAnalyticsTracker"
import { useTranslation } from "react-i18next"
import { MinimalAuthShell } from "@/layouts/auth/MinimalAuthShell"
import { MinimalAuthTools } from "@/layouts/auth/MinimalAuthTools"

export default function AuthLayout() {
  const location = useLocation()
  const { t } = useTranslation()
  const isMinimalAuthPage =
    location.pathname === "/login" || location.pathname === "/register"
  const helpLabel = t("login.help.label")
  const helpSteps = [
    t("login.help.steps.0"),
    t("login.help.steps.1"),
    t("login.help.steps.2"),
  ]

  if (isMinimalAuthPage) {
    return (
      <MinimalAuthShell
        tools={<MinimalAuthTools helpLabel={helpLabel} helpSteps={helpSteps} />}
      >
        <Outlet />
      </MinimalAuthShell>
    )
  }

  return (
    <div className="relative min-h-screen overflow-hidden bg-[linear-gradient(160deg,var(--app-bg)_0%,var(--app-surface)_100%)] px-4 py-6 sm:py-10">
      <div className="pointer-events-none absolute top-16 -left-24 size-72 rounded-full bg-[color-mix(in_oklab,var(--app-active-bg)_30%,transparent)] blur-3xl" />
      <div className="pointer-events-none absolute -right-24 bottom-10 size-72 rounded-full bg-[color-mix(in_oklab,var(--app-hover)_80%,transparent)] blur-3xl" />

      <div className="relative z-30 mx-auto flex w-full max-w-5xl justify-end gap-3 pb-5 sm:pb-6">
        <ThemeToggle />
        <LanguageSwitch />
      </div>

      <ProductAnalyticsTracker />
      <Outlet />
    </div>
  )
}
