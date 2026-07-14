import type { ReactNode } from "react"
import { ProductAnalyticsTracker } from "@/analytics/ProductAnalyticsTracker"
import { MinimalAuthSurface } from "./MinimalAuthSurface"

export interface MinimalAuthShellProps {
  children: ReactNode
  tools?: ReactNode
}

export function MinimalAuthShell({ children, tools }: MinimalAuthShellProps) {
  return (
    <MinimalAuthSurface className="min-h-screen place-items-center px-6 py-14 sm:px-6">
      {tools ? (
        <div className="absolute top-6 right-5 z-30 flex gap-2.5 sm:top-10 sm:right-12">
          {tools}
        </div>
      ) : null}

      <ProductAnalyticsTracker />
      {children}
    </MinimalAuthSurface>
  )
}
