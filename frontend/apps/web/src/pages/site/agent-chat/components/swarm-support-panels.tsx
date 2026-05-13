import {
  Bot,
  ClipboardCheck,
  Database,
  FileText,
  MonitorCog,
  PlugZap,
  ShieldAlert,
} from "lucide-react"
import type { ComponentType, SVGProps } from "react"
import { useTranslation } from "react-i18next"
import { useNavigate } from "react-router"
import { WORKSPACE_LANES } from "../view-model/swarm-panel-view-model"
import { StatePill } from "./swarm-state-pill"

const WORKSPACE_HANDOFFS = [
  ["agentChat.swarm.handoffs.context.title", "agentChat.swarm.handoffs.context.detail", Database, "/products"],
  ["agentChat.swarm.handoffs.execution.title", "agentChat.swarm.handoffs.execution.detail", ClipboardCheck, "/workspaces"],
  ["agentChat.swarm.handoffs.delivery.title", "agentChat.swarm.handoffs.delivery.detail", FileText, "/agent-output"],
  ["agentChat.swarm.handoffs.governance.title", "agentChat.swarm.handoffs.governance.detail", ShieldAlert, "/approvals"],
  ["agentChat.swarm.handoffs.review.title", "agentChat.swarm.handoffs.review.detail", Bot, "/overview"],
] as const

export function WorkspaceHandoffRail() {
  const { t } = useTranslation()
  const navigate = useNavigate()

  return (
    <div className="rounded-xl border border-border bg-background p-3 shadow-sm">
      <div className="mb-3 flex items-center justify-between gap-3">
        <div className="flex items-center gap-2 text-sm font-medium">
          <PlugZap className="size-4 text-muted-foreground" />
          {t("agentChat.workbench.handoff")}
        </div>
        <span className="text-xs text-muted-foreground">
          {t("agentChat.workbench.futureModules")}
        </span>
      </div>
      <div className="grid gap-2 min-[1500px]:grid-cols-2 min-[1800px]:grid-cols-4">
        {WORKSPACE_HANDOFFS.map(([titleKey, detailKey, Icon, path]) => (
          <button
            key={titleKey}
            type="button"
            className="flex min-w-0 items-start gap-2 rounded-lg border border-border bg-card px-3 py-2 text-left transition-colors hover:bg-muted"
            onClick={() => navigate(path)}
          >
            <div className="flex size-8 shrink-0 items-center justify-center rounded-md bg-muted text-muted-foreground">
              <Icon className="size-4" />
            </div>
            <div className="min-w-0">
              <p className="truncate text-sm font-medium">{t(titleKey)}</p>
              <p className="mt-1 line-clamp-2 text-xs/5 text-muted-foreground">
                {t(detailKey)}
              </p>
            </div>
          </button>
        ))}
      </div>
    </div>
  )
}

export function CompactInfoBlock({
  icon: Icon,
  items,
  title,
}: {
  icon: ComponentType<SVGProps<SVGSVGElement>>
  items: string[]
  title: string
}) {
  return (
    <div className="rounded-xl border border-border bg-background p-3 shadow-sm">
      <div className="mb-2 flex items-center gap-2 text-sm font-medium">
        <Icon className="size-4 text-muted-foreground" />
        {title}
      </div>
      <div className="space-y-1.5">
        {items.map((item) => (
          <div
            key={item}
            className="flex items-center gap-2 text-xs text-muted-foreground"
          >
            <span className="size-1.5 shrink-0 rounded-full bg-muted-foreground/35" />
            <span className="truncate">{item}</span>
          </div>
        ))}
      </div>
    </div>
  )
}

export function SwarmIdleDetail({
  hasTask,
  isStreaming,
  lastTask,
}: {
  hasTask: boolean
  isStreaming: boolean
  lastTask?: string
}) {
  const { t } = useTranslation()
  const idleState = getIdleState(hasTask, isStreaming, t)
  const lines = t("agentChat.workbench.idle.lines", {
    returnObjects: true,
  }) as string[]

  return (
    <div className="p-3 min-[1500px]:p-4">
      <div className="min-w-0 overflow-hidden rounded-xl border border-[#d8d8d3] bg-white shadow-sm dark:border-border dark:bg-background">
        <div className="flex items-center justify-between gap-3 border-b border-border bg-white px-5 py-3 dark:bg-background">
          <div className="flex min-w-0 items-center gap-2">
            <MonitorCog className="size-4 shrink-0 text-muted-foreground" />
            <div className="min-w-0">
              <p className="truncate font-mono text-sm font-semibold">
                {t("agentChat.workbench.idle.loopTitle")}
              </p>
              <p className="truncate text-xs text-muted-foreground">
                {t("agentChat.workbench.idle.previewTitle")}
              </p>
            </div>
          </div>
          <StatePill state={isStreaming ? "working" : "idle"} />
        </div>
        <div className="grid min-h-100 grid-rows-[auto_minmax(0,1fr)_auto]">
          <div className="border-b border-border bg-white px-5 py-3 dark:bg-background">
            <span className="text-xs font-medium text-muted-foreground">
              {t("agentChat.workbench.businessInput")}
            </span>
            <p className="mt-1 line-clamp-2 text-sm/6 text-foreground">
              {lastTask ?? idleState.description}
            </p>
          </div>
          <IdlePreview title={idleState.title} description={idleState.description} lines={lines} />
          <div className="grid gap-2 border-t border-border bg-[#f5f5f2] p-3 min-[1500px]:grid-cols-4 dark:bg-card">
            {WORKSPACE_LANES.map(([labelKey, valueKey]) => (
              <div
                key={labelKey}
                className="min-w-0 rounded-md border border-border bg-background px-3 py-2"
              >
                <p className="text-xs text-muted-foreground">{t(labelKey)}</p>
                <p className="mt-1 truncate text-xs font-medium min-[1500px]:text-sm">
                  {t(valueKey)}
                </p>
              </div>
            ))}
          </div>
        </div>
      </div>
    </div>
  )
}

function IdlePreview({
  description,
  lines,
  title,
}: {
  description: string
  lines: string[]
  title: string
}) {
  const { t } = useTranslation()

  return (
    <div className="min-h-0 bg-white p-3 min-[1500px]:p-5 dark:bg-background">
      <div className="flex h-full min-h-72 flex-col overflow-hidden rounded-lg border border-border bg-[#fbfbfa] dark:bg-card">
        <div className="flex items-center justify-between gap-2 border-b border-border bg-[#f7f7f4] px-4 py-2.5 dark:bg-card">
          <div className="flex min-w-0 items-center gap-2">
            <FileText className="size-4 text-muted-foreground" />
            <span className="truncate text-sm font-medium">
              {t("agentChat.workbench.idle.previewFile")}
            </span>
          </div>
          <span className="shrink-0 rounded-full bg-secondary px-2 py-1 text-[11px] text-secondary-foreground">
            {t("agentChat.workbench.idle.pending")}
          </span>
        </div>
        <div className="min-h-0 flex-1 overflow-hidden py-2 font-mono text-[11px]/5">
          {[`# ${title}`, description, ...lines].map((line, index) => (
            <div
              key={`${index}-${line}`}
              className="grid grid-cols-[2.5rem_minmax(0,1fr)] gap-3 px-3 py-0.5"
            >
              <span className="text-right text-muted-foreground/70 tabular-nums select-none">
                {String(index + 1).padStart(2, "0")}
              </span>
              <span className="min-w-0 wrap-break-word text-foreground/85">
                {line}
              </span>
            </div>
          ))}
        </div>
      </div>
    </div>
  )
}

function getIdleState(
  hasTask: boolean,
  isStreaming: boolean,
  t: ReturnType<typeof useTranslation>["t"]
) {
  if (isStreaming) {
    return {
      title: t("agentChat.workbench.idle.planningTitle"),
      description: t("agentChat.workbench.idle.planningDescription"),
    }
  }
  if (hasTask) {
    return {
      title: t("agentChat.workbench.idle.historyTitle"),
      description: t("agentChat.workbench.idle.historyDescription"),
    }
  }
  return {
    title: t("agentChat.workbench.idle.waitingTitle"),
    description: t("agentChat.workbench.idle.waitingDescription"),
  }
}
