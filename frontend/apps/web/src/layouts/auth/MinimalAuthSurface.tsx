import type { ReactNode } from "react"

export interface MinimalAuthSurfaceProps {
  children: ReactNode
  className: string
}

export function MinimalAuthSurface({
  children,
  className,
}: MinimalAuthSurfaceProps) {
  return (
    <div
      className={`login-auth-bg relative grid overflow-hidden bg-[radial-gradient(ellipse_58%_74%_at_100%_48%,oklch(0.885_0.062_262/0.5)_0%,oklch(0.93_0.034_258/0.3)_42%,transparent_76%),radial-gradient(ellipse_52%_80%_at_0%_62%,oklch(0.89_0.062_222/0.42)_0%,oklch(0.94_0.03_232/0.24)_46%,transparent_78%),linear-gradient(90deg,oklch(0.945_0.028_231/0.78)_0%,oklch(0.985_0.006_250/0.96)_32%,oklch(0.99_0.004_250/0.96)_58%,oklch(0.93_0.04_260/0.62)_100%)] dark:bg-[radial-gradient(circle_at_59%_16%,rgba(139,61,255,0.3),transparent_28%),radial-gradient(circle_at_34%_54%,rgba(49,93,255,0.24),transparent_34%),radial-gradient(circle_at_84%_80%,rgba(0,205,255,0.08),transparent_30%),linear-gradient(125deg,#060a16_0%,#0b1224_42%,#111735_72%,#120c25_100%)] ${className}`}
    >
      <div className="login-auth-flow pointer-events-none absolute inset-0 bg-[radial-gradient(circle_at_18%_34%,oklch(0.88_0.06_222/0.24)_0%,transparent_30%),radial-gradient(circle_at_84%_42%,oklch(0.88_0.064_262/0.22)_0%,transparent_32%),linear-gradient(115deg,transparent_18%,oklch(1_0_0/0.24)_46%,transparent_68%)] mix-blend-normal dark:bg-[radial-gradient(circle_at_20%_32%,rgba(139,61,255,0.22),transparent_24%),radial-gradient(circle_at_78%_36%,rgba(49,93,255,0.18),transparent_26%),linear-gradient(115deg,transparent_12%,rgba(118,162,255,0.12)_42%,transparent_62%)] dark:mix-blend-soft-light" />
      <div className="login-auth-grid pointer-events-none absolute inset-0 hidden bg-[linear-gradient(rgba(126,154,255,0.11)_1px,transparent_1px),linear-gradient(90deg,rgba(166,119,255,0.105)_1px,transparent_1px)] mask-[radial-gradient(circle_at_50%_40%,#000_0%,rgba(0,0,0,0.72)_34%,rgba(0,0,0,0.28)_58%,transparent_78%)] bg-size-[56px_56px] dark:block" />
      <div className="login-auth-vignette pointer-events-none absolute inset-0 hidden bg-[radial-gradient(circle_at_50%_38%,transparent_0%,rgba(6,10,22,0.1)_34%,rgba(6,10,22,0.54)_78%),linear-gradient(90deg,rgba(6,10,22,0.36),transparent_34%,transparent_66%,rgba(6,10,22,0.42))] dark:block" />

      {children}
    </div>
  )
}
