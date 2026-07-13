import type { ReactNode } from "react"
import {
  Tabs as CoreTabs,
  TabsContent as CoreTabsContent,
  TabsList as CoreTabsList,
  TabsTrigger as CoreTabsTrigger,
} from "@workspace/ui-core/components/tabs"

export interface TabsProps {
  value: string
  onValueChange: (value: string) => void
  children: ReactNode
  className?: string
}

export interface TabsListProps {
  children: ReactNode
  className?: string
}

export interface TabsTriggerProps {
  value: string
  disabled?: boolean
  children: ReactNode
  className?: string
}

export interface TabsContentProps {
  value: string
  children: ReactNode
  className?: string
}

export function Tabs({ value, onValueChange, children, className }: TabsProps) {
  return (
    <CoreTabs value={value} onValueChange={onValueChange} className={className}>
      {children}
    </CoreTabs>
  )
}

export function TabsList({ children, className }: TabsListProps) {
  return <CoreTabsList className={className}>{children}</CoreTabsList>
}

export function TabsTrigger({
  value,
  disabled = false,
  children,
  className,
}: TabsTriggerProps) {
  return (
    <CoreTabsTrigger value={value} disabled={disabled} className={className}>
      {children}
    </CoreTabsTrigger>
  )
}

export function TabsContent({ value, children, className }: TabsContentProps) {
  return (
    <CoreTabsContent value={value} className={className}>
      {children}
    </CoreTabsContent>
  )
}
