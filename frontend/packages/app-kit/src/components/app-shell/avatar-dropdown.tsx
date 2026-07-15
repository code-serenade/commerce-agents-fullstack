import type { ReactNode } from "react"
import { LogOut, UserRound } from "lucide-react"
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
  onTriggerSelect?: () => void
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
  return "flex min-h-8 cursor-pointer items-center gap-2 rounded-md px-2 py-1.5 text-sm leading-none font-medium text-foreground outline-hidden transition data-[disabled]:pointer-events-none data-[disabled]:opacity-50 hover:bg-(--shell-sidebar-item-hover-background) hover:text-foreground focus:bg-(--shell-sidebar-item-hover-background) focus:text-foreground"
}

function AvatarPlaceholder({ size }: { size: "sm" | "md" | "lg" }) {
  const sizeClassName =
    size === "sm" ? "h-6 w-6" : size === "md" ? "h-8 w-8" : "h-10 w-10"
  const iconClassName =
    size === "sm" ? "size-3.5" : size === "md" ? "size-4" : "size-5"

  return (
    <span
      className={`flex ${sizeClassName} items-center justify-center rounded-full bg-slate-200 text-slate-600 ring-1 ring-slate-300/80 dark:bg-slate-700 dark:text-slate-300 dark:ring-slate-600/80`}
    >
      <UserRound aria-hidden="true" className={iconClassName} />
    </span>
  )
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
  displayName,
  displayId,
  children,
  logout,
  onTriggerSelect,
  triggerVariant = "icon",
}: AvatarDropdownProps) {
  const { t } = useTranslation("components")
  const openAccountMenuLabel = t("header.common.openAccountMenu")
  const logoutLabel = t("header.me.logout")

  const trigger = triggerVariant === "sidebar" ? (
    <button
      type="button"
      className="flex w-full cursor-pointer items-center gap-3 overflow-hidden rounded-xl bg-transparent px-2.5 py-2 text-left outline-hidden transition group-data-[collapsible=icon]:w-10 group-data-[collapsible=icon]:gap-0 group-data-[collapsible=icon]:rounded-full group-data-[collapsible=icon]:px-0 group-data-[collapsible=icon]:py-0"
      aria-label={openAccountMenuLabel}
      onClick={onTriggerSelect}
    >
      <span className="flex h-10 w-10 shrink-0 items-center justify-center overflow-hidden rounded-full">
        {avatarSrc ? (
          <img
            src={avatarSrc}
            alt={avatarAlt}
            className="block h-10 w-10 rounded-full object-cover"
          />
        ) : null}
        {!avatarSrc ? <AvatarPlaceholder size="lg" /> : null}
      </span>
      <span className="min-w-0 flex-1 group-data-[collapsible=icon]:hidden">
        <span className="block truncate text-sm font-semibold text-foreground">
          {displayName}
        </span>
        <span className="block min-w-0 text-xs text-muted-foreground">
          {displayId}
        </span>
      </span>
    </button>
  ) : (
    <button
      type="button"
      className="relative z-10 flex h-7 w-7 shrink-0 items-center justify-center overflow-hidden rounded-full border border-white/35 bg-[#121821] p-0.5 outline-hidden transition hover:border-white/55 focus-visible:ring-2 focus-visible:ring-(--app-primary)"
      aria-label={openAccountMenuLabel}
      onClick={onTriggerSelect}
    >
      <span className="flex h-6 w-6 items-center justify-center overflow-hidden rounded-full">
        {avatarSrc ? (
          <img
            src={avatarSrc}
            alt={avatarAlt}
            className="block h-6 w-6 rounded-full object-cover"
          />
        ) : null}
        {!avatarSrc ? <AvatarPlaceholder size="sm" /> : null}
      </span>
    </button>
  )

  if (onTriggerSelect) {
    return trigger
  }

  return (
    <DropdownMenu>
      <DropdownMenuTrigger asChild>
        {trigger}
      </DropdownMenuTrigger>

      <DropdownMenuContent
        mode="primitive"
        align="end"
        sideOffset={10}
        className="z-[2000] w-56 overflow-hidden rounded-[10px] border border-[var(--border)] bg-[var(--surface)] p-0 text-[var(--surface-foreground)] opacity-100 shadow-[0_24px_80px_rgba(0,0,0,0.55)] outline-hidden"
      >
        <div className="flex items-center gap-2 px-2 py-2">
          <span className="flex h-8 w-8 shrink-0 items-center justify-center overflow-hidden rounded-full">
            {avatarSrc ? (
              <img
                src={avatarSrc}
                alt={avatarAlt}
                className="block h-8 w-8 rounded-full object-cover"
              />
            ) : null}
            {!avatarSrc ? <AvatarPlaceholder size="md" /> : null}
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
            <div className="grid gap-1 bg-slate-50/80 p-1 dark:bg-white/[0.035]">
              {children}
            </div>
            <div className="h-px bg-slate-50/80 dark:bg-white/[0.035]" />
          </>
        ) : null}

        {logout ? (
          <div className="bg-slate-50/80 p-1 dark:bg-white/[0.035]">
            <DropdownMenuItem
              mode="primitive"
              className={`${itemClassName()} text-destructive! hover:text-destructive! focus:text-destructive! [&_svg]:text-destructive!`}
              onSelect={handleAction(logout)}
              disabled={logout.disabled}
            >
              <LogOut className="size-5 shrink-0" />
              <span>{logout.label ?? logoutLabel}</span>
            </DropdownMenuItem>
          </div>
        ) : null}
      </DropdownMenuContent>
    </DropdownMenu>
  )
}
