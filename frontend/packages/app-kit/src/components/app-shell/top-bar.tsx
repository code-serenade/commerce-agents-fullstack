import type { ReactNode } from "react"
import { SidebarTrigger } from "@workspace/ui-components"
import { useTranslation } from "react-i18next"

export interface TopBarProps {
  title?: ReactNode
  meta?: ReactNode
  leading?: ReactNode
  trailing?: ReactNode[]
  showSidebarTrigger?: boolean
}

export function TopBar({
  title,
  meta,
  leading,
  trailing = [],
  showSidebarTrigger = true,
}: TopBarProps) {
  const { t } = useTranslation("components")

  return (
    <header className="sticky top-0 z-30 w-full min-w-0 shrink-0 bg-[var(--surface)] text-[var(--surface-foreground)]">
      <div className="flex min-w-0 items-center justify-between gap-4 px-4 py-1">
        <div className="flex min-w-0 items-center gap-3">
          {showSidebarTrigger ? (
            <SidebarTrigger ariaLabel={t("header.common.toggleSidebar")} />
          ) : null}
          {leading ? <div className="shrink-0">{leading}</div> : null}
          {title || meta ? (
            <div className="min-w-0">
              {title ? (
                <div className="truncate text-lg font-semibold text-(--app-text)">
                  {title}
                </div>
              ) : null}
              {meta ? (
                <div className="truncate text-sm text-(--app-muted-text)">
                  {meta}
                </div>
              ) : null}
            </div>
          ) : null}
        </div>

        <div className="flex min-w-0 flex-nowrap items-center justify-end gap-3">
          {trailing.map((item, index) => (
            <div key={index} className="shrink-0">
              {item}
            </div>
          ))}
        </div>
      </div>
    </header>
  )
}
