import type { CSSProperties, ReactNode } from "react"
import {
  Dialog as CoreDialog,
  DialogClose as CoreDialogClose,
  DialogContent as CoreDialogContent,
  DialogDescription as CoreDialogDescription,
  DialogFooter as CoreDialogFooter,
  DialogHeader as CoreDialogHeader,
  DialogOverlay as CoreDialogOverlay,
  DialogPortal as CoreDialogPortal,
  DialogTitle as CoreDialogTitle,
  DialogTrigger as CoreDialogTrigger,
} from "@workspace/ui-core/components/dialog"

export interface DialogProps {
  open: boolean
  onOpenChange: (open: boolean) => void
  children: ReactNode
}

export interface DialogTriggerProps {
  children: ReactNode
}

export interface DialogPortalProps {
  children: ReactNode
}

export interface DialogOverlayProps {}

interface DialogContentPropsBase {
  children: ReactNode
  className?: string
  showCloseButton?: boolean
  style?: CSSProperties
}

export type DialogContentProps = DialogContentPropsBase

interface DialogHeaderPropsBase {
  children: ReactNode
  className?: string
}

export type DialogHeaderProps = DialogHeaderPropsBase

interface DialogFooterPropsBase {
  children: ReactNode
  className?: string
}

export type DialogFooterProps = DialogFooterPropsBase

export interface DialogTitleProps {
  children: ReactNode
}

export interface DialogDescriptionProps {
  children: ReactNode
}

interface DialogClosePropsBase {
  children: ReactNode
  asChild?: boolean
  className?: string
}

export type DialogCloseProps = DialogClosePropsBase

export function Dialog({ open, onOpenChange, children }: DialogProps) {
  return (
    <CoreDialog open={open} onOpenChange={onOpenChange}>
      {children}
    </CoreDialog>
  )
}

export function DialogTrigger({ children }: DialogTriggerProps) {
  return <CoreDialogTrigger>{children}</CoreDialogTrigger>
}

export function DialogPortal({ children }: DialogPortalProps) {
  return <CoreDialogPortal>{children}</CoreDialogPortal>
}

export function DialogOverlay(_: DialogOverlayProps) {
  return <CoreDialogOverlay />
}

export function DialogContent({
  children,
  className,
  showCloseButton,
  style,
}: DialogContentProps) {
  return (
    <CoreDialogContent
      className={className}
      showCloseButton={showCloseButton}
      style={style}
    >
      {children}
    </CoreDialogContent>
  )
}

export function DialogHeader({ children, className }: DialogHeaderProps) {
  return <CoreDialogHeader className={className}>{children}</CoreDialogHeader>
}

export function DialogFooter({ children, className }: DialogFooterProps) {
  return <CoreDialogFooter className={className}>{children}</CoreDialogFooter>
}

export function DialogTitle({ children }: DialogTitleProps) {
  return <CoreDialogTitle>{children}</CoreDialogTitle>
}

export function DialogDescription({ children }: DialogDescriptionProps) {
  return <CoreDialogDescription>{children}</CoreDialogDescription>
}

export function DialogClose({ asChild, children, className }: DialogCloseProps) {
  return (
    <CoreDialogClose asChild={asChild} className={className}>
      {children}
    </CoreDialogClose>
  )
}
