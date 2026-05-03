import type {
  ChangeEvent,
  MouseEvent,
  MouseEventHandler,
  ReactNode,
} from "react"
import {
  Sidebar as CoreSidebar,
  SidebarContent as CoreSidebarContent,
  SidebarFooter as CoreSidebarFooter,
  SidebarGroup as CoreSidebarGroup,
  SidebarGroupAction as CoreSidebarGroupAction,
  SidebarGroupContent as CoreSidebarGroupContent,
  SidebarGroupLabel as CoreSidebarGroupLabel,
  SidebarHeader as CoreSidebarHeader,
  SidebarInput as CoreSidebarInput,
  SidebarInset as CoreSidebarInset,
  SidebarMenu as CoreSidebarMenu,
  SidebarMenuAction as CoreSidebarMenuAction,
  SidebarMenuBadge as CoreSidebarMenuBadge,
  SidebarMenuButton as CoreSidebarMenuButton,
  SidebarMenuItem as CoreSidebarMenuItem,
  SidebarMenuSkeleton as CoreSidebarMenuSkeleton,
  SidebarMenuSub as CoreSidebarMenuSub,
  SidebarMenuSubButton as CoreSidebarMenuSubButton,
  SidebarMenuSubItem as CoreSidebarMenuSubItem,
  SidebarProvider as CoreSidebarProvider,
  SidebarRail as CoreSidebarRail,
  SidebarSeparator as CoreSidebarSeparator,
  SidebarTrigger as CoreSidebarTrigger,
  useSidebar as useCoreSidebar,
} from "@workspace/ui-core/components/sidebar"

export type SidebarSide = "left" | "right"
export type SidebarVariant = "sidebar" | "floating" | "inset"
export type SidebarCollapsible = "offcanvas" | "icon" | "none"
export type SidebarMenuButtonVariant = "default" | "primary" | "outline"
export type SidebarMenuButtonSize = "default" | "sm" | "lg"
export type SidebarMenuSubButtonSize = "sm" | "md"

export interface SidebarProviderProps {
  children: ReactNode
  defaultOpen?: boolean
  open?: boolean
  onOpenChange?: (open: boolean) => void
}

export interface SidebarProps {
  children: ReactNode
  side?: SidebarSide
  variant?: SidebarVariant
  collapsible?: SidebarCollapsible
}

export interface SidebarTriggerProps {
  ariaLabel?: string
}

export interface SidebarRailProps {
  ariaLabel?: string
}

export interface SidebarInsetProps {
  children: ReactNode
}

export interface SidebarInputProps {
  value?: string
  defaultValue?: string
  placeholder?: string
  disabled?: boolean
  onValueChange?: (value: string) => void
}

export interface SidebarHeaderProps {
  children: ReactNode
}

export interface SidebarFooterProps {
  children: ReactNode
}

export interface SidebarContentProps {
  children: ReactNode
}

export interface SidebarGroupProps {
  children: ReactNode
}

export interface SidebarGroupLabelProps {
  children: ReactNode
}

export interface SidebarGroupActionProps {
  children: ReactNode
  ariaLabel: string
  disabled?: boolean
  onClick?: () => void
}

export interface SidebarGroupContentProps {
  children: ReactNode
}

export interface SidebarMenuProps {
  children: ReactNode
}

export interface SidebarMenuItemProps {
  children: ReactNode
}

export interface SidebarMenuButtonProps {
  children: ReactNode
  active?: boolean
  activeGroup?: boolean
  disabled?: boolean
  disclosure?: boolean
  disclosureOpen?: boolean
  tooltip?: string
  variant?: SidebarMenuButtonVariant
  size?: SidebarMenuButtonSize
  onClick?: () => void
}

export interface SidebarMenuActionProps {
  children: ReactNode
  ariaLabel: string
  disabled?: boolean
  showOnHover?: boolean
  onClick?: () => void
}

export interface SidebarMenuBadgeProps {
  children: ReactNode
}

export interface SidebarMenuSkeletonProps {
  showIcon?: boolean
}

export interface SidebarMenuSubProps {
  children: ReactNode
}

export interface SidebarMenuSubItemProps {
  children: ReactNode
}

export interface SidebarMenuSubButtonProps {
  children: ReactNode
  href: string
  active?: boolean
  size?: SidebarMenuSubButtonSize
  onClick?: () => void
}

export function SidebarProvider({
  children,
  defaultOpen,
  open,
  onOpenChange,
}: SidebarProviderProps) {
  return (
    <CoreSidebarProvider
      defaultOpen={defaultOpen}
      open={open}
      onOpenChange={onOpenChange}
    >
      {children}
    </CoreSidebarProvider>
  )
}

export function Sidebar({
  children,
  side = "left",
  variant = "sidebar",
  collapsible = "offcanvas",
}: SidebarProps) {
  return (
    <CoreSidebar side={side} variant={variant} collapsible={collapsible}>
      {children}
    </CoreSidebar>
  )
}

export function SidebarTrigger({
  ariaLabel = "Toggle sidebar",
}: SidebarTriggerProps) {
  return <CoreSidebarTrigger aria-label={ariaLabel} />
}

export function SidebarRail({
  ariaLabel = "Toggle sidebar",
}: SidebarRailProps) {
  return <CoreSidebarRail aria-label={ariaLabel} />
}

export function useSidebar() {
  return useCoreSidebar()
}

export function SidebarInset({ children }: SidebarInsetProps) {
  return <CoreSidebarInset>{children}</CoreSidebarInset>
}

export function SidebarInput({
  value,
  defaultValue,
  placeholder,
  disabled = false,
  onValueChange,
}: SidebarInputProps) {
  const handleChange = (event: ChangeEvent<HTMLInputElement>) => {
    onValueChange?.(event.target.value)
  }

  return (
    <CoreSidebarInput
      value={value}
      defaultValue={defaultValue}
      placeholder={placeholder}
      disabled={disabled}
      onChange={handleChange}
    />
  )
}

export function SidebarHeader({ children }: SidebarHeaderProps) {
  return <CoreSidebarHeader>{children}</CoreSidebarHeader>
}

export function SidebarFooter({ children }: SidebarFooterProps) {
  return <CoreSidebarFooter>{children}</CoreSidebarFooter>
}

export function SidebarContent({ children }: SidebarContentProps) {
  return <CoreSidebarContent>{children}</CoreSidebarContent>
}

export function SidebarSeparator() {
  return <CoreSidebarSeparator />
}

export function SidebarGroup({ children }: SidebarGroupProps) {
  return <CoreSidebarGroup>{children}</CoreSidebarGroup>
}

export function SidebarGroupLabel({ children }: SidebarGroupLabelProps) {
  return <CoreSidebarGroupLabel>{children}</CoreSidebarGroupLabel>
}

export function SidebarGroupAction({
  children,
  ariaLabel,
  disabled = false,
  onClick,
}: SidebarGroupActionProps) {
  const handleClick = (event: MouseEvent<HTMLButtonElement>) => {
    event.preventDefault()
    onClick?.()
  }

  return (
    <CoreSidebarGroupAction
      aria-label={ariaLabel}
      disabled={disabled}
      onClick={handleClick}
    >
      {children}
    </CoreSidebarGroupAction>
  )
}

export function SidebarGroupContent({ children }: SidebarGroupContentProps) {
  return <CoreSidebarGroupContent>{children}</CoreSidebarGroupContent>
}

export function SidebarMenu({ children }: SidebarMenuProps) {
  return <CoreSidebarMenu>{children}</CoreSidebarMenu>
}

export function SidebarMenuItem({ children }: SidebarMenuItemProps) {
  return <CoreSidebarMenuItem>{children}</CoreSidebarMenuItem>
}

export function SidebarMenuButton({
  children,
  active = false,
  activeGroup = false,
  disabled = false,
  disclosure = false,
  disclosureOpen = false,
  tooltip,
  variant = "default",
  size = "default",
  onClick,
}: SidebarMenuButtonProps) {
  return (
    <CoreSidebarMenuButton
      isActive={active}
      isActiveGroup={activeGroup}
      disabled={disabled}
      disclosure={disclosure}
      disclosureOpen={disclosureOpen}
      tooltip={tooltip}
      variant={variant}
      size={size}
      onClick={onClick}
    >
      {children}
    </CoreSidebarMenuButton>
  )
}

export function SidebarMenuAction({
  children,
  ariaLabel,
  disabled = false,
  showOnHover = false,
  onClick,
}: SidebarMenuActionProps) {
  const handleClick = (event: MouseEvent<HTMLButtonElement>) => {
    event.preventDefault()
    onClick?.()
  }

  return (
    <CoreSidebarMenuAction
      aria-label={ariaLabel}
      disabled={disabled}
      showOnHover={showOnHover}
      onClick={handleClick}
    >
      {children}
    </CoreSidebarMenuAction>
  )
}

export function SidebarMenuBadge({ children }: SidebarMenuBadgeProps) {
  return <CoreSidebarMenuBadge>{children}</CoreSidebarMenuBadge>
}

export function SidebarMenuSkeleton({
  showIcon = false,
}: SidebarMenuSkeletonProps) {
  return <CoreSidebarMenuSkeleton showIcon={showIcon} />
}

export function SidebarMenuSub({ children }: SidebarMenuSubProps) {
  return <CoreSidebarMenuSub>{children}</CoreSidebarMenuSub>
}

export function SidebarMenuSubItem({ children }: SidebarMenuSubItemProps) {
  return <CoreSidebarMenuSubItem>{children}</CoreSidebarMenuSubItem>
}

export function SidebarMenuSubButton({
  children,
  href,
  active = false,
  size = "md",
  onClick,
}: SidebarMenuSubButtonProps) {
  const handleClick: MouseEventHandler<HTMLAnchorElement> = (event) => {
    if (!onClick) {
      return
    }

    event.preventDefault()
    onClick()
  }

  return (
    <CoreSidebarMenuSubButton
      href={href}
      isActive={active}
      size={size}
      onClick={handleClick}
    >
      {children}
    </CoreSidebarMenuSubButton>
  )
}
