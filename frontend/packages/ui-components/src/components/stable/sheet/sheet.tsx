import type { ReactNode } from "react"
import {
  Sheet as CoreSheet,
  SheetContent as CoreSheetContent,
  SheetDescription as CoreSheetDescription,
  SheetHeader as CoreSheetHeader,
  SheetTitle as CoreSheetTitle,
  SheetTrigger as CoreSheetTrigger,
} from "@workspace/ui-core/components/sheet"

export type SheetSide = "top" | "right" | "bottom" | "left"

export interface SheetProps {
  open: boolean
  onOpenChange: (open: boolean) => void
  triggerLabel: ReactNode
  title: ReactNode
  description?: ReactNode
  side?: SheetSide
  children?: ReactNode
}

export function Sheet({
  open,
  onOpenChange,
  triggerLabel,
  title,
  description,
  side = "right",
  children,
}: SheetProps) {
  return (
    <CoreSheet open={open} onOpenChange={onOpenChange}>
      <CoreSheetTrigger>{triggerLabel}</CoreSheetTrigger>
      <CoreSheetContent side={side}>
        <CoreSheetHeader>
          <CoreSheetTitle>{title}</CoreSheetTitle>
          {description ? (
            <CoreSheetDescription>{description}</CoreSheetDescription>
          ) : null}
        </CoreSheetHeader>
        {children}
      </CoreSheetContent>
    </CoreSheet>
  )
}
