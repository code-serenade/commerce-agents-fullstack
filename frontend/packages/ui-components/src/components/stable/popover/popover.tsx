import type { ReactNode } from "react"
import {
  Popover as CorePopover,
  PopoverAnchor as CorePopoverAnchor,
  PopoverContent as CorePopoverContent,
  PopoverDescription as CorePopoverDescription,
  PopoverHeader as CorePopoverHeader,
  PopoverTitle as CorePopoverTitle,
  PopoverTrigger as CorePopoverTrigger,
} from "@workspace/ui-core/components/popover"

export interface PopoverProps {
  open: boolean
  onOpenChange: (open: boolean) => void
  children: ReactNode
}

export interface PopoverTriggerProps {
  children: ReactNode
}

export interface PopoverAnchorProps {
  children: ReactNode
}

export interface PopoverContentProps {
  children: ReactNode
}

export interface PopoverHeaderProps {
  children: ReactNode
}

export interface PopoverTitleProps {
  children: ReactNode
}

export interface PopoverDescriptionProps {
  children: ReactNode
}

export function Popover({ open, onOpenChange, children }: PopoverProps) {
  return (
    <CorePopover open={open} onOpenChange={onOpenChange}>
      {children}
    </CorePopover>
  )
}

export function PopoverTrigger({ children }: PopoverTriggerProps) {
  return <CorePopoverTrigger>{children}</CorePopoverTrigger>
}

export function PopoverAnchor({ children }: PopoverAnchorProps) {
  return <CorePopoverAnchor>{children}</CorePopoverAnchor>
}

export function PopoverContent({ children }: PopoverContentProps) {
  return <CorePopoverContent>{children}</CorePopoverContent>
}

export function PopoverHeader({ children }: PopoverHeaderProps) {
  return <CorePopoverHeader>{children}</CorePopoverHeader>
}

export function PopoverTitle({ children }: PopoverTitleProps) {
  return <CorePopoverTitle>{children}</CorePopoverTitle>
}

export function PopoverDescription({ children }: PopoverDescriptionProps) {
  return <CorePopoverDescription>{children}</CorePopoverDescription>
}
