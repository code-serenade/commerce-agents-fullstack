import type { InlineAgentState } from "../view-model/thread-inline-view-model"

export function getAgentRoleStatusTagClassName(state: InlineAgentState) {
  if (state === "done" || state === "boundary") {
    return "bg-emerald-100/90 text-emerald-700 dark:bg-emerald-500/18 dark:text-emerald-300"
  }
  if (state === "failed") {
    return "bg-destructive/12 text-destructive dark:bg-destructive/18 dark:text-red-300"
  }
  if (state === "working") {
    return "bg-primary/12 text-primary dark:bg-primary/18 dark:text-primary"
  }
  if (state === "reworking") {
    return "bg-violet-100/90 text-violet-700 dark:bg-violet-500/18 dark:text-violet-300"
  }
  if (state === "queued") {
    return "bg-sky-100/90 text-sky-700 dark:bg-sky-500/18 dark:text-sky-300"
  }
  return "bg-muted/80 text-muted-foreground dark:bg-muted/30 dark:text-muted-foreground"
}
