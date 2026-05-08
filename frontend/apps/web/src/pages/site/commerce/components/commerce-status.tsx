import type { ReactNode } from "react"
import { Badge } from "@workspace/ui-components"

export type StatusTone = "default" | "success" | "warning" | "danger" | "muted"

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

export function toneForStatus(status?: string | null): StatusTone {
  if (!status) return "muted"
  if (
    [
      "active",
      "enabled",
      "configured",
      "success",
      "succeeded",
      "completed",
      "done",
      "approved",
      "executed",
      "acted",
      "ready",
    ].includes(status)
  ) {
    return "success"
  }
  if (
    [
      "pending",
      "draft",
      "planned",
      "running",
      "syncing",
      "in_progress",
      "snoozed",
    ].includes(status)
  ) {
    return "warning"
  }
  if (
    [
      "failed",
      "cancelled",
      "rejected",
      "critical",
      "dismissed",
      "archived",
      "disabled",
      "deprecated",
      "blocked",
    ].includes(status)
  ) {
    return "danger"
  }

  return "default"
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
