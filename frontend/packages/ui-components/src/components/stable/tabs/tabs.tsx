import type { ReactNode } from "react"
import {
  Tabs as CoreTabs,
  TabsContent as CoreTabsContent,
  TabsList as CoreTabsList,
  TabsTrigger as CoreTabsTrigger,
} from "@workspace/ui-core/components/tabs"

interface TabsPropsBase {
  value: string
  onValueChange: (value: string) => void
  children: ReactNode
  className?: string
}

export type TabsProps = TabsPropsBase

interface TabsListPropsBase {
  children: ReactNode
  className?: string
}

export type TabsListProps = TabsListPropsBase

interface TabsTriggerPropsBase {
  value: string
  disabled?: boolean
  children: ReactNode
  className?: string
}

export type TabsTriggerProps = TabsTriggerPropsBase

interface TabsContentPropsBase {
  value: string
  children: ReactNode
  className?: string
}

export type TabsContentProps = TabsContentPropsBase

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
