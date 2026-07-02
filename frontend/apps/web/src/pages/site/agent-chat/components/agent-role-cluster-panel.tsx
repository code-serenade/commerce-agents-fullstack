import { UsersRound } from "lucide-react"
import { useTranslation } from "react-i18next"
import {
  getAgentRoleActivities,
  getAgentRoleSummary,
} from "../view-model/agent-role-cluster-view-model"
import type { InlineAgent } from "../view-model/thread-inline-view-model"
import { AgentRoleActivityItem } from "./agent-role-activity-item"
import type { AgentRoleClusterVariant } from "./agent-role-cluster-styles"

export function AgentRoleClusterPanel({
  agents,
  animatePortraits = false,
  ariaLabel,
  eyebrow,
  footer,
  itemDetail = "action",
  summary,
  title,
  variant = "timeline",
}: {
  agents: InlineAgent[]
  animatePortraits?: boolean
  ariaLabel?: string
  eyebrow?: string
  footer?: string
  itemDetail?: "action" | "summary"
  summary?: string
  title?: string
  variant?: AgentRoleClusterVariant
}) {
  const { t } = useTranslation()
  const activities = getAgentRoleActivities(agents, t)

  if (activities.length === 0) return null
  const isShowcase = variant === "showcase"

  return (
    <section
      className={
        isShowcase
          ? "relative min-w-0 max-w-full overflow-hidden rounded-lg border border-border bg-muted/30 p-3 shadow-xs"
          : "min-w-0 max-w-full overflow-hidden rounded-md border border-border bg-background px-3 py-2"
      }
      aria-label={ariaLabel ?? t("agentChat.thread.roleClusterActivity.ariaLabel")}
      data-agent-role-cluster="true"
    >
      <div
        className={
          isShowcase
            ? "flex min-w-0 flex-wrap items-end justify-between gap-2"
            : "flex min-w-0 items-center gap-2 text-xs"
        }
      >
        <div className="flex min-w-0 items-center gap-2">
          <UsersRound className="size-4 shrink-0 text-primary" />
          <div className="min-w-0">
            {eyebrow ? (
              <p className="text-[10px] font-semibold uppercase text-muted-foreground">
                {eyebrow}
              </p>
            ) : null}
            <p className="truncate text-xs font-medium text-foreground">
              {title ?? t("agentChat.thread.roleClusterActivity.title")}
            </p>
          </div>
        </div>
        <span
          className={
            isShowcase
              ? "min-w-0 text-xs text-muted-foreground"
              : "min-w-0 truncate text-muted-foreground"
          }
        >
          {summary ?? getAgentRoleSummary(activities, t)}
        </span>
      </div>
      <div
        className={
          isShowcase
            ? "mt-3 max-w-full overflow-x-auto pb-1 [scrollbar-gutter:stable]"
            : "mt-2 max-w-full overflow-x-auto pb-1 [scrollbar-gutter:stable]"
        }
      >
        <div
          className={
            isShowcase
              ? "grid min-w-216 grid-cols-5 gap-2 sm:min-w-0 sm:grid-cols-3 lg:grid-cols-5"
              : "flex w-max min-w-full items-stretch gap-2"
          }
        >
          {activities.map((activity, index) => (
            <AgentRoleActivityItem
              key={activity.agent.nodeId}
              animatePortraits={animatePortraits}
              activity={activity}
              index={index}
              itemDetail={itemDetail}
              variant={variant}
            />
          ))}
        </div>
      </div>
      {footer ? (
        <p className="mt-2 text-[11px]/5 text-muted-foreground">{footer}</p>
      ) : null}
    </section>
  )
}
