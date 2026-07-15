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
export type SheetSize = "default" | "wide"

export interface SheetProps {
  open: boolean
  onOpenChange: (open: boolean) => void
  triggerLabel?: ReactNode
  title: ReactNode
  description?: ReactNode
  side?: SheetSide
  size?: SheetSize
  children?: ReactNode
}

export function Sheet({
  open,
  onOpenChange,
  triggerLabel,
  title,
  description,
  side = "right",
  size = "default",
  children,
}: SheetProps) {
  return (
    <CoreSheet open={open} onOpenChange={onOpenChange}>
      {triggerLabel ? <CoreSheetTrigger>{triggerLabel}</CoreSheetTrigger> : null}
      <CoreSheetContent
        side={side}
        className={
          size === "wide"
            ? "data-[side=left]:w-full data-[side=left]:sm:max-w-[min(46rem,calc(100vw-4rem))] data-[side=right]:w-full data-[side=right]:sm:max-w-[min(46rem,calc(100vw-4rem))]"
            : undefined
        }
      >
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
