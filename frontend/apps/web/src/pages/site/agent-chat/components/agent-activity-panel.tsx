import { ChevronDown, CircleCheck, CircleDot, CircleX, Loader2 } from "lucide-react"
import { useState } from "react"
import { useTranslation } from "react-i18next"
import { Badge } from "@workspace/ui-components/stable/badge"
import { Collapsible } from "@workspace/ui-components/stable/collapsible"
import type { AgentActivityItem } from "../types"

export interface AgentActivityPanelProps {
  activities?: AgentActivityItem[]
}

export function AgentActivityPanel({ activities = [] }: AgentActivityPanelProps) {
  const { t } = useTranslation()
  const [open, setOpen] = useState(false)

  if (activities.length === 0) return null

  return (
    <div className="mb-3 rounded-md border border-(--app-border) bg-(--app-muted-surface) px-3 py-2">
      <Collapsible
        open={open}
        onOpenChange={setOpen}
        triggerLabel={
          <span className="flex items-center gap-2 rounded-md px-2 py-1 text-xs font-medium text-(--app-text) hover:bg-(--app-hover-bg)">
            <ChevronDown
              className={`size-4 transition-transform ${open ? "rotate-180" : ""}`}
            />
            <span>{t("agentChat.activity.title")}</span>
            <Badge variant="outline">{activities.length}</Badge>
          </span>
        }
      >
        <ol className="mt-2 space-y-2 border-t border-(--app-border) pt-2">
          {activities.map((activity) => (
            <li key={activity.id} className="grid grid-cols-[1rem_1fr] gap-2 text-xs">
              <ActivityStatusIcon status={activity.status} />
              <div className="min-w-0">
                <div className="flex flex-wrap items-center gap-2">
                  <span className="font-medium text-(--app-text)">
                    {t(activity.title)}
                  </span>
                  <Badge variant="secondary">
                    {t(`agentChat.activity.kinds.${activity.kind}`)}
                  </Badge>
                </div>
                <ActivityMeta activity={activity} />
                {activity.description ? (
                  <p className="mt-1 max-h-16 overflow-hidden whitespace-pre-wrap text-(--app-muted-text)">
                    {activity.description}
                  </p>
                ) : null}
              </div>
            </li>
          ))}
        </ol>
      </Collapsible>
    </div>
  )
}

function ActivityStatusIcon({
  status = "info",
}: {
  status?: AgentActivityItem["status"]
}) {
  if (status === "running") {
    return <Loader2 className="mt-1 size-4 animate-spin text-(--app-muted-text)" />
  }
  if (status === "success") {
    return <CircleCheck className="mt-1 size-4 text-emerald-600" />
  }
  if (status === "error") {
    return <CircleX className="mt-1 size-4 text-destructive" />
  }
  return <CircleDot className="mt-1 size-4 text-(--app-muted-text)" />
}

function ActivityMeta({ activity }: { activity: AgentActivityItem }) {
  const meta = [
    activity.workerId ? `worker ${activity.workerId}` : "",
    activity.taskId ? `task ${activity.taskId}` : "",
    activity.nodeId ? `node ${activity.nodeId}` : "",
    activity.graphId ? `graph ${activity.graphId}` : "",
  ].filter(Boolean)

  if (meta.length === 0) return null

  return (
    <p className="mt-1 truncate font-mono text-[11px] text-(--app-muted-text)">
      {meta.join(" / ")}
    </p>
  )
}
