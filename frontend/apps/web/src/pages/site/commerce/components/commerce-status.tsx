import type { ReactNode } from "react"
import { Badge } from "@workspace/ui-components"
import type { StatusTone } from "./commerce-status-utils"

export type { StatusTone } from "./commerce-status-utils"

export function StatusBadge({
  children,
  tone = "default",
}: {
  children: ReactNode
  tone?: StatusTone
}) {
  const variant =
    tone === "danger"
      ? "destructive"
      : tone === "muted"
        ? "secondary"
        : tone === "default"
          ? "outline"
          : "default"

  return <Badge variant={variant}>{children}</Badge>
}

export function MutedText({ children }: { children: ReactNode }) {
  return <span className="text-sm text-muted-foreground">{children}</span>
}

export function IdText({ children }: { children: ReactNode }) {
  return (
    <span className="block max-w-44 truncate font-mono text-xs text-muted-foreground">
      {children}
    </span>
  )
}
