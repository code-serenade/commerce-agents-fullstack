import {
  Bot,
  ClipboardCheck,
  Database,
  FileText,
  MonitorCog,
  PlugZap,
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
  ["agentChat.swarm.handoffs.review.title", "agentChat.swarm.handoffs.review.detail", Bot, "/overview"],
] as const

export function WorkspaceHandoffRail({ compact = false }: { compact?: boolean }) {
  const { t } = useTranslation()
  const navigate = useNavigate()

  return (
    <div className="rounded-md border border-border bg-background p-2">
      <div className="mb-2 flex items-center justify-between gap-2">
        <div className="flex min-w-0 items-center gap-1.5 text-xs font-medium">
          <PlugZap className="size-4 text-muted-foreground" />
          <span className="truncate">{t("agentChat.workbench.handoff")}</span>
        </div>
        <span className="hidden shrink-0 text-xs text-muted-foreground min-[1500px]:inline">
          {t("agentChat.workbench.futureModules")}
        </span>
      </div>
      <div className="grid gap-1">
        {WORKSPACE_HANDOFFS.map(([titleKey, detailKey, Icon, path]) => (
          <button
            key={titleKey}
            type="button"
            className="flex min-w-0 items-start gap-2 rounded-md border border-border bg-card px-2 py-1.5 text-left transition-colors hover:bg-muted"
            onClick={() => navigate(path)}
          >
            <div className="mt-0.5 flex size-6 shrink-0 items-center justify-center rounded-md bg-muted text-muted-foreground">
              <Icon className="size-3.5" />
            </div>
            <div className="min-w-0 flex-1">
              <p className="truncate text-xs font-medium">{t(titleKey)}</p>
              {compact ? null : (
                <p className="mt-1 wrap-break-word text-xs/5 text-muted-foreground">
                  {t(detailKey)}
                </p>
              )}
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
    <div className="rounded-md border border-border bg-background p-2">
      <div className="mb-2 flex min-w-0 items-center gap-1.5 text-xs font-medium">
        <Icon className="size-4 text-muted-foreground" />
        <span className="truncate">{title}</span>
      </div>
      <div className="space-y-1">
        {items.slice(0, 3).map((item, index) => (
          <div
            key={`${index}-${item}`}
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
    <div className="min-h-0">
      <div className="min-w-0 overflow-hidden rounded-lg border border-border bg-card">
        <div className="flex items-center justify-between gap-3 border-b border-border bg-card px-5 py-3">
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
        <div className="grid min-h-0 grid-rows-[auto_minmax(0,1fr)_auto]">
          <div className="border-b border-border bg-card px-5 py-3">
            <span className="text-xs font-medium text-muted-foreground">
              {t("agentChat.workbench.businessInput")}
            </span>
            <p className="mt-1 wrap-break-word text-sm/6 text-foreground">
              {lastTask ?? idleState.description}
            </p>
          </div>
          <IdlePreview title={idleState.title} description={idleState.description} lines={lines} />
          <div className="grid gap-2 border-t border-border bg-background p-3 sm:grid-cols-2">
            {WORKSPACE_LANES.map(([labelKey, valueKey]) => (
              <div
                key={labelKey}
                className="min-w-0 rounded-md border border-border bg-background px-3 py-2"
              >
                <p className="text-xs text-muted-foreground">{t(labelKey)}</p>
                <p className="mt-1 truncate text-xs font-medium">
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
    <div className="min-h-0 bg-card p-3">
      <div className="flex max-h-56 min-h-0 flex-col overflow-hidden rounded-md border border-border bg-background">
        <div className="flex items-center justify-between gap-2 border-b border-border bg-card px-4 py-2.5">
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
        <div className="min-h-0 flex-1 overflow-x-hidden overflow-y-auto py-2 font-mono text-[11px]/5 [scrollbar-gutter:stable]">
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
