import type { ReactNode } from "react"
import {
  Drawer as CoreDrawer,
  DrawerClose as CoreDrawerClose,
  DrawerContent as CoreDrawerContent,
  DrawerDescription as CoreDrawerDescription,
  DrawerFooter as CoreDrawerFooter,
  DrawerHeader as CoreDrawerHeader,
  DrawerTitle as CoreDrawerTitle,
  DrawerTrigger as CoreDrawerTrigger,
} from "@workspace/ui-core/components/drawer"

export interface DrawerProps {
  open: boolean
  onOpenChange: (open: boolean) => void
  triggerLabel: ReactNode
  title: ReactNode
  description?: ReactNode
  closeLabel?: ReactNode
  children?: ReactNode
}

export function Drawer({
  open,
  onOpenChange,
  triggerLabel,
  title,
  description,
  closeLabel = "Close",
  children,
}: DrawerProps) {
  return (
    <CoreDrawer open={open} onOpenChange={onOpenChange}>
      <CoreDrawerTrigger>{triggerLabel}</CoreDrawerTrigger>
      <CoreDrawerContent>
        <CoreDrawerHeader>
          <CoreDrawerTitle>{title}</CoreDrawerTitle>
          {description ? (
            <CoreDrawerDescription>{description}</CoreDrawerDescription>
          ) : null}
        </CoreDrawerHeader>
        {children}
        <CoreDrawerFooter>
          <CoreDrawerClose>{closeLabel}</CoreDrawerClose>
        </CoreDrawerFooter>
      </CoreDrawerContent>
    </CoreDrawer>
  )
}
