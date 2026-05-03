import { Outlet } from "react-router"
import { useTranslation } from "react-i18next"
import { LanguageSwitch, ThemeToggle } from "@workspace/app-kit"

export default function AuthLayout() {
  const { t } = useTranslation()

  return (
    <div className="relative min-h-screen overflow-hidden bg-[linear-gradient(145deg,color-mix(in_oklab,var(--background)_92%,#8fb3ff_8%)_0%,var(--surface)_100%)] px-4 py-6 sm:px-6 sm:py-10">
      <div className="pointer-events-none absolute inset-0 bg-[radial-gradient(circle_at_top_left,color-mix(in_oklab,var(--primary)_22%,transparent),transparent_28%),radial-gradient(circle_at_bottom_right,color-mix(in_oklab,var(--info)_20%,transparent),transparent_24%)]" />

      <div className="relative z-10 mx-auto max-w-6xl">
        <div className="mb-6 flex flex-col gap-3 sm:mb-8 sm:flex-row sm:items-center sm:justify-between">
          <div>
            <p className="text-xs tracking-[0.22em] text-(--app-muted-text)">
              {t("admin.authLayout.eyebrow")}
            </p>
            <h1 className="mt-2 text-3xl font-semibold text-(--app-text)">
              {t("admin.authLayout.title")}
            </h1>
          </div>
          <div className="flex flex-wrap gap-3">
            <LanguageSwitch />
            <ThemeToggle />
          </div>
        </div>

        <Outlet />
      </div>
    </div>
  )
}
