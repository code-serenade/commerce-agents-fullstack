import { useEffect, useState, type ReactNode } from "react"

export interface MobileBottomNavSubItem {
  key?: string
  label: ReactNode
  href: string
  active?: boolean
  onSelect?: () => void
}

export interface MobileBottomNavItem {
  key?: string
  label: ReactNode
  href?: string
  active?: boolean
  icon?: ReactNode
  onSelect?: () => void
  subItems?: MobileBottomNavSubItem[]
}

export interface MobileBottomNavProps {
  label?: string
  items: MobileBottomNavItem[]
}

export function MobileBottomNav({
  label = "Primary navigation",
  items,
}: MobileBottomNavProps) {
  const activeGroupKey = items.find((item) =>
    item.subItems?.some((subItem) => subItem.active)
  )?.key
  const [openKey, setOpenKey] = useState<string | null>(null)

  useEffect(() => {
    setOpenKey(null)
  }, [activeGroupKey])

  if (!items.length) {
    return null
  }

  const openItem = items.find((item) => item.key === openKey)

  return (
    <>
      {openItem?.subItems?.length ? (
        <div className="fixed inset-x-3 bottom-[calc(4.5rem+env(safe-area-inset-bottom))] z-40 overflow-hidden rounded-xl border border-(--app-border) bg-[var(--surface)] p-2 text-[var(--surface-foreground)] shadow-[0_18px_64px_rgba(0,0,0,0.32)] md:hidden">
          <div className="grid gap-1">
            {openItem.subItems.map((subItem) => (
              <a
                key={subItem.key ?? subItem.href}
                href={subItem.href}
                className={[
                  "flex min-h-10 items-center rounded-lg px-3 text-sm font-medium outline-hidden transition",
                  subItem.active
                    ? "bg-sidebar-accent text-sidebar-primary"
                    : "text-sidebar-foreground hover:bg-sidebar-accent hover:text-sidebar-accent-foreground focus-visible:ring-2 focus-visible:ring-sidebar-ring",
                ].join(" ")}
                aria-current={subItem.active ? "page" : undefined}
                onClick={(event) => {
                  if (subItem.onSelect) {
                    event.preventDefault()
                    subItem.onSelect()
                  }

                  setOpenKey(null)
                }}
              >
                <span className="truncate">{subItem.label}</span>
              </a>
            ))}
          </div>
        </div>
      ) : null}

      <nav
        aria-label={label}
        className="fixed inset-x-0 bottom-0 z-40 border-t border-(--app-border) bg-[var(--surface)] px-2 pb-[max(env(safe-area-inset-bottom),0.5rem)] pt-2 text-[var(--surface-foreground)] shadow-[0_-18px_48px_rgba(0,0,0,0.2)] md:hidden"
      >
        <div className="mx-auto grid max-w-lg grid-flow-col auto-cols-fr gap-1">
          {items.map((item) => (
            <MobileBottomNavButton
              key={item.key ?? item.href}
              item={item}
              open={openKey === item.key}
              onOpenChange={setOpenKey}
            />
          ))}
        </div>
      </nav>
    </>
  )
}

function MobileBottomNavButton({
  item,
  open,
  onOpenChange,
}: {
  item: MobileBottomNavItem
  open: boolean
  onOpenChange: (key: string | null) => void
}) {
  const hasSubItems = Boolean(item.subItems?.length)
  const active = item.active || item.subItems?.some((subItem) => subItem.active)
  const content = (
    <>
      <span className="flex size-5 shrink-0 items-center justify-center [&_svg]:size-5">
        {item.icon}
      </span>
      <span className="w-full truncate text-center text-[11px] leading-4 font-medium">
        {item.label}
      </span>
    </>
  )
  const className = [
    "flex h-12 min-w-0 flex-col items-center justify-center gap-1 rounded-lg px-2 outline-hidden transition",
    active || open
      ? "bg-sidebar-accent text-sidebar-primary"
      : "text-sidebar-foreground hover:bg-sidebar-accent hover:text-sidebar-accent-foreground focus-visible:ring-2 focus-visible:ring-sidebar-ring",
  ].join(" ")

  if (hasSubItems) {
    return (
      <button
        type="button"
        className={className}
        aria-current={active ? "page" : undefined}
        aria-expanded={open}
        onClick={() => onOpenChange(open ? null : (item.key ?? null))}
      >
        {content}
      </button>
    )
  }

  if (item.href) {
    return (
      <a
        href={item.href}
        className={className}
        aria-current={item.active ? "page" : undefined}
        onClick={(event) => {
          if (!item.onSelect) {
            return
          }

          event.preventDefault()
          item.onSelect()
        }}
      >
        {content}
      </a>
    )
  }

  return (
    <button
      type="button"
      className={className}
      aria-current={item.active ? "page" : undefined}
      onClick={item.onSelect}
    >
      {content}
    </button>
  )
}
