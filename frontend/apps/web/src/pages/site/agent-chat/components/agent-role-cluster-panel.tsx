import type { ReactNode } from "react"
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
  headerAction,
  itemDetail = "action",
  isCollapsed = false,
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
  headerAction?: ReactNode
  itemDetail?: "action" | "summary"
  isCollapsed?: boolean
  summary?: string
  surface?: "card" | "plain"
  title?: string
  variant?: AgentRoleClusterVariant
}) {
  const { t } = useTranslation()
  const activities = getAgentRoleActivities(agents, t)

  if (activities.length === 0) return null
  const isShowcase = variant === "showcase"
  const isCompact = variant === "compact"
  const isPlain = surface === "plain"

  return (
    <section
      className={
        isShowcase
          ? isPlain
            ? "relative mx-auto max-w-6xl min-w-0 overflow-hidden"
            : "relative max-w-full min-w-0 overflow-hidden rounded-lg border border-border bg-muted/30 p-3 shadow-xs"
          : isCompact
            ? "max-w-full min-w-0 overflow-hidden bg-transparent px-3 py-2"
            : "max-w-full min-w-0 overflow-hidden rounded-xl border border-(--agent-chat-soft-border) bg-(--agent-chat-elevated-strong)/70 px-3 py-2 shadow-sm"
      }
      aria-label={
        ariaLabel ?? t("agentChat.thread.roleClusterActivity.ariaLabel")
      }
      data-agent-role-cluster="true"
    >
      <div
        className={
          isShowcase && isPlain
            ? "mb-4 flex min-w-0 flex-wrap items-center justify-between gap-x-4 gap-y-2"
            : isShowcase
              ? "flex min-w-0 flex-wrap items-end justify-between gap-2"
              : isCompact
                ? "flex min-w-0 items-center justify-between gap-3 text-xs"
                : "flex min-w-0 items-center gap-2 text-xs"
        }
      >
        <div
          className={`flex min-w-0 ${
            isShowcase && isPlain ? "items-center gap-2" : "items-center gap-2"
          }`}
        >
          <div
            className={
              isShowcase && isPlain
                ? "flex min-w-0 flex-wrap items-center gap-x-2 gap-y-1"
                : "min-w-0"
            }
          >
            {eyebrow ? (
              <p
                className={
                  isShowcase && isPlain
                    ? "shrink-0 text-base font-bold text-foreground"
                    : "text-[10px] font-semibold text-muted-foreground uppercase"
                }
              >
                {eyebrow}
              </p>
            ) : null}
            {isShowcase && isPlain && title ? (
              <span
                className="hidden h-px w-3 shrink-0 bg-[#7b8ca3] sm:block dark:bg-[#5f6d80]"
                aria-hidden="true"
              />
            ) : null}
            <p
              className={
                isShowcase && isPlain
                  ? "min-w-0 text-sm text-[#64748b] dark:text-muted-foreground"
                  : "truncate text-sm font-medium text-foreground"
              }
            >
              {title ?? t("agentChat.thread.roleClusterActivity.title")}
            </p>
          </div>
        </div>
        <span
          className={
            isShowcase
              ? "min-w-0 text-sm text-[#64748b] dark:text-muted-foreground"
              : "min-w-0 truncate text-muted-foreground"
          }
        >
          {summary ?? getAgentRoleSummary(activities, t)}
        </span>
        {headerAction ? <div className="shrink-0">{headerAction}</div> : null}
      </div>
      {isCollapsed ? null : (
        <div
          className={
            isShowcase
              ? isPlain
                ? "max-w-full min-w-0"
                : "mt-3 max-w-full min-w-0"
              : isCompact
                ? "mt-2 max-w-full overflow-x-auto pb-1 [scrollbar-gutter:stable] md:overflow-visible"
                : "mt-2 max-w-full"
          }
        >
          <div
            className={
              isShowcase
                ? "grid min-w-0 grid-cols-1 gap-3 md:grid-cols-3 xl:grid-cols-5"
                : isCompact
                  ? "flex min-w-max gap-2"
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
      )}
      {footer && !isCollapsed ? (
        <p className="mt-2 text-[11px]/5 text-muted-foreground">{footer}</p>
      ) : null}
    </section>
  )
}
