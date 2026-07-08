import { Outlet, useLocation } from "react-router"
import { LanguageSwitch, ThemeToggle } from "@workspace/app-kit"
import { ProductAnalyticsTracker } from "@/analytics/ProductAnalyticsTracker"
import { CircleHelp } from "lucide-react"
import { useTranslation } from "react-i18next"

const authToolClassName =
  "size-9 rounded-xl border border-[rgba(209,220,240,0.96)] bg-white/80 text-[#475467] shadow-[0_12px_28px_rgba(38,76,160,0.08)] backdrop-blur-xl hover:bg-white/90 dark:border-[rgba(159,179,217,0.22)] dark:bg-[rgba(11,18,38,0.58)] dark:text-[#d6e2ff] dark:shadow-[0_16px_34px_rgba(0,0,0,0.22)] dark:hover:bg-[rgba(16,25,50,0.78)]"

export default function AuthLayout() {
  const location = useLocation()
  const { t } = useTranslation()
  const isLoginPage = location.pathname === "/login"

  if (isLoginPage) {
    return (
      <div className="login-auth-bg relative grid min-h-screen place-items-center overflow-hidden bg-[radial-gradient(circle_at_60%_12%,rgba(120,70,255,0.14),transparent_30%),radial-gradient(circle_at_34%_50%,rgba(67,151,255,0.16),transparent_34%),linear-gradient(115deg,#fbfdff_0%,#f7fbff_38%,#eef5ff_72%,#f7f4ff_100%)] px-6 py-14 sm:px-6 dark:bg-[radial-gradient(circle_at_59%_16%,rgba(139,61,255,0.3),transparent_28%),radial-gradient(circle_at_34%_54%,rgba(49,93,255,0.24),transparent_34%),radial-gradient(circle_at_84%_80%,rgba(0,205,255,0.08),transparent_30%),linear-gradient(125deg,#060a16_0%,#0b1224_42%,#111735_72%,#120c25_100%)]">
        <div className="login-auth-flow pointer-events-none absolute -inset-[18%] bg-[radial-gradient(circle_at_20%_32%,rgba(120,70,255,0.16),transparent_24%),radial-gradient(circle_at_78%_36%,rgba(67,151,255,0.14),transparent_26%),linear-gradient(115deg,transparent_12%,rgba(255,255,255,0.26)_42%,transparent_62%)] mix-blend-soft-light dark:bg-[radial-gradient(circle_at_20%_32%,rgba(139,61,255,0.22),transparent_24%),radial-gradient(circle_at_78%_36%,rgba(49,93,255,0.18),transparent_26%),linear-gradient(115deg,transparent_12%,rgba(118,162,255,0.12)_42%,transparent_62%)]" />
        <div className="login-auth-grid pointer-events-none absolute inset-0 hidden bg-[linear-gradient(rgba(126,154,255,0.11)_1px,transparent_1px),linear-gradient(90deg,rgba(166,119,255,0.105)_1px,transparent_1px)] [mask-image:radial-gradient(circle_at_50%_40%,#000_0%,rgba(0,0,0,0.72)_34%,rgba(0,0,0,0.28)_58%,transparent_78%)] bg-[length:56px_56px] dark:block" />
        <div className="login-auth-vignette pointer-events-none absolute inset-0 hidden bg-[radial-gradient(circle_at_50%_38%,transparent_0%,rgba(6,10,22,0.1)_34%,rgba(6,10,22,0.54)_78%),linear-gradient(90deg,rgba(6,10,22,0.36),transparent_34%,transparent_66%,rgba(6,10,22,0.42))] dark:block" />

        <div className="absolute top-6 right-5 z-30 flex gap-2.5 sm:top-10 sm:right-12">
          <ThemeToggle className={authToolClassName} />
          <LanguageSwitch className={authToolClassName} />
          <details className="group relative">
            <summary
              className={`${authToolClassName} inline-flex cursor-pointer list-none items-center justify-center [&::-webkit-details-marker]:hidden`}
              aria-label={t("login.help.label")}
              title={t("login.help.label")}
            >
              <CircleHelp
                aria-hidden="true"
                className="size-5"
                strokeWidth={2.2}
              />
            </summary>
            <div className="invisible absolute top-12 right-0 z-40 w-[min(330px,calc(100vw-40px))] rounded-2xl border border-[rgba(209,220,240,0.96)] bg-white/90 p-[18px] text-left opacity-0 shadow-[0_24px_60px_rgba(38,76,160,0.16)] backdrop-blur-xl transition group-open:visible group-open:opacity-100 sm:w-[360px] dark:border-[rgba(159,179,217,0.24)] dark:bg-[rgba(11,18,38,0.86)] dark:shadow-[0_26px_70px_rgba(0,0,0,0.34)]">
              <ol className="m-0 grid gap-3 pl-5">
                {[
                  t("login.help.steps.0"),
                  t("login.help.steps.1"),
                  t("login.help.steps.2"),
                ].map((step) => (
                  <li
                    key={step}
                    className="text-[13px] leading-[1.75] text-[#344054] dark:text-[#d8e4ff]"
                  >
                    {step}
                  </li>
                ))}
              </ol>
            </div>
          </details>
        </div>

        <ProductAnalyticsTracker />
        <Outlet />
      </div>
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
