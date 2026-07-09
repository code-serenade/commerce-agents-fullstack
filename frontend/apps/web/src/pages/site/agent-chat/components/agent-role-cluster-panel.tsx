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
  surface = "card",
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
  surface?: "card" | "plain"
  title?: string
  variant?: AgentRoleClusterVariant
}) {
  const { t } = useTranslation()
  const activities = getAgentRoleActivities(agents, t)

  if (activities.length === 0) return null
  const isShowcase = variant === "showcase"
  const isPlain = surface === "plain"

  return (
    <section
      className={
        isShowcase
          ? isPlain
            ? "relative max-w-full min-w-0 overflow-hidden"
            : "relative max-w-full min-w-0 overflow-hidden rounded-lg border border-border bg-muted/30 p-3 shadow-xs"
          : "max-w-full min-w-0 overflow-hidden rounded-xl border border-(--agent-chat-soft-border) bg-(--agent-chat-subtle-surface) px-3 py-2"
      }
      aria-label={
        ariaLabel ?? t("agentChat.thread.roleClusterActivity.ariaLabel")
      }
      data-agent-role-cluster="true"
    >
      <div
        className={
          isShowcase && isPlain
            ? "mb-3 flex min-w-0 flex-wrap items-start justify-between gap-2"
            : isShowcase
              ? "flex min-w-0 flex-wrap items-end justify-between gap-2"
              : "flex min-w-0 items-center gap-2 text-xs"
        }
      >
        <div
          className={`flex min-w-0 ${
            isShowcase && isPlain ? "items-start" : "items-center gap-2"
          }`}
        >
          <div className="min-w-0">
            {eyebrow ? (
              <p
                className={
                  isShowcase && isPlain
                    ? "text-base font-bold text-foreground"
                    : "text-[10px] font-semibold text-muted-foreground uppercase"
                }
              >
                {eyebrow}
              </p>
            ) : null}
            <p
              className={
                isShowcase && isPlain
                  ? "mt-0.5 truncate text-xs text-muted-foreground"
                  : "truncate text-xs font-medium text-foreground"
              }
            >
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
            ? isPlain
              ? "max-w-full overflow-x-auto pb-1 [scrollbar-gutter:stable]"
              : "mt-3 max-w-full overflow-x-auto pb-1 [scrollbar-gutter:stable]"
            : "mt-2 max-w-full"
        }
      >
        <div
          className={
            isShowcase
              ? "grid min-w-216 grid-cols-5 gap-2 sm:min-w-0 sm:grid-cols-3 lg:grid-cols-5"
              : "grid min-w-0 gap-1.5"
          }
        >
          {activities.map((activity, index) => (
            <AgentRoleActivityItem
              key={activity.agent.nodeId}
              animatePortraits={animatePortraits}
              activity={activity}
              index={index}
              itemDetail={itemDetail}
              showStatus={!isPlain}
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
