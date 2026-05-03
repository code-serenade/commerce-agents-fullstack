import type { ReactNode } from "react"
import { Bell, CheckCheck } from "lucide-react"
import {
  DropdownMenu,
  DropdownMenuContent,
  DropdownMenuTrigger,
} from "@workspace/ui-core/components/dropdown-menu"
import { cn } from "@workspace/ui-core/lib/utils.js"

export interface NotificationDropdownAction {
  label: string
  icon?: ReactNode
  onSelect?: () => void
}

export interface NotificationDropdownItem {
  id: string
  title: ReactNode
  description?: ReactNode
  timeLabel: ReactNode
  unread?: boolean
  leading: ReactNode
  action?: NotificationDropdownAction
  onSelect?: () => void
}

export interface NotificationDropdownProps {
  label: string
  panelTitle: ReactNode
  markAllLabel?: string
  items: NotificationDropdownItem[]
  emptyLabel?: ReactNode
  onMarkAll?: () => void
}

export function NotificationDropdown({
  label,
  panelTitle,
  markAllLabel,
  items,
  emptyLabel,
  onMarkAll,
}: NotificationDropdownProps) {
  const unreadCount = items.filter((item) => item.unread).length

  return (
    <DropdownMenu>
      <DropdownMenuTrigger
        mode="primitive"
        className="relative inline-flex size-9 items-center justify-center rounded-full border border-transparent bg-transparent text-muted-foreground shadow-none transition-colors hover:bg-[var(--surface-hover)] hover:text-foreground"
        aria-label={label}
      >
        <Bell aria-hidden="true" className="size-5" strokeWidth={2.2} />
        {unreadCount > 0 ? (
          <span className="absolute top-1.5 right-1.5 size-2.5 rounded-full bg-[var(--destructive)] ring-2 ring-background" />
        ) : null}
        <span className="sr-only">{label}</span>
      </DropdownMenuTrigger>

      <DropdownMenuContent
        mode="primitive"
        align="end"
        sideOffset={8}
        className="z-[2000] w-[22rem] overflow-hidden rounded-[18px] border border-[var(--border)] bg-[var(--surface)] p-0 text-[var(--surface-foreground)] shadow-[0_24px_80px_rgba(0,0,0,0.45)] outline-hidden"
      >
        <div className="flex items-center justify-between border-b border-[var(--border)] px-4 py-4">
          <div className="text-xl font-semibold text-foreground">
            {panelTitle}
          </div>
          {markAllLabel ? (
            <button
              type="button"
              onClick={onMarkAll}
              className="inline-flex size-9 items-center justify-center rounded-full text-muted-foreground outline-hidden transition hover:bg-[var(--surface-hover)] hover:text-foreground focus-visible:ring-2 focus-visible:ring-[var(--ring)]"
              aria-label={markAllLabel}
            >
              <CheckCheck
                aria-hidden="true"
                className="size-5"
                strokeWidth={2.1}
              />
            </button>
          ) : null}
        </div>

        {items.length === 0 ? (
          <div className="px-4 py-8 text-sm text-muted-foreground">
            {emptyLabel}
          </div>
        ) : (
          <div className="max-h-[26rem] overflow-y-auto">
            {items.map((item, index) => (
              <div
                key={item.id}
                className={cn(
                  "relative px-4 py-4",
                  index > 0 && "border-t border-[var(--border)]"
                )}
              >
                <button
                  type="button"
                  onClick={item.onSelect}
                  className="flex w-full items-start gap-3 rounded-2xl text-left outline-hidden transition hover:bg-[var(--surface-hover)] focus-visible:ring-2 focus-visible:ring-[var(--ring)]"
                >
                  <span className="mt-0.5 shrink-0">{item.leading}</span>
                  <span className="min-w-0 flex-1 pr-8">
                    <span className="block truncate text-[15px] font-semibold text-foreground">
                      {item.title}
                    </span>
                    {item.description ? (
                      <span className="mt-1 line-clamp-2 block text-sm leading-5 text-muted-foreground">
                        {item.description}
                      </span>
                    ) : null}
                    <span className="mt-2 block text-sm text-muted-foreground">
                      {item.timeLabel}
                    </span>
                  </span>
                </button>

                {item.unread ? (
                  <span className="absolute top-4 right-4 size-2.5 rounded-full bg-[var(--destructive)]" />
                ) : null}

                {item.action ? (
                  <button
                    type="button"
                    onClick={(event) => {
                      event.stopPropagation()
                      item.action?.onSelect?.()
                    }}
                    className="absolute right-3 bottom-4 inline-flex size-9 items-center justify-center rounded-full text-muted-foreground outline-hidden transition hover:bg-[var(--surface-hover)] hover:text-foreground focus-visible:ring-2 focus-visible:ring-[var(--ring)]"
                    aria-label={item.action.label}
                  >
                    {item.action.icon}
                  </button>
                ) : null}
              </div>
            ))}
          </div>
        )}
      </DropdownMenuContent>
    </DropdownMenu>
  )
}
