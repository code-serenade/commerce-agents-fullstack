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
}

export interface TabsListProps {
  children: ReactNode
}

export interface TabsTriggerProps {
  value: string
  disabled?: boolean
  children: ReactNode
}

export interface TabsContentProps {
  value: string
  children: ReactNode
}

export function Tabs({ value, onValueChange, children }: TabsProps) {
  return (
    <CoreTabs value={value} onValueChange={onValueChange}>
      {children}
    </CoreTabs>
  )
}

export function TabsList({ children }: TabsListProps) {
  return <CoreTabsList>{children}</CoreTabsList>
}

export function TabsTrigger({
  value,
  disabled = false,
  children,
}: TabsTriggerProps) {
  return (
    <CoreTabsTrigger value={value} disabled={disabled}>
      {children}
    </CoreTabsTrigger>
  )
}

export function TabsContent({ value, children }: TabsContentProps) {
  return <CoreTabsContent value={value}>{children}</CoreTabsContent>
}
