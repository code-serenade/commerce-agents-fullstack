import {
  BrainCircuit,
  ChevronDown,
  FileText,
  Megaphone,
  ShoppingBag,
  UsersRound,
} from "lucide-react"
import { useTranslation } from "react-i18next"
import type {
  AgentChatMessageContextReference,
  AgentChatSourceReadiness,
  AgentChatMessageContextSummary,
} from "../types"

export function MessageContextSummary({
  summary,
}: {
  summary?: AgentChatMessageContextSummary
}) {
  const { t } = useTranslation()
  if (!summary) return null
  const sourceReadiness = summarizeSourceReadiness(summary.sources)

  const chips = [
    {
      icon: <ShoppingBag className="size-3.5" />,
      label:
        summary.productIds.length > 0
          ? t("agentChat.thread.contextSummary.productCount", {
              count: summary.productIds.length,
            })
          : t("agentChat.thread.contextSummary.productNone"),
    },
  ]

  if (summary.sourceRefs.length > 0) {
    chips.push({
      icon: <FileText className="size-3.5" />,
      label: sourceReadiness.total
        ? t("agentChat.thread.contextSummary.sourceCountWithReadiness", {
            count: summary.sourceRefs.length,
            readable: sourceReadiness.readable,
            unreadable: summary.sourceRefs.length - sourceReadiness.readable,
          })
        : t("agentChat.thread.contextSummary.sourceCount", {
            count: summary.sourceRefs.length,
          }),
    })
  }

  if (summary.campaignIds.length > 0) {
    chips.push({
      icon: <Megaphone className="size-3.5" />,
      label: t("agentChat.thread.contextSummary.campaignCount", {
        count: summary.campaignIds.length,
      }),
    })
  }

  if (summary.strategyId) {
    chips.push({
      icon: <BrainCircuit className="size-3.5" />,
      label: t("agentChat.thread.contextSummary.strategy", {
        strategy: t(`agentChat.strategy.options.${summary.strategyId}.label`),
      }),
    })
  }

  if (
    summary.collaborationMode === "auto" &&
    summary.selectedRoles.length === 0
  ) {
    chips.push({
      icon: <UsersRound className="size-3.5" />,
      label: t("agentChat.thread.contextSummary.autoRoles"),
    })
  } else if (summary.primaryRole || summary.selectedRoles.length > 0) {
    chips.push({
      icon: <UsersRound className="size-3.5" />,
      label: t("agentChat.thread.contextSummary.roles", {
        count: summary.selectedRoles.length,
        role: formatContextValue(
          summary.primaryRole ?? summary.selectedRoles[0] ?? ""
        ),
      }),
    })
  }

  return (
    <div
      className="mt-1.5 flex max-w-full flex-col items-end gap-1.5 text-[11px] text-muted-foreground"
      aria-label={t("agentChat.thread.contextSummary.label")}
    >
      <div className="flex max-w-full flex-wrap justify-end gap-2">
        {chips.map((chip) => (
          <span
            key={chip.label}
            className="inline-flex max-w-full items-center gap-1.5 rounded-full border border-(--agent-chat-soft-border) bg-(--agent-chat-elevated-strong) px-2.5 py-1 text-(--agent-chat-accent-foreground) shadow-sm backdrop-blur-md"
          >
            <span className="shrink-0 text-primary">{chip.icon}</span>
            <span className="truncate">{chip.label}</span>
          </span>
        ))}
      </div>
      {summary.products.length > 0 ||
      summary.campaigns.length > 0 ||
      summary.sources.length > 0 ? (
        <details className="group max-w-full">
          <summary className="flex cursor-pointer list-none items-center gap-1.5 rounded-full border border-(--agent-chat-soft-border) bg-(--agent-chat-elevated-strong) px-2.5 py-1 text-(--agent-chat-accent-foreground) shadow-sm backdrop-blur-md transition-colors hover:bg-(--agent-chat-accent-soft) hover:text-primary [&::-webkit-details-marker]:hidden">
            <span>{t("agentChat.thread.contextSummary.details")}</span>
            <ChevronDown className="size-3.5 transition-transform group-open:rotate-180" />
          </summary>
          <div className="mt-1.5 max-h-36 w-72 max-w-[min(72vw,18rem)] overflow-y-auto rounded-2xl border border-(--agent-chat-soft-border) bg-(--agent-chat-elevated-strong) p-2 text-left shadow-(--agent-chat-card-shadow) backdrop-blur-xl">
            <ContextReferenceGroup
              icon={<ShoppingBag className="size-3.5" />}
              items={summary.products}
              title={t("agentChat.thread.contextSummary.productDetails")}
            />
            <ContextReferenceGroup
              icon={<Megaphone className="size-3.5" />}
              items={summary.campaigns}
              title={t("agentChat.thread.contextSummary.campaignDetails")}
            />
            <ContextReferenceGroup
              icon={<FileText className="size-3.5" />}
              items={summary.sources}
              readinessLabel={(readiness) =>
                t(
                  `agentChat.thread.contextSummary.sourceReadiness.${readiness}`
                )
              }
              title={t("agentChat.thread.contextSummary.sourceDetails")}
            />
          </div>
        </details>
      ) : null}
    </div>
  )
}

function ContextReferenceGroup({
  icon,
  items,
  readinessLabel,
  title,
}: {
  icon: React.ReactNode
  items: AgentChatMessageContextReference[]
  readinessLabel?: (readiness: AgentChatSourceReadiness) => string
  title: string
}) {
  if (items.length === 0) return null

  return (
    <div className="space-y-1 py-1 first:pt-0 last:pb-0">
      <div className="flex items-center gap-1.5 font-medium text-foreground">
        {icon}
        <span>{title}</span>
      </div>
      <div className="space-y-1">
        {items.map((item) => (
          <div
            key={item.id}
            className="rounded-xl bg-(--agent-chat-subtle-surface) px-2 py-1 text-muted-foreground"
            title={item.id}
          >
            <div className="truncate text-foreground">
              {item.label ?? item.id}
            </div>
            {item.readiness && readinessLabel ? (
              <div className="mt-1">
                <span className="rounded-full bg-muted-foreground/10 px-1.5 py-0.5 text-[10px] text-muted-foreground">
                  {readinessLabel(item.readiness)}
                </span>
              </div>
            ) : null}
            {item.secondary ? (
              <div className="truncate">{item.secondary}</div>
            ) : item.label ? (
              <div className="truncate">{item.id}</div>
            ) : null}
          </div>
        ))}
      </div>
    </div>
  )
}

function formatContextValue(value: string) {
  return value.replace(/_/g, " ")
}

function summarizeSourceReadiness(items: AgentChatMessageContextReference[]) {
  return items.reduce(
    (summary, item) => {
      if (!item.readiness) return summary
      summary.total += 1
      if (item.readiness === "readable") {
        summary.readable += 1
      } else {
        summary.unreadable += 1
      }
      return summary
    },
    { readable: 0, total: 0, unreadable: 0 }
  )
}
