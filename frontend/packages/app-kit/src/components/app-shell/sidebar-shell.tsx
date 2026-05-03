import { useEffect, useState, type ReactNode } from "react"
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
  useSidebar,
} from "@workspace/ui-components"
import { TooltipProvider } from "../tooltip"

export interface SidebarShellSubEntry {
  key?: string
  label: ReactNode
  href: string
  active?: boolean
  onSelect?: () => void
}

export interface SidebarShellNavEntry {
  key?: string
  label: ReactNode
  href?: string
  active?: boolean
  icon?: ReactNode
  onSelect?: () => void
  subItems?: SidebarShellSubEntry[]
}

export interface SidebarShellSection {
  key?: string
  label: ReactNode
  items: SidebarShellNavEntry[]
}

export interface SidebarShellProps {
  brandEyebrow?: ReactNode
  brandTitle: ReactNode
  brandDescription?: ReactNode
  sections: SidebarShellSection[]
  header?: ReactNode
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
  brandTitle,
  brandDescription,
  sections,
  header,
  mobileNavigation,
  footerAccount,
  children,
}: SidebarShellProps) {
  return (
    <div className="h-svh overflow-hidden [&_[data-sidebar=trigger]]:max-md:hidden">
      <TooltipProvider>
        <SidebarProvider defaultOpen>
          <div className="hidden md:contents">
            <Sidebar collapsible="icon">
              <SidebarHeader>
                <div className="flex items-center gap-2 group-data-[collapsible=icon]:gap-0">
                  <div className="flex size-8 shrink-0 items-center justify-center rounded-lg bg-sidebar-primary text-sm font-semibold text-sidebar-primary-foreground">
                    {typeof brandTitle === "string" ? brandTitle.charAt(0).toUpperCase() : "W"}
                  </div>
                  <div className="min-w-0 flex-1 group-data-[collapsible=icon]:hidden">
                    <div>{brandTitle}</div>
                    {(brandDescription ?? brandEyebrow) ? (
                      <div>{brandDescription ?? brandEyebrow}</div>
                    ) : null}
                  </div>
                </div>
              </SidebarHeader>

              <div aria-hidden="true" className="h-px bg-sidebar-border" />

              <SidebarContent>
                {sections.map((section) => (
                  <SidebarGroup key={section.key ?? String(section.label)}>
                    <SidebarGroupLabel>{section.label}</SidebarGroupLabel>
                    <SidebarGroupContent>
                      <SidebarMenu>
                        {section.items.map((item) => (
                          <SidebarNavRow key={item.key ?? item.href ?? String(item.label)} item={item} />
                        ))}
                      </SidebarMenu>
                    </SidebarGroupContent>
                  </SidebarGroup>
                ))}
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

              <SidebarRail />
            </Sidebar>
          </div>

          <SidebarInset>
            <div className="flex h-svh min-h-0 min-w-0 flex-col overflow-hidden">
              {header}
              <div className="flex min-h-0 min-w-0 flex-1 overflow-hidden p-0 pb-[calc(4.25rem+env(safe-area-inset-bottom))] md:p-3">
                <div className="flex min-h-0 min-w-0 flex-1 overflow-x-hidden overflow-y-auto md:pr-1">
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

function SidebarNavRow({ item }: { item: SidebarShellNavEntry }) {
  const { state } = useSidebar()
  const hasSubItems = Boolean(item.subItems?.length)

  if (!hasSubItems) {
    return (
      <SidebarMenuItem>
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
              <SidebarMenuSubButton
                active={subItem.active}
                href={subItem.href}
                onClick={subItem.onSelect}
              >
                {subItem.label}
              </SidebarMenuSubButton>
            </SidebarMenuSubItem>
          ))}
        </SidebarMenuSub>
      ) : null}
    </SidebarMenuItem>
  )
}
