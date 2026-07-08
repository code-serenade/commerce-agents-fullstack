import type { ReactNode } from "react"
import { ProductAnalyticsTracker } from "@/analytics/ProductAnalyticsTracker"

export interface MinimalAuthShellProps {
  children: ReactNode
  tools?: ReactNode
}

export function MinimalAuthShell({ children, tools }: MinimalAuthShellProps) {
  return (
    <div className="login-auth-bg relative grid min-h-screen place-items-center overflow-hidden bg-[radial-gradient(circle_at_60%_12%,rgba(120,70,255,0.14),transparent_30%),radial-gradient(circle_at_34%_50%,rgba(67,151,255,0.16),transparent_34%),linear-gradient(115deg,#fbfdff_0%,#f7fbff_38%,#eef5ff_72%,#f7f4ff_100%)] px-6 py-14 sm:px-6 dark:bg-[radial-gradient(circle_at_59%_16%,rgba(139,61,255,0.3),transparent_28%),radial-gradient(circle_at_34%_54%,rgba(49,93,255,0.24),transparent_34%),radial-gradient(circle_at_84%_80%,rgba(0,205,255,0.08),transparent_30%),linear-gradient(125deg,#060a16_0%,#0b1224_42%,#111735_72%,#120c25_100%)]">
      <div className="login-auth-flow pointer-events-none absolute inset-0 bg-[radial-gradient(circle_at_20%_32%,rgba(120,70,255,0.16),transparent_24%),radial-gradient(circle_at_78%_36%,rgba(67,151,255,0.14),transparent_26%),linear-gradient(115deg,transparent_12%,rgba(255,255,255,0.26)_42%,transparent_62%)] mix-blend-soft-light dark:bg-[radial-gradient(circle_at_20%_32%,rgba(139,61,255,0.22),transparent_24%),radial-gradient(circle_at_78%_36%,rgba(49,93,255,0.18),transparent_26%),linear-gradient(115deg,transparent_12%,rgba(118,162,255,0.12)_42%,transparent_62%)]" />
      <div className="login-auth-grid pointer-events-none absolute inset-0 hidden bg-[linear-gradient(rgba(126,154,255,0.11)_1px,transparent_1px),linear-gradient(90deg,rgba(166,119,255,0.105)_1px,transparent_1px)] [mask-image:radial-gradient(circle_at_50%_40%,#000_0%,rgba(0,0,0,0.72)_34%,rgba(0,0,0,0.28)_58%,transparent_78%)] bg-[length:56px_56px] dark:block" />
      <div className="login-auth-vignette pointer-events-none absolute inset-0 hidden bg-[radial-gradient(circle_at_50%_38%,transparent_0%,rgba(6,10,22,0.1)_34%,rgba(6,10,22,0.54)_78%),linear-gradient(90deg,rgba(6,10,22,0.36),transparent_34%,transparent_66%,rgba(6,10,22,0.42))] dark:block" />

      {tools ? (
        <div className="absolute top-6 right-5 z-30 flex gap-2.5 sm:top-10 sm:right-12">
          {tools}
        </div>
      ) : null}

      <ProductAnalyticsTracker />
      {children}
    </div>
  )
}
