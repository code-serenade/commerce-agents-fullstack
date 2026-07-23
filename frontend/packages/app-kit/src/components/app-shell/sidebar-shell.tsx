import { useEffect, useState, type ReactNode } from "react"
import { ChevronRight, MoreHorizontal } from "lucide-react"
import {
  AvatarDropdown,
  AvatarDropdownItem,
  type AvatarDropdownAction,
} from "./avatar-dropdown"
import { MobileBottomNav, type MobileBottomNavItem } from "./mobile-bottom-nav"
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
  iconSize?: "default" | "xs"
  iconTone?: "default" | "muted"
  tone?: "default" | "muted"
  textSize?: "default" | "lg"
  variant?: "default" | "primaryAction" | "prominentAction" | "plainText"
  icon?: ReactNode
  onSelect?: () => void
  action?: ReactNode
  subItems?: SidebarShellSubEntry[]
}

export interface SidebarShellItemOverflowConfig {
  maxVisibleItems: number
  defaultOpen?: boolean
  moreLabel: ReactNode
  lessLabel: ReactNode
}

export interface SidebarShellSection {
  key?: string
  label: ReactNode
  hideLabel?: boolean
  action?: ReactNode
  contentClassName?: string
  leadingItems?: SidebarShellNavEntry[]
  leadingItemsFixed?: boolean
  items: SidebarShellNavEntry[]
  hideWhenSidebarCollapsed?: boolean
  collapsible?: boolean
  defaultOpen?: boolean
  fillRemaining?: boolean
  maxContentHeight?: "sm" | "md" | "lg"
  open?: boolean
  toggleLabel?: string
  toggleHoverTone?: "default" | "transparent"
  onOpenChange?: (open: boolean) => void
  itemOverflow?: SidebarShellItemOverflowConfig
}

export interface SidebarShellProps {
  brandEyebrow?: ReactNode
  brandLogo?: ReactNode
  brandTitle: ReactNode
  brandDescription?: ReactNode
  onBrandSelect?: () => void
  defaultSidebarOpen?: boolean
  sections: SidebarShellSection[]
  header?: ReactNode
  showSidebarRail?: boolean
  contentScroll?: "auto" | "hidden"
  contentPadding?: "default" | "none"
  mobileNavigation?: {
    label?: string
    items: MobileBottomNavItem[]
  }
  navigationHeader?: ReactNode
  footerAccount?: {
    avatarSrc?: string
    avatarAlt: string
    avatarFallback: ReactNode
    displayName: ReactNode
    displayId: ReactNode
    onSelect?: () => void
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

const sidebarMenuSpacingClassName = "[&_[data-sidebar=menu]]:gap-0"

const collapsedSidebarIconLayoutClassName =
  "contents group-data-[collapsible=icon]:[&_[data-sidebar=group]]:items-center group-data-[collapsible=icon]:[&_[data-sidebar=group]]:p-0 group-data-[collapsible=icon]:[&_[data-sidebar=group-content]]:flex group-data-[collapsible=icon]:[&_[data-sidebar=group-content]]:justify-center group-data-[collapsible=icon]:[&_[data-sidebar=menu]]:w-10 group-data-[collapsible=icon]:[&_[data-sidebar=menu]]:items-center group-data-[collapsible=icon]:[&_[data-sidebar=menu-item]]:flex group-data-[collapsible=icon]:[&_[data-sidebar=menu-item]]:h-10 group-data-[collapsible=icon]:[&_[data-sidebar=menu-item]]:items-center group-data-[collapsible=icon]:[&_[data-sidebar=menu-item]]:justify-center group-data-[collapsible=icon]:[&_[data-sidebar=menu-button]]:mx-auto group-data-[collapsible=icon]:[&_[data-sidebar=menu-button]]:justify-center"

const sidebarMenuButtonBaseClassName =
  "[&_[data-sidebar=menu-button]]:h-10 [&_[data-sidebar=menu-button]]:cursor-pointer [&_[data-sidebar=menu-button]]:rounded-lg [&_[data-sidebar=menu-button]]:px-3 [&_[data-sidebar=menu-button]]:shadow-none [&_[data-sidebar=menu-button]]:hover:bg-(--shell-sidebar-item-hover-background) [&_[data-sidebar=menu-button]]:hover:text-sidebar-foreground [&_[data-sidebar=menu-button]]:active:bg-(--shell-sidebar-item-hover-background) [&_[data-sidebar=menu-button]]:active:text-sidebar-foreground [&_[data-sidebar=menu-button][data-active=true]]:bg-white [&_[data-sidebar=menu-button][data-active=true]]:shadow-[0_1px_2px_rgba(15,23,42,0.04)] [&_[data-sidebar=menu-button][data-active=true]]:hover:bg-white dark:[&_[data-sidebar=menu-button][data-active=true]]:bg-black dark:[&_[data-sidebar=menu-button][data-active=true]]:shadow-[0_1px_2px_rgba(0,0,0,0.12)] dark:[&_[data-sidebar=menu-button][data-active=true]]:hover:bg-black"

const sidebarPrimaryActionButtonClassName =
  "[&_[data-sidebar=menu-button]]:h-10 [&_[data-sidebar=menu-button]]:cursor-pointer [&_[data-sidebar=menu-button]]:rounded-lg [&_[data-sidebar=menu-button]]:border [&_[data-sidebar=menu-button]]:border-primary [&_[data-sidebar=menu-button]]:bg-primary/10 [&_[data-sidebar=menu-button]]:text-sm [&_[data-sidebar=menu-button]]:font-bold [&_[data-sidebar=menu-button]]:text-primary [&_[data-sidebar=menu-button]]:shadow-none [&_[data-sidebar=menu-button]]:hover:border-primary [&_[data-sidebar=menu-button]]:hover:bg-primary/15 [&_[data-sidebar=menu-button]]:hover:text-primary [&_[data-sidebar=menu-button][data-active=true]]:border-primary [&_[data-sidebar=menu-button][data-active=true]]:bg-primary/10 [&_[data-sidebar=menu-button][data-active=true]]:text-primary [&_[data-sidebar=menu-button][data-active=true]]:shadow-[0_1px_2px_rgba(15,23,42,0.04)] [&_[data-sidebar=menu-button][data-active=true]]:hover:bg-primary/10 [&_[data-sidebar=menu-button][data-active=true]]:hover:text-primary"

const sidebarProminentActionButtonClassName =
  "[&_[data-sidebar=menu-button]]:!h-10 [&_[data-sidebar=menu-button]]:!rounded-lg [&_[data-sidebar=menu-button]]:border [&_[data-sidebar=menu-button]]:border-sidebar-border [&_[data-sidebar=menu-button]]:bg-background [&_[data-sidebar=menu-button]]:!px-3 [&_[data-sidebar=menu-button]]:text-sm [&_[data-sidebar=menu-button]]:font-medium [&_[data-sidebar=menu-button]]:text-sidebar-foreground [&_[data-sidebar=menu-button]]:shadow-[0_1px_3px_rgba(100,116,139,0.18)] [&_[data-sidebar=menu-button]]:hover:border-sidebar-primary/35 [&_[data-sidebar=menu-button]]:hover:bg-background [&_[data-sidebar=menu-button]]:hover:text-sidebar-foreground group-data-[collapsible=icon]:[&_[data-sidebar=menu-button]]:justify-center group-data-[collapsible=icon]:[&_[data-sidebar=menu-button]]:!border-transparent group-data-[collapsible=icon]:[&_[data-sidebar=menu-button]]:!bg-transparent group-data-[collapsible=icon]:[&_[data-sidebar=menu-button]]:!px-2 group-data-[collapsible=icon]:[&_[data-sidebar=menu-button]]:!shadow-none group-data-[collapsible=icon]:[&_[data-sidebar=menu-button][data-active=true]]:!border-transparent group-data-[collapsible=icon]:[&_[data-sidebar=menu-button][data-active=true]]:!bg-transparent group-data-[collapsible=icon]:[&_[data-sidebar=menu-button][data-active=true]]:!shadow-none dark:[&_[data-sidebar=menu-button]]:border-sidebar-border/80 dark:[&_[data-sidebar=menu-button]]:bg-sidebar-accent/55 dark:[&_[data-sidebar=menu-button]]:text-sidebar-foreground dark:[&_[data-sidebar=menu-button]]:shadow-[0_1px_3px_rgba(148,163,184,0.12)] dark:[&_[data-sidebar=menu-button]]:hover:border-sidebar-primary/45 dark:[&_[data-sidebar=menu-button]]:hover:bg-sidebar-accent/75 dark:[&_[data-sidebar=menu-button][data-active=true]]:border-sidebar-primary/50 dark:[&_[data-sidebar=menu-button][data-active=true]]:bg-sidebar-accent/65 dark:[&_[data-sidebar=menu-button][data-active=true]]:text-sidebar-foreground dark:[&_[data-sidebar=menu-button][data-active=true]]:shadow-[0_1px_3px_rgba(148,163,184,0.12)] dark:[&_[data-sidebar=menu-button][data-active=true]]:hover:bg-sidebar-accent/65 [&_[data-sidebar=menu-button][data-active=true]]:border-sidebar-primary/40 [&_[data-sidebar=menu-button][data-active=true]]:bg-background [&_[data-sidebar=menu-button][data-active=true]]:text-sidebar-foreground [&_[data-sidebar=menu-button][data-active=true]]:shadow-[0_1px_3px_rgba(100,116,139,0.18)] [&_[data-sidebar=menu-button][data-active=true]]:hover:bg-background [&_[data-sidebar=menu-button][data-active=true]]:hover:text-sidebar-foreground group-data-[collapsible=icon]:[&_[data-sidebar=menu-button]]:!border-transparent group-data-[collapsible=icon]:[&_[data-sidebar=menu-button]]:!bg-transparent group-data-[collapsible=icon]:[&_[data-sidebar=menu-button]]:!shadow-none group-data-[collapsible=icon]:[&_[data-sidebar=menu-button][data-active=true]]:!border-transparent group-data-[collapsible=icon]:[&_[data-sidebar=menu-button][data-active=true]]:!bg-transparent group-data-[collapsible=icon]:[&_[data-sidebar=menu-button][data-active=true]]:!shadow-none [&_[data-sidebar=menu-button]_svg]:text-sidebar-foreground"

export function SidebarShell({
  brandEyebrow,
  brandLogo,
  brandTitle,
  brandDescription,
  onBrandSelect,
  defaultSidebarOpen = true,
  sections,
  header,
  showSidebarRail = true,
  contentScroll = "auto",
  contentPadding = "default",
  mobileNavigation,
  navigationHeader,
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
  const contentFrameClassName = [
    "flex min-h-0 min-w-0 flex-1 overflow-hidden p-0 pb-[calc(4.25rem+env(safe-area-inset-bottom))] md:pb-0",
    contentPadding === "default" ? "md:p-3" : "md:p-0",
  ].join(" ")
  const brandSecondaryText = getDistinctBrandSecondaryText(
    brandTitle,
    brandDescription ?? brandEyebrow
  )

  return (
    <div className="app-shell-root h-dvh overflow-hidden [&_[data-sidebar=trigger]]:max-md:hidden">
      <TooltipProvider>
        <SidebarProvider defaultOpen={defaultSidebarOpen}>
          <div className="hidden md:contents">
            <Sidebar collapsible="icon">
              <div className={collapsedSidebarIconLayoutClassName}>
                <SidebarHeader>
                  <div className="flex h-12 items-center gap-2 group-data-[collapsible=icon]:h-[4.75rem] group-data-[collapsible=icon]:flex-col group-data-[collapsible=icon]:justify-start group-data-[collapsible=icon]:gap-1.5 group-data-[collapsible=icon]:pt-1">
                    {onBrandSelect ? (
                      <button
                        type="button"
                        className="flex min-w-0 flex-1 cursor-pointer items-center gap-2 rounded-lg text-left outline-hidden transition-colors group-data-[collapsible=icon]:mt-0.5 group-data-[collapsible=icon]:flex-none group-data-[collapsible=icon]:justify-center focus-visible:ring-2 focus-visible:ring-(--app-primary)"
                        onClick={onBrandSelect}
                      >
                        <BrandContent
                          brandLogo={brandLogo}
                          brandSecondaryText={brandSecondaryText}
                          brandTitle={brandTitle}
                        />
                      </button>
                    ) : (
                      <div className="flex min-w-0 flex-1 items-center gap-2 group-data-[collapsible=icon]:mt-0.5 group-data-[collapsible=icon]:flex-none group-data-[collapsible=icon]:justify-center">
                        <BrandContent
                          brandLogo={brandLogo}
                          brandSecondaryText={brandSecondaryText}
                          brandTitle={brandTitle}
                        />
                      </div>
                    )}
                    <div className="shrink-0 rounded-lg border border-sidebar-border bg-(--shell-sidebar-trigger-background) shadow-sm transition-colors hover:bg-sidebar-accent [&_[data-sidebar=trigger]]:size-8">
                      <SidebarTrigger
                        ariaLabel={t("header.common.toggleSidebar")}
                      />
                    </div>
                  </div>
                </SidebarHeader>

                <SidebarContent>
                  {navigationHeader ? (
                    <div className="px-2 pt-1 group-data-[collapsible=icon]:hidden">
                      {navigationHeader}
                    </div>
                  ) : null}
                  <FixedLeadingItems sections={sections} />
                  <div className="flex min-h-0 flex-1 basis-0 flex-col overflow-hidden group-data-[collapsible=icon]:overflow-hidden">
                    {sections.map((section) => (
                      <SidebarSectionGroup
                        key={section.key ?? String(section.label)}
                        section={section}
                        showLeadingItems={!section.leadingItemsFixed}
                      />
                    ))}
                  </div>
                </SidebarContent>

                {footerAccount ? (
                  <>
                    <div
                      aria-hidden="true"
                      className="h-px bg-sidebar-border"
                    />
                    <div className="z-20 shrink-0 bg-sidebar bg-[image:var(--shell-sidebar-background)] group-data-[collapsible=icon]:bg-transparent group-data-[collapsible=icon]:bg-none">
                      <SidebarFooter>
                        <AvatarDropdown
                          triggerVariant="sidebar"
                          avatarAlt={footerAccount.avatarAlt}
                          avatarSrc={footerAccount.avatarSrc}
                          avatarFallback={footerAccount.avatarFallback}
                          displayName={footerAccount.displayName}
                          displayId={footerAccount.displayId}
                          onTriggerSelect={footerAccount.onSelect}
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
                    </div>
                  </>
                ) : null}

                {showSidebarRail ? <SidebarRail /> : null}
              </div>
            </Sidebar>
          </div>

          <SidebarInset>
            <div className="flex h-dvh min-h-0 min-w-0 flex-col overflow-hidden">
              {header}
              <div className={contentFrameClassName}>
                <div className={contentClassName}>{children}</div>
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

function BrandContent({
  brandLogo,
  brandSecondaryText,
  brandTitle,
}: {
  brandLogo?: ReactNode
  brandSecondaryText: ReactNode
  brandTitle: ReactNode
}) {
  return (
    <>
      {brandLogo ? (
        <div className="shrink-0">{brandLogo}</div>
      ) : (
        <div className="flex size-8 shrink-0 items-center justify-center rounded-lg bg-sidebar-primary text-sm font-semibold text-sidebar-primary-foreground">
          {typeof brandTitle === "string"
            ? brandTitle.charAt(0).toUpperCase()
            : "W"}
        </div>
      )}
      <div className="min-w-0 flex-1 group-data-[collapsible=icon]:hidden">
        <div className="auth-login-brand-title text-base leading-tight font-bold">
          {brandTitle}
        </div>
        {brandSecondaryText ? (
          <div className="text-[12px] text-(--shell-brand-description-foreground)">
            {brandSecondaryText}
          </div>
        ) : null}
      </div>
    </>
  )
}

function FixedLeadingItems({ sections }: { sections: SidebarShellSection[] }) {
  const fixedItems = sections.flatMap((section) =>
    section.leadingItemsFixed ? (section.leadingItems ?? []) : []
  )

  if (!fixedItems.length) return null

  return (
    <div className="shrink-0 group-data-[collapsible=icon]:pt-0">
      <SidebarGroup>
        <SidebarGroupContent>
          <div className={sidebarMenuSpacingClassName}>
            <SidebarMenu>
              {fixedItems.map((item) => (
                <SidebarNavRow
                  key={item.key ?? item.href ?? String(item.label)}
                  item={item}
                />
              ))}
            </SidebarMenu>
          </div>
        </SidebarGroupContent>
      </SidebarGroup>
    </div>
  )
}

function getDistinctBrandSecondaryText(
  brandTitle: ReactNode,
  brandSecondary: ReactNode
) {
  if (!brandSecondary) return null
  if (
    typeof brandTitle === "string" &&
    typeof brandSecondary === "string" &&
    brandTitle.trim() === brandSecondary.trim()
  ) {
    return null
  }
  return brandSecondary
}

function SidebarSectionGroup({
  section,
  showLeadingItems = true,
}: {
  section: SidebarShellSection
  showLeadingItems?: boolean
}) {
  const [uncontrolledOpen, setUncontrolledOpen] = useState(
    section.defaultOpen ?? true
  )
  const [itemOverflowOpen, setItemOverflowOpen] = useState(
    section.itemOverflow?.defaultOpen ?? false
  )
  const open = section.collapsible ? (section.open ?? uncontrolledOpen) : true

  const setOpen = (nextOpen: boolean) => {
    if (section.open === undefined) {
      setUncontrolledOpen(nextOpen)
    }
    section.onOpenChange?.(nextOpen)
  }

  const sectionClassName = [
    section.fillRemaining ? "flex min-h-0 flex-1 basis-0 flex-col" : "",
    section.hideWhenSidebarCollapsed ? "group-data-[collapsible=icon]:hidden" : "",
  ]
    .filter(Boolean)
    .join(" ")
  const groupClassName = [
    "relative flex w-full min-w-0 flex-col p-2 pb-0",
    section.fillRemaining ? "min-h-0 flex-1 basis-0 overflow-hidden" : "",
  ]
    .filter(Boolean)
    .join(" ")

  return (
    <div className={sectionClassName}>
      <div
        data-slot="sidebar-group"
        data-sidebar="group"
        className={groupClassName}
      >
        {showLeadingItems && section.leadingItems?.length ? (
          <SidebarGroupContentFrame>
            <div className={sidebarMenuSpacingClassName}>
              <SidebarMenu>
                {section.leadingItems.map((item) => (
                  <SidebarNavRow
                    key={item.key ?? item.href ?? String(item.label)}
                    item={item}
                  />
                ))}
              </SidebarMenu>
            </div>
          </SidebarGroupContentFrame>
        ) : null}
        {section.collapsible ? (
          <div className="flex min-w-0 items-center gap-1 group-data-[collapsible=icon]:hidden">
            <button
              type="button"
              aria-expanded={open}
              aria-label={section.toggleLabel}
              className={[
                "flex h-8 min-w-0 flex-1 shrink-0 items-center gap-2 rounded-lg px-2 text-xs font-medium text-sidebar-foreground/70 ring-sidebar-ring outline-hidden transition-colors hover:bg-sidebar-accent hover:text-sidebar-accent-foreground focus-visible:ring-2",
                section.toggleHoverTone === "transparent"
                  ? "cursor-pointer hover:bg-transparent hover:text-sidebar-foreground"
                  : "",
              ]
                .filter(Boolean)
                .join(" ")}
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
        ) : section.hideLabel ? null : (
          <SidebarGroupLabel>{section.label}</SidebarGroupLabel>
        )}
        {open ? (
          <SidebarGroupContentFrame fillRemaining={section.fillRemaining}>
            <div
              className={getSectionContentClassName(
                section.maxContentHeight,
                section.fillRemaining,
                section.contentClassName
              )}
            >
              <div className={sidebarMenuSpacingClassName}>
                <SidebarMenu>
                  <SidebarSectionItems
                    items={section.items}
                    itemOverflow={section.itemOverflow}
                    itemOverflowOpen={itemOverflowOpen}
                    onItemOverflowOpenChange={setItemOverflowOpen}
                  />
                </SidebarMenu>
              </div>
            </div>
          </SidebarGroupContentFrame>
        ) : null}
      </div>
    </div>
  )
}

function getSectionContentClassName(
  maxContentHeight: SidebarShellSection["maxContentHeight"],
  fillRemaining?: boolean,
  contentClassName?: string
) {
  const classNames = [
    "app-shell-scrollarea",
    "min-w-0",
    "group-data-[collapsible=icon]:max-h-none",
    "group-data-[collapsible=icon]:overflow-hidden",
    "group-data-[collapsible=icon]:pr-0",
  ]

  if (fillRemaining) {
    classNames.push(
      "min-h-0 flex-1 basis-0 overflow-x-hidden overflow-y-auto pr-3 [scrollbar-gutter:stable]"
    )
  } else if (maxContentHeight === "sm") {
    classNames.push(
      "max-h-56 overflow-x-hidden overflow-y-auto pr-3 [scrollbar-gutter:stable]"
    )
  } else if (maxContentHeight === "md") {
    classNames.push(
      "max-h-72 overflow-x-hidden overflow-y-auto pr-3 [scrollbar-gutter:stable]"
    )
  } else if (maxContentHeight === "lg") {
    classNames.push(
      "max-h-80 overflow-x-hidden overflow-y-auto pr-3 [scrollbar-gutter:stable]"
    )
  }

  if (contentClassName) {
    classNames.push(contentClassName)
  }

  return classNames.join(" ")
}

function SidebarGroupContentFrame({
  children,
  fillRemaining,
}: {
  children: ReactNode
  fillRemaining?: boolean
}) {
  return (
    <div
      data-slot="sidebar-group-content"
      data-sidebar="group-content"
      className={[
        "w-full text-sm",
        fillRemaining
          ? "flex min-h-0 flex-1 basis-0 flex-col overflow-hidden"
          : "",
      ]
        .filter(Boolean)
        .join(" ")}
    >
      {children}
    </div>
  )
}

function SidebarSectionItems({
  items,
  itemOverflow,
  itemOverflowOpen,
  onItemOverflowOpenChange,
}: {
  items: SidebarShellNavEntry[]
  itemOverflow?: SidebarShellItemOverflowConfig
  itemOverflowOpen: boolean
  onItemOverflowOpenChange: (open: boolean) => void
}) {
  const { state } = useSidebar()
  const isCollapsedSidebar = state === "collapsed"

  if (!itemOverflow) {
    return items.map((item) => (
      <SidebarNavRow
        key={item.key ?? item.href ?? String(item.label)}
        item={item}
      />
    ))
  }

  const maxVisibleItems = Math.max(3, itemOverflow.maxVisibleItems)
  if (items.length <= maxVisibleItems) {
    return items.map((item) => (
      <SidebarNavRow
        key={item.key ?? item.href ?? String(item.label)}
        item={item}
      />
    ))
  }

  const pinnedItems = items.slice(0, maxVisibleItems - 1)
  const overflowItems = items.slice(maxVisibleItems - 1)
  const showOverflowItems = itemOverflowOpen && !isCollapsedSidebar
  const showOverflowControl = !isCollapsedSidebar

  return (
    <>
      {pinnedItems.map((item) => (
        <SidebarNavRow
          key={item.key ?? item.href ?? String(item.label)}
          item={item}
        />
      ))}
      {showOverflowControl ? (
        <SidebarOverflowControlRow
          expanded={itemOverflowOpen}
          lessLabel={itemOverflow.lessLabel}
          moreLabel={itemOverflow.moreLabel}
          overflowItems={overflowItems}
          onExpandedChange={onItemOverflowOpenChange}
        />
      ) : null}
      {showOverflowItems
        ? overflowItems.map((item) => (
            <SidebarNavRow
              key={item.key ?? item.href ?? String(item.label)}
              item={item}
            />
          ))
        : null}
    </>
  )
}

function SidebarOverflowControlRow({
  expanded,
  lessLabel,
  moreLabel,
  overflowItems,
  onExpandedChange,
}: {
  expanded: boolean
  lessLabel: ReactNode
  moreLabel: ReactNode
  overflowItems: SidebarShellNavEntry[]
  onExpandedChange: (expanded: boolean) => void
}) {
  const { state } = useSidebar()
  const isExpandedSidebar = state !== "collapsed"
  const textSize = overflowItems.some((item) => item.textSize === "lg")
    ? "lg"
    : "default"
  const controlLabel = expanded ? lessLabel : moreLabel

  return (
    <SidebarMenuItem>
      <div className="group/overflow relative">
        <div
          className={[
            sidebarMenuButtonBaseClassName,
            textSize === "lg" ? "[&_[data-sidebar=menu-button]]:text-sm" : "",
          ]
            .filter(Boolean)
            .join(" ")}
        >
          <SidebarMenuButton
            active={false}
            disclosure={!expanded}
            disclosureOpen={false}
            variant="primary"
            tooltip={
              typeof controlLabel === "string" ? controlLabel : undefined
            }
            onClick={() => onExpandedChange(!expanded)}
          >
            <MoreHorizontal
              className={[
                "size-4 shrink-0",
                expanded ? "text-sidebar-foreground/55" : "",
              ]
                .filter(Boolean)
                .join(" ")}
            />
            <span
              className={[
                "truncate group-data-[collapsible=icon]:hidden",
                expanded ? "text-sidebar-foreground/55" : "",
              ]
                .filter(Boolean)
                .join(" ")}
            >
              {controlLabel}
            </span>
          </SidebarMenuButton>
        </div>

        {!expanded && isExpandedSidebar ? (
          <div
            aria-hidden="true"
            className="pointer-events-none absolute top-0 left-[calc(100%+0.5rem)] z-50 hidden min-w-40 rounded-lg border border-sidebar-border bg-sidebar p-1.5 text-sidebar-foreground shadow-xl group-focus-within/overflow:block group-hover/overflow:block"
          >
            <div className="grid gap-1">
              {overflowItems.map((item) => (
                <div
                  key={item.key ?? item.href ?? String(item.label)}
                  className={[
                    "flex min-h-9 min-w-0 items-center gap-2 rounded-md px-2 text-left text-sm",
                    item.active
                      ? "bg-sidebar-accent font-medium text-sidebar-primary"
                      : "text-sidebar-foreground",
                  ].join(" ")}
                >
                  <span className="flex size-4 shrink-0 items-center justify-center [&_svg]:size-4">
                    {item.icon}
                  </span>
                  <span className="min-w-0 truncate">{item.label}</span>
                </div>
              ))}
            </div>
          </div>
        ) : null}
      </div>
    </SidebarMenuItem>
  )
}

function SidebarNavRow({ item }: { item: SidebarShellNavEntry }) {
  const { state } = useSidebar()
  const hasSubItems = Boolean(item.subItems?.length)
  const isExpandedActionRow = state !== "collapsed"
  const shouldHighlightActionRow = item.active && isExpandedActionRow
  const navButtonWrapperClassName = [
    sidebarMenuButtonBaseClassName,
    item.variant === "primaryAction" ? sidebarPrimaryActionButtonClassName : "",
    item.variant === "prominentAction"
      ? sidebarProminentActionButtonClassName
      : "",
    item.tone === "muted"
      ? "[&_[data-sidebar=menu-button]]:text-sidebar-foreground/55 [&_[data-sidebar=menu-button]_svg]:text-sidebar-foreground/45"
      : "",
    item.iconTone === "muted"
      ? "[&_[data-sidebar=menu-button]_svg]:text-sidebar-foreground/55"
      : "",
    item.iconSize === "xs" ? "[&_[data-sidebar=menu-button]_svg]:size-3" : "",
    item.textSize === "lg" ? "[&_[data-sidebar=menu-button]]:text-sm" : "",
  ]
    .filter(Boolean)
    .join(" ")

  if (item.variant === "plainText") {
    return (
      <SidebarMenuItem>
        <div className="flex h-10 items-center px-3 text-sm text-sidebar-foreground/55 group-data-[collapsible=icon]:hidden">
          <span className="block min-w-0 truncate">{item.label}</span>
        </div>
      </SidebarMenuItem>
    )
  }

  if (!hasSubItems) {
    return (
      <SidebarMenuItem>
        {item.action ? (
          <div className={navButtonWrapperClassName}>
            <div
              className={[
                "group/item relative flex min-w-0 items-center rounded-lg transition-colors",
                isExpandedActionRow
                  ? "hover:bg-(--shell-sidebar-item-hover-background) [&_[data-sidebar=menu-button]]:group-focus-within/item:pr-10 [&_[data-sidebar=menu-button]]:group-hover/item:pr-10 [&_[data-sidebar=menu-button]]:hover:bg-transparent [&_[data-sidebar=menu-button]]:hover:text-sidebar-foreground [&_[data-sidebar=menu-button]]:active:bg-transparent [&_[data-sidebar=menu-button]]:active:text-sidebar-foreground [&_[data-sidebar=menu-button][data-active=true]]:bg-transparent [&_[data-sidebar=menu-button][data-active=true]]:pr-10 [&_[data-sidebar=menu-button][data-active=true]]:shadow-none"
                  : "",
                shouldHighlightActionRow
                  ? "bg-white shadow-[0_1px_2px_rgba(15,23,42,0.04)] hover:bg-white dark:bg-black dark:shadow-[0_1px_2px_rgba(0,0,0,0.12)] dark:hover:bg-black"
                  : "",
              ]
                .filter(Boolean)
                .join(" ")}
            >
              <div className="min-w-0 flex-1">
                <SidebarMenuButton
                  active={item.active}
                  variant="primary"
                  tooltip={
                    typeof item.label === "string" ? item.label : undefined
                  }
                  onClick={item.onSelect}
                >
                  {item.icon}
                  <span className="min-w-0 flex-1 truncate group-data-[collapsible=icon]:hidden">
                    {item.label}
                  </span>
                </SidebarMenuButton>
              </div>
              <div
                className={[
                  "absolute top-1/2 right-1.5 -translate-y-1/2 bg-transparent transition-opacity group-focus-within/item:opacity-100 group-hover/item:opacity-100 group-data-[collapsible=icon]:hidden",
                  shouldHighlightActionRow ? "opacity-100" : "opacity-0",
                ]
                  .filter(Boolean)
                  .join(" ")}
              >
                {item.action}
              </div>
            </div>
          </div>
        ) : (
          <div className={navButtonWrapperClassName}>
            <SidebarMenuButton
              active={item.active}
              variant="primary"
              tooltip={typeof item.label === "string" ? item.label : undefined}
              onClick={item.onSelect}
            >
              {item.icon}
              <span className="min-w-0 flex-1 truncate group-data-[collapsible=icon]:hidden">
                {item.label}
              </span>
            </SidebarMenuButton>
          </div>
        )}
      </SidebarMenuItem>
    )
  }

  const hasActiveChild =
    item.subItems?.some((subItem) => subItem.active) ?? false
  const [open, setOpen] = useState(hasActiveChild)

  useEffect(() => {
    if (hasActiveChild) {
      setOpen(true)
    }
  }, [hasActiveChild])

  return (
    <SidebarMenuItem>
      <div className={navButtonWrapperClassName}>
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
          <span className="min-w-0 flex-1 truncate group-data-[collapsible=icon]:hidden">
            {item.label}
          </span>
        </SidebarMenuButton>
      </div>

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
