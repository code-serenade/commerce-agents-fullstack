import { useEffect, useState, type ReactNode } from "react"
import { ChevronRight } from "lucide-react"
import {
  AvatarDropdown,
  AvatarDropdownItem,
  type AvatarDropdownAction,
} from "./avatar-dropdown"
import {
  MobileBottomNav,
  type MobileBottomNavItem,
} from "./mobile-bottom-nav"
import {
  SidebarFooter,
  Sidebar,
  SidebarContent,
  SidebarGroup,
  SidebarGroupContent,
  SidebarGroupLabel,
  SidebarHeader,
  SidebarInset,
  SidebarMenu,
  SidebarMenuButton,
  SidebarMenuItem,
  SidebarMenuSub,
  SidebarMenuSubButton,
  SidebarMenuSubItem,
  SidebarProvider,
  SidebarRail,
  SidebarTrigger,
  useSidebar,
} from "@workspace/ui-components"
import { TooltipProvider } from "../tooltip"
import { useTranslation } from "react-i18next"

export interface SidebarShellSubEntry {
  key?: string
  label: ReactNode
  href: string
  active?: boolean
  onSelect?: () => void
  action?: ReactNode
}

export interface SidebarShellNavEntry {
  key?: string
  label: ReactNode
  href?: string
  active?: boolean
  icon?: ReactNode
  onSelect?: () => void
  action?: ReactNode
  subItems?: SidebarShellSubEntry[]
}

export interface SidebarShellSection {
  key?: string
  label: ReactNode
  action?: ReactNode
  leadingItems?: SidebarShellNavEntry[]
  items: SidebarShellNavEntry[]
  collapsible?: boolean
  defaultOpen?: boolean
  maxContentHeight?: "sm" | "md" | "lg"
  open?: boolean
  toggleLabel?: string
  onOpenChange?: (open: boolean) => void
}

export interface SidebarShellProps {
  brandEyebrow?: ReactNode
  brandLogo?: ReactNode
  brandTitle: ReactNode
  brandDescription?: ReactNode
  defaultSidebarOpen?: boolean
  sections: SidebarShellSection[]
  header?: ReactNode
  showSidebarRail?: boolean
  contentScroll?: "auto" | "hidden"
  mobileNavigation?: {
    label?: string
    items: MobileBottomNavItem[]
  }
  footerAccount?: {
    avatarSrc?: string
    avatarAlt: string
    avatarFallback: ReactNode
    displayName: ReactNode
    displayId: ReactNode
    actions?: Array<{
      icon?: ReactNode
      label: ReactNode
      onSelect?: () => void
      href?: string
      disabled?: boolean
    }>
    logout?: AvatarDropdownAction
  }
  children: ReactNode
}

export function SidebarShell({
  brandEyebrow,
  brandLogo,
  brandTitle,
  brandDescription,
  defaultSidebarOpen = true,
  sections,
  header,
  showSidebarRail = true,
  contentScroll = "auto",
  mobileNavigation,
  footerAccount,
  children,
}: SidebarShellProps) {
  const { t } = useTranslation("components")
  const contentClassName = [
    "flex min-h-0 min-w-0 flex-1",
    contentScroll === "hidden"
      ? "overflow-hidden"
      : "overflow-x-hidden overflow-y-auto [scrollbar-gutter:stable] md:pr-1",
  ].join(" ")

  return (
    <div className="h-svh overflow-hidden [&_[data-sidebar=trigger]]:max-md:hidden">
      <TooltipProvider>
        <SidebarProvider defaultOpen={defaultSidebarOpen}>
          <div className="hidden md:contents">
            <Sidebar collapsible="icon">
              <SidebarHeader>
                <div className="flex min-h-10 items-center gap-2 group-data-[collapsible=icon]:min-h-20 group-data-[collapsible=icon]:flex-col group-data-[collapsible=icon]:justify-center">
                  <div className="flex min-w-0 flex-1 items-center gap-2 group-data-[collapsible=icon]:flex-none group-data-[collapsible=icon]:justify-center">
                    {brandLogo ? (
                      <div className="shrink-0">{brandLogo}</div>
                    ) : (
                      <div className="flex size-8 shrink-0 items-center justify-center rounded-lg bg-sidebar-primary text-sm font-semibold text-sidebar-primary-foreground">
                        {typeof brandTitle === "string" ? brandTitle.charAt(0).toUpperCase() : "W"}
                      </div>
                    )}
                    <div className="min-w-0 flex-1 group-data-[collapsible=icon]:hidden">
                      <div>{brandTitle}</div>
                      {(brandDescription ?? brandEyebrow) ? (
                        <div>{brandDescription ?? brandEyebrow}</div>
                      ) : null}
                    </div>
                  </div>
                  <div className="shrink-0 rounded-md border border-sidebar-border bg-background shadow-sm transition-colors hover:bg-sidebar-accent [&_[data-sidebar=trigger]]:size-8">
                    <SidebarTrigger ariaLabel={t("header.common.toggleSidebar")} />
                  </div>
                </div>
              </SidebarHeader>

              <div aria-hidden="true" className="h-px bg-sidebar-border" />

              <SidebarContent>
                <div className="flex min-h-0 flex-1 flex-col overflow-x-hidden overflow-y-auto pr-1 [scrollbar-gutter:stable] group-data-[collapsible=icon]:overflow-hidden group-data-[collapsible=icon]:pr-0">
                  {sections.map((section) => (
                    <SidebarSectionGroup
                      key={section.key ?? String(section.label)}
                      section={section}
                    />
                  ))}
                </div>
              </SidebarContent>

              {footerAccount ? (
                <>
                  <div aria-hidden="true" className="h-px bg-sidebar-border" />
                  <SidebarFooter>
                    <AvatarDropdown
                      triggerVariant="sidebar"
                      avatarAlt={footerAccount.avatarAlt}
                      avatarSrc={footerAccount.avatarSrc}
                      avatarFallback={footerAccount.avatarFallback}
                      displayName={footerAccount.displayName}
                      displayId={footerAccount.displayId}
                      logout={footerAccount.logout}
                    >
                      {footerAccount.actions?.map((action, index) => (
                        <AvatarDropdownItem
                          key={index}
                          icon={action.icon}
                          label={action.label}
                          onSelect={action.onSelect}
                          href={action.href}
                          disabled={action.disabled}
                        />
                      ))}
                    </AvatarDropdown>
                  </SidebarFooter>
                </>
              ) : null}

              {showSidebarRail ? <SidebarRail /> : null}
            </Sidebar>
          </div>

          <SidebarInset>
            <div className="flex h-svh min-h-0 min-w-0 flex-col overflow-hidden">
              {header}
              <div className="flex min-h-0 min-w-0 flex-1 overflow-hidden p-0 pb-[calc(4.25rem+env(safe-area-inset-bottom))] md:p-3">
                <div className={contentClassName}>
                  {children}
                </div>
              </div>
            </div>
          </SidebarInset>

          {mobileNavigation ? (
            <MobileBottomNav
              label={mobileNavigation.label}
              items={mobileNavigation.items}
            />
          ) : null}
        </SidebarProvider>
      </TooltipProvider>
    </div>
  )
}

function SidebarSectionGroup({ section }: { section: SidebarShellSection }) {
  const [uncontrolledOpen, setUncontrolledOpen] = useState(
    section.defaultOpen ?? true
  )
  const open = section.collapsible
    ? (section.open ?? uncontrolledOpen)
    : true

  const setOpen = (nextOpen: boolean) => {
    if (section.open === undefined) {
      setUncontrolledOpen(nextOpen)
    }
    section.onOpenChange?.(nextOpen)
  }

  return (
    <SidebarGroup>
      {section.leadingItems?.length ? (
        <SidebarGroupContent>
          <SidebarMenu>
            {section.leadingItems.map((item) => (
              <SidebarNavRow
                key={item.key ?? item.href ?? String(item.label)}
                item={item}
              />
            ))}
          </SidebarMenu>
        </SidebarGroupContent>
      ) : null}
      {section.collapsible ? (
        <div className="flex min-w-0 items-center gap-1 group-data-[collapsible=icon]:hidden">
          <button
            type="button"
            aria-expanded={open}
            aria-label={section.toggleLabel}
            className="flex h-8 min-w-0 flex-1 shrink-0 items-center gap-2 rounded-md px-2 text-xs font-medium text-sidebar-foreground/70 ring-sidebar-ring transition-colors outline-hidden hover:bg-sidebar-accent hover:text-sidebar-accent-foreground focus-visible:ring-2"
            onClick={() => setOpen(!open)}
          >
            <span className="min-w-0 flex-1 truncate text-left">
              {section.label}
            </span>
            <ChevronRight
              className={[
                "size-3.5 shrink-0 transition-transform",
                open ? "rotate-90" : "",
              ]
                .filter(Boolean)
                .join(" ")}
              aria-hidden="true"
            />
          </button>
          {section.action ? (
            <div className="shrink-0">{section.action}</div>
          ) : null}
        </div>
      ) : (
        <SidebarGroupLabel>{section.label}</SidebarGroupLabel>
      )}
      {open ? (
        <SidebarGroupContent>
          <div className={getSectionContentClassName(section.maxContentHeight)}>
            <SidebarMenu>
              {section.items.map((item) => (
                <SidebarNavRow
                  key={item.key ?? item.href ?? String(item.label)}
                  item={item}
                />
              ))}
            </SidebarMenu>
          </div>
        </SidebarGroupContent>
      ) : null}
    </SidebarGroup>
  )
}

function getSectionContentClassName(
  maxContentHeight: SidebarShellSection["maxContentHeight"]
) {
  const classNames = ["min-w-0"]

  if (maxContentHeight === "sm") {
    classNames.push("max-h-56 overflow-y-auto pr-1")
  } else if (maxContentHeight === "md") {
    classNames.push("max-h-72 overflow-y-auto pr-1")
  } else if (maxContentHeight === "lg") {
    classNames.push("max-h-80 overflow-y-auto pr-1")
  }

  return classNames.join(" ")
}

function SidebarNavRow({ item }: { item: SidebarShellNavEntry }) {
  const { state } = useSidebar()
  const hasSubItems = Boolean(item.subItems?.length)

  if (!hasSubItems) {
    return (
      <SidebarMenuItem>
        {item.action ? (
          <div className="group/item flex min-w-0 items-center gap-1">
            <div className="min-w-0 flex-1">
              <SidebarMenuButton
                active={item.active}
                variant="primary"
                tooltip={typeof item.label === "string" ? item.label : undefined}
                onClick={item.onSelect}
              >
                {item.icon}
                <span className="truncate group-data-[collapsible=icon]:hidden">
                  {item.label}
                </span>
              </SidebarMenuButton>
            </div>
            <div className="shrink-0 opacity-0 transition-opacity group-hover/item:opacity-100 focus-within:opacity-100 group-data-[collapsible=icon]:hidden">
              {item.action}
            </div>
          </div>
        ) : (
          <SidebarMenuButton
            active={item.active}
            variant="primary"
            tooltip={typeof item.label === "string" ? item.label : undefined}
            onClick={item.onSelect}
          >
            {item.icon}
            <span className="truncate group-data-[collapsible=icon]:hidden">
              {item.label}
            </span>
          </SidebarMenuButton>
        )}
      </SidebarMenuItem>
    )
  }

  const hasActiveChild = item.subItems?.some((subItem) => subItem.active) ?? false
  const [open, setOpen] = useState(hasActiveChild)

  useEffect(() => {
    if (hasActiveChild) {
      setOpen(true)
    }
  }, [hasActiveChild])

  return (
    <SidebarMenuItem>
      <SidebarMenuButton
        active={item.active}
        activeGroup={hasActiveChild}
        variant="primary"
        disclosure
        disclosureOpen={open}
        tooltip={typeof item.label === "string" ? item.label : undefined}
        onClick={() => {
          if (state === "collapsed") {
            if (hasActiveChild) {
              return
            }

            item.subItems?.[0]?.onSelect?.()
            return
          }

          setOpen((current) => !current)
        }}
      >
        {item.icon}
        <span className="truncate group-data-[collapsible=icon]:hidden">
          {item.label}
        </span>
      </SidebarMenuButton>

      {open ? (
        <SidebarMenuSub>
          {item.subItems?.map((subItem) => (
            <SidebarMenuSubItem key={subItem.key ?? subItem.href}>
              {subItem.action ? (
                <div className="group/subitem flex min-w-0 items-center gap-1">
                  <div className="min-w-0 flex-1">
                    <SidebarMenuSubButton
                      active={subItem.active}
                      href={subItem.href}
                      onClick={subItem.onSelect}
                    >
                      {subItem.label}
                    </SidebarMenuSubButton>
                  </div>
                  <div className="shrink-0 opacity-0 transition-opacity group-hover/subitem:opacity-100 focus-within:opacity-100">
                    {subItem.action}
                  </div>
                </div>
              ) : (
                <SidebarMenuSubButton
                  active={subItem.active}
                  href={subItem.href}
                  onClick={subItem.onSelect}
                >
                  {subItem.label}
                </SidebarMenuSubButton>
              )}
            </SidebarMenuSubItem>
          ))}
        </SidebarMenuSub>
      ) : null}
    </SidebarMenuItem>
  )
}
