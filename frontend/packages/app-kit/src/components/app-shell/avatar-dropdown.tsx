import type { ReactNode } from "react"
import { ChevronsUpDown, LogOut } from "lucide-react"
import { useTranslation } from "react-i18next"
import {
  DropdownMenu,
  DropdownMenuContent,
  DropdownMenuItem,
  DropdownMenuTrigger,
} from "@workspace/ui-core/components/dropdown-menu"
import { Separator } from "@workspace/ui-components/stable/separator"

export interface AvatarDropdownAction {
  label?: ReactNode
  onSelect?: () => void
  href?: string
  disabled?: boolean
}

export interface AvatarDropdownItemProps {
  icon?: ReactNode
  label: ReactNode
  onSelect?: () => void
  href?: string
  disabled?: boolean
}

export interface AvatarDropdownProps {
  avatarSrc?: string
  avatarAlt: string
  avatarFallback: ReactNode
  displayName: ReactNode
  displayId: ReactNode
  children?: ReactNode
  logout?: AvatarDropdownAction
  triggerVariant?: "icon" | "sidebar"
}

function handleAction(action?: { href?: string; onSelect?: () => void }) {
  if (!action) {
    return undefined
  }

  return (event: Event) => {
    if (action.href) {
      event.preventDefault()
      window.location.assign(action.href)
      return
    }

    action.onSelect?.()
  }
}

function itemClassName() {
  return "flex min-h-8 cursor-default items-center gap-2 rounded-md px-2 py-1.5 text-sm leading-none font-medium text-foreground outline-hidden transition data-[disabled]:pointer-events-none data-[disabled]:opacity-50 hover:bg-[var(--surface-hover)] hover:text-foreground focus:bg-[var(--surface-hover)] focus:text-foreground"
}

export function AvatarDropdownItem({
  icon,
  label,
  onSelect,
  href,
  disabled = false,
}: AvatarDropdownItemProps) {
  return (
    <DropdownMenuItem
      mode="primitive"
      className={itemClassName()}
      onSelect={handleAction({ href, onSelect })}
      disabled={disabled}
    >
      {icon ? (
        <span className="shrink-0 text-muted-foreground [&_svg]:size-5">
          {icon}
        </span>
      ) : null}
      <span>{label}</span>
    </DropdownMenuItem>
  )
}

export function AvatarDropdown({
  avatarSrc,
  avatarAlt,
  avatarFallback,
  displayName,
  displayId,
  children,
  logout,
  triggerVariant = "icon",
}: AvatarDropdownProps) {
  const { t } = useTranslation("components")
  const openAccountMenuLabel = t("header.common.openAccountMenu")
  const logoutLabel = t("header.me.logout")

  return (
    <DropdownMenu>
      <DropdownMenuTrigger asChild>
        {triggerVariant === "sidebar" ? (
          <button
            type="button"
            className="flex w-full items-center gap-2.5 overflow-hidden rounded-xl bg-[var(--surface)] px-2.5 py-2 text-left outline-hidden transition hover:bg-[var(--surface-hover)] focus-visible:ring-2 focus-visible:ring-(--app-primary) group-data-[collapsible=icon]:w-9 group-data-[collapsible=icon]:gap-0 group-data-[collapsible=icon]:rounded-full group-data-[collapsible=icon]:px-0 group-data-[collapsible=icon]:py-0"
            aria-label={openAccountMenuLabel}
          >
            <span className="flex h-9 w-9 shrink-0 items-center justify-center overflow-hidden rounded-full bg-[#0f1724]">
              {avatarSrc ? (
                <img
                  src={avatarSrc}
                  alt={avatarAlt}
                  className="block h-9 w-9 rounded-full object-cover"
                />
              ) : null}
              {!avatarSrc ? (
                <span className="flex h-9 w-9 items-center justify-center rounded-full bg-[#1c2431] text-[13px] font-semibold text-white">
                  {avatarFallback}
                </span>
              ) : null}
            </span>
            <span className="min-w-0 flex-1 group-data-[collapsible=icon]:hidden">
              <span className="block truncate text-sm font-semibold text-foreground">
                {displayName}
              </span>
              <span className="block min-w-0 text-xs text-muted-foreground">
                {displayId}
              </span>
            </span>
            <ChevronsUpDown className="size-4 shrink-0 text-muted-foreground group-data-[collapsible=icon]:hidden" />
          </button>
        ) : (
          <button
            type="button"
            className="relative z-10 flex h-7 w-7 shrink-0 items-center justify-center overflow-hidden rounded-full border border-white/35 bg-[#121821] p-0.5 outline-hidden transition hover:border-white/55 focus-visible:ring-2 focus-visible:ring-(--app-primary)"
            aria-label={openAccountMenuLabel}
          >
            <span className="flex h-6 w-6 items-center justify-center overflow-hidden rounded-full bg-[#0f1724]">
              {avatarSrc ? (
                <img
                  src={avatarSrc}
                  alt={avatarAlt}
                  className="block h-6 w-6 rounded-full object-cover"
                />
              ) : null}
              {!avatarSrc ? (
                <span className="flex h-6 w-6 items-center justify-center rounded-full bg-[#1c2431] text-[10px] font-semibold text-white">
                  {avatarFallback}
                </span>
              ) : null}
            </span>
          </button>
        )}
      </DropdownMenuTrigger>

      <DropdownMenuContent
        mode="primitive"
        align="end"
        sideOffset={10}
        className="z-[2000] w-56 overflow-hidden rounded-[10px] border border-[var(--border)] bg-[var(--surface)] p-0 text-[var(--surface-foreground)] opacity-100 shadow-[0_24px_80px_rgba(0,0,0,0.55)] outline-hidden"
      >
        <div className="flex items-center gap-2 px-2 py-2">
          <span className="flex h-8 w-8 shrink-0 items-center justify-center overflow-hidden rounded-full bg-[#0f1724]">
            {avatarSrc ? (
              <img
                src={avatarSrc}
                alt={avatarAlt}
                className="block h-8 w-8 rounded-full object-cover"
              />
            ) : null}
            {!avatarSrc ? (
              <span className="flex h-8 w-8 items-center justify-center rounded-full bg-[#1c2431] text-[11px] font-semibold text-white">
                {avatarFallback}
              </span>
            ) : null}
          </span>
          <div className="min-w-0">
            <div className="truncate text-[13px] font-semibold text-foreground">
              {displayName}
            </div>
            <div className="min-w-0 text-[11px] text-muted-foreground">
              {displayId}
            </div>
          </div>
        </div>

        <Separator />

        {children ? (
          <>
            <div className="p-1">{children}</div>
            <Separator />
          </>
        ) : null}

        {logout ? (
          <div className="p-1">
            <DropdownMenuItem
              mode="primitive"
              className={itemClassName()}
              onSelect={handleAction(logout)}
              disabled={logout.disabled}
            >
              <LogOut className="size-5 shrink-0 text-muted-foreground" />
              <span>{logout.label ?? logoutLabel}</span>
            </DropdownMenuItem>
          </div>
        ) : null}
      </DropdownMenuContent>
    </DropdownMenu>
  )
}
