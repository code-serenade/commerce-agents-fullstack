import type { ReactNode } from "react"
import { PanelTopClose, X } from "lucide-react"
import { useTranslation } from "react-i18next"
import { Tooltip, TooltipContent, TooltipTrigger } from "../tooltip"
import { cn } from "@workspace/ui-core/lib/utils.js"

export interface WorkspaceTabItem {
  key: string
  label: ReactNode
  href: string
  active?: boolean
  onSelect?: () => void
  onClose?: () => void
}

export interface WorkspaceTabsProps {
  clearLabel?: ReactNode
  items: WorkspaceTabItem[]
  onClear?: () => void
}

export function WorkspaceTabs({
  clearLabel,
  items,
  onClear,
}: WorkspaceTabsProps) {
  const { t } = useTranslation("components")

  if (items.length === 0) {
    return null
  }

  return (
    <div className="bg-[var(--surface)] px-4 py-1 text-[var(--surface-foreground)]">
      <div className="flex min-w-0 items-center gap-3">
        {clearLabel ? (
          <Tooltip>
            <TooltipTrigger>
              <button
                type="button"
                onClick={onClear}
                className="inline-flex size-8 shrink-0 items-center justify-center rounded-md text-muted-foreground transition hover:bg-accent hover:text-accent-foreground"
              >
                <PanelTopClose aria-hidden="true" className="size-4.5" />
                <span className="sr-only">{clearLabel}</span>
              </button>
            </TooltipTrigger>
            <TooltipContent>{clearLabel}</TooltipContent>
          </Tooltip>
        ) : null}

        <div className="flex min-w-0 flex-1 items-center overflow-x-auto">
          <ol className="flex min-w-0 items-center gap-1.5 text-sm text-muted-foreground">
            {items.map((item, index) => {
              const isLast = index === items.length - 1
              const canClose =
                items.length > 1 && typeof item.onClose === "function"

              return (
                <li
                  key={item.key}
                  className="inline-flex shrink-0 items-center gap-1.5"
                >
                  <div className="group inline-flex min-w-0 items-center gap-1">
                    <button
                      type="button"
                      onClick={item.onSelect}
                      className={cn(
                        "inline-flex min-w-0 items-center gap-1.5 rounded-md px-1 py-0.5 text-left outline-hidden transition hover:text-foreground focus-visible:ring-2 focus-visible:ring-[var(--ring)]",
                        item.active && "text-[var(--info)]"
                      )}
                    >
                      <span
                        className={cn(
                          "max-w-40 truncate",
                          item.active ? "font-semibold" : "font-medium"
                        )}
                      >
                        {item.label}
                      </span>
                    </button>

                    {canClose ? (
                      <button
                        type="button"
                        onClick={item.onClose}
                        className="inline-flex size-4.5 items-center justify-center rounded-full text-muted-foreground outline-hidden transition hover:bg-[var(--surface-hover)] hover:text-foreground focus-visible:ring-2 focus-visible:ring-[var(--ring)]"
                        aria-label={t("header.common.closeTab")}
                      >
                        <X
                          aria-hidden="true"
                          className="size-3"
                          strokeWidth={2.4}
                        />
                      </button>
                    ) : null}
                  </div>

                  {!isLast ? (
                    <span
                      aria-hidden="true"
                      className="shrink-0 px-0.5 text-muted-foreground/70"
                    >
                      /
                    </span>
                  ) : null}
                </li>
              )
            })}
          </ol>
        </div>
      </div>
    </div>
  )
}
