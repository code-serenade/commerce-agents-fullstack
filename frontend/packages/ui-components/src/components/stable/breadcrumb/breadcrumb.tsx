import type { ReactNode } from "react"
import {
  Breadcrumb as CoreBreadcrumb,
  BreadcrumbItem as CoreBreadcrumbItem,
  BreadcrumbLink as CoreBreadcrumbLink,
  BreadcrumbList as CoreBreadcrumbList,
  BreadcrumbPage as CoreBreadcrumbPage,
  BreadcrumbSeparator as CoreBreadcrumbSeparator,
} from "@workspace/ui-core/components/breadcrumb"

export interface BreadcrumbEntry {
  label: ReactNode
  href?: string
}

export interface BreadcrumbProps {
  items: BreadcrumbEntry[]
}

export function Breadcrumb({ items }: BreadcrumbProps) {
  return (
    <CoreBreadcrumb>
      <CoreBreadcrumbList>
        {items.map((item, index) => {
          const isLast = index === items.length - 1

          return (
            <CoreBreadcrumbItem key={index}>
              {!isLast && item.href ? (
                <CoreBreadcrumbLink href={item.href}>{item.label}</CoreBreadcrumbLink>
              ) : (
                <CoreBreadcrumbPage>{item.label}</CoreBreadcrumbPage>
              )}
              {!isLast ? <CoreBreadcrumbSeparator /> : null}
            </CoreBreadcrumbItem>
          )
        })}
      </CoreBreadcrumbList>
    </CoreBreadcrumb>
  )
}
