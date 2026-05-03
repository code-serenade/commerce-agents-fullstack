import type { ReactNode } from "react"
import {
  DropdownMenu as CoreDropdownMenu,
  DropdownMenuCheckboxItem as CoreDropdownMenuCheckboxItem,
  DropdownMenuContent as CoreDropdownMenuContent,
  DropdownMenuGroup as CoreDropdownMenuGroup,
  DropdownMenuItem as CoreDropdownMenuItem,
  DropdownMenuLabel as CoreDropdownMenuLabel,
  DropdownMenuPortal as CoreDropdownMenuPortal,
  DropdownMenuRadioGroup as CoreDropdownMenuRadioGroup,
  DropdownMenuRadioItem as CoreDropdownMenuRadioItem,
  DropdownMenuSeparator as CoreDropdownMenuSeparator,
  DropdownMenuShortcut as CoreDropdownMenuShortcut,
  DropdownMenuSub as CoreDropdownMenuSub,
  DropdownMenuSubContent as CoreDropdownMenuSubContent,
  DropdownMenuSubTrigger as CoreDropdownMenuSubTrigger,
  DropdownMenuTrigger as CoreDropdownMenuTrigger,
} from "@workspace/ui-core/components/dropdown-menu"

export type DropdownMenuItemVariant = "default" | "destructive"

export interface DropdownMenuProps {
  open?: boolean
  onOpenChange?: (open: boolean) => void
  children: ReactNode
}

export interface DropdownMenuTriggerProps {
  children: ReactNode
}

export interface DropdownMenuPortalProps {
  children: ReactNode
}

export interface DropdownMenuContentProps {
  children: ReactNode
}

export interface DropdownMenuGroupProps {
  children: ReactNode
}

export interface DropdownMenuLabelProps {
  children: ReactNode
  inset?: boolean
}

export interface DropdownMenuItemProps {
  children: ReactNode
  disabled?: boolean
  inset?: boolean
  variant?: DropdownMenuItemVariant
  onSelect?: () => void
}

export interface DropdownMenuCheckboxItemProps {
  children: ReactNode
  checked: boolean
  disabled?: boolean
  inset?: boolean
  onCheckedChange?: (checked: boolean) => void
}

export interface DropdownMenuRadioGroupProps {
  value: string
  onValueChange: (value: string) => void
  children: ReactNode
}

export interface DropdownMenuRadioItemProps {
  children: ReactNode
  value: string
  disabled?: boolean
  inset?: boolean
}

export interface DropdownMenuSeparatorProps {}

export interface DropdownMenuShortcutProps {
  children: ReactNode
}

export interface DropdownMenuSubProps {
  children: ReactNode
  open?: boolean
  onOpenChange?: (open: boolean) => void
}

export interface DropdownMenuSubTriggerProps {
  children: ReactNode
  inset?: boolean
}

export interface DropdownMenuSubContentProps {
  children: ReactNode
}

export function DropdownMenu({ open, onOpenChange, children }: DropdownMenuProps) {
  return (
    <CoreDropdownMenu open={open} onOpenChange={onOpenChange}>
      {children}
    </CoreDropdownMenu>
  )
}

export function DropdownMenuTrigger({ children }: DropdownMenuTriggerProps) {
  return <CoreDropdownMenuTrigger>{children}</CoreDropdownMenuTrigger>
}

export function DropdownMenuPortal({ children }: DropdownMenuPortalProps) {
  return <CoreDropdownMenuPortal>{children}</CoreDropdownMenuPortal>
}

export function DropdownMenuContent({ children }: DropdownMenuContentProps) {
  return <CoreDropdownMenuContent>{children}</CoreDropdownMenuContent>
}

export function DropdownMenuGroup({ children }: DropdownMenuGroupProps) {
  return <CoreDropdownMenuGroup>{children}</CoreDropdownMenuGroup>
}

export function DropdownMenuLabel({
  children,
  inset = false,
}: DropdownMenuLabelProps) {
  return <CoreDropdownMenuLabel inset={inset}>{children}</CoreDropdownMenuLabel>
}

export function DropdownMenuItem({
  children,
  disabled = false,
  inset = false,
  variant = "default",
  onSelect,
}: DropdownMenuItemProps) {
  return (
    <CoreDropdownMenuItem
      disabled={disabled}
      inset={inset}
      variant={variant}
      onSelect={onSelect}
    >
      {children}
    </CoreDropdownMenuItem>
  )
}

export function DropdownMenuCheckboxItem({
  children,
  checked,
  disabled = false,
  inset = false,
  onCheckedChange,
}: DropdownMenuCheckboxItemProps) {
  return (
    <CoreDropdownMenuCheckboxItem
      checked={checked}
      disabled={disabled}
      inset={inset}
      onCheckedChange={
        onCheckedChange ? (value) => onCheckedChange(value === true) : undefined
      }
    >
      {children}
    </CoreDropdownMenuCheckboxItem>
  )
}

export function DropdownMenuRadioGroup({
  value,
  onValueChange,
  children,
}: DropdownMenuRadioGroupProps) {
  return (
    <CoreDropdownMenuRadioGroup value={value} onValueChange={onValueChange}>
      {children}
    </CoreDropdownMenuRadioGroup>
  )
}

export function DropdownMenuRadioItem({
  children,
  value,
  disabled = false,
  inset = false,
}: DropdownMenuRadioItemProps) {
  return (
    <CoreDropdownMenuRadioItem value={value} disabled={disabled} inset={inset}>
      {children}
    </CoreDropdownMenuRadioItem>
  )
}

export function DropdownMenuSeparator(_: DropdownMenuSeparatorProps) {
  return <CoreDropdownMenuSeparator />
}

export function DropdownMenuShortcut({ children }: DropdownMenuShortcutProps) {
  return <CoreDropdownMenuShortcut>{children}</CoreDropdownMenuShortcut>
}

export function DropdownMenuSub({
  children,
  open,
  onOpenChange,
}: DropdownMenuSubProps) {
  return (
    <CoreDropdownMenuSub open={open} onOpenChange={onOpenChange}>
      {children}
    </CoreDropdownMenuSub>
  )
}

export function DropdownMenuSubTrigger({
  children,
  inset = false,
}: DropdownMenuSubTriggerProps) {
  return <CoreDropdownMenuSubTrigger inset={inset}>{children}</CoreDropdownMenuSubTrigger>
}

export function DropdownMenuSubContent({ children }: DropdownMenuSubContentProps) {
  return <CoreDropdownMenuSubContent>{children}</CoreDropdownMenuSubContent>
}
