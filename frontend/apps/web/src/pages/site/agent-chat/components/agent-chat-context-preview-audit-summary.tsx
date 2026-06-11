import { Boxes, Brain, Gauge, History, LibraryBig } from "lucide-react"
import type { TFunction } from "i18next"
import type { ReactNode } from "react"
import { useTranslation } from "react-i18next"
import type {
  AgentContextPreviewAuditSummary,
  AgentContextPreviewReadiness,
} from "../view-model/agent-context-preview-view-model"

export function AgentChatContextPreviewAuditSummary({
  summary,
}: {
  summary: AgentContextPreviewAuditSummary
}) {
  const { t } = useTranslation()

  return (
    <section className="grid gap-2">
      <h3 className="text-xs font-semibold text-foreground">
        {t("agentChat.composer.contextPreview.audit.title")}
      </h3>
      <div className="grid gap-2 sm:grid-cols-2">
        <AuditSummaryItem
          icon={<History className="size-3.5" aria-hidden="true" />}
          label={t("agentChat.composer.contextPreview.audit.history.label")}
          value={t("agentChat.composer.contextPreview.audit.history.value", {
            count: summary.conversation.messageCount,
            total: summary.conversation.totalMessageCount,
          })}
          detail={t("agentChat.composer.contextPreview.audit.history.detail", {
            chars: summary.conversation.historyChars,
            memory: summary.conversation.memoryChars,
            older: summary.conversation.relevantOlderCount,
            truncated: summary.conversation.truncated
              ? t("agentChat.composer.contextPreview.audit.history.truncated")
              : t("agentChat.composer.contextPreview.audit.history.complete"),
          })}
          muted={!summary.conversation.hasHistory}
        />
        <AuditSummaryItem
          icon={<Brain className="size-3.5" aria-hidden="true" />}
          label={t("agentChat.composer.contextPreview.audit.memory.label")}
          value={t("agentChat.composer.contextPreview.audit.memory.value", {
            constraints: summary.conversationMemory.constraintCount,
            decisions: summary.conversationMemory.decisionCount,
            facts: summary.conversationMemory.factCount,
            questions: summary.conversationMemory.openQuestionCount,
          })}
          detail={t("agentChat.composer.contextPreview.audit.memory.detail", {
            chars: summary.conversation.memoryChars,
            summary: summary.conversationMemory.summaryAvailable
              ? t("agentChat.composer.contextPreview.audit.memory.summaryReady")
              : t("agentChat.composer.contextPreview.audit.memory.summaryEmpty"),
          })}
          muted={!summary.conversationMemory.available}
        />
        <AuditSummaryItem
          icon={<LibraryBig className="size-3.5" aria-hidden="true" />}
          label={t("agentChat.composer.contextPreview.audit.workspace.label")}
          value={t("agentChat.composer.contextPreview.audit.workspace.value", {
            count: summary.workspace.sourceCount,
            readiness: getReadinessLabel(summary.workspace.readiness, t),
          })}
          detail={t("agentChat.composer.contextPreview.audit.workspace.detail", {
            missing: summary.workspace.missingCount,
            warnings: summary.workspace.warningCount,
          })}
          muted={summary.workspace.readiness === "empty"}
        />
        <AuditSummaryItem
          icon={<Boxes className="size-3.5" aria-hidden="true" />}
          label={t("agentChat.composer.contextPreview.audit.pack.label")}
          value={t("agentChat.composer.contextPreview.audit.pack.value", {
            count: summary.workspacePack.readableItemCount,
            total: summary.workspacePack.itemCount,
          })}
          detail={t("agentChat.composer.contextPreview.audit.pack.detail", {
            chars: summary.workspacePack.charCount,
            omitted: summary.workspacePack.omittedCount,
            readiness: getReadinessLabel(summary.workspacePack.readiness, t),
            warnings: summary.workspacePack.warningCount,
          })}
          muted={summary.workspacePack.readableItemCount === 0}
        />
        <AuditSummaryItem
          icon={<Gauge className="size-3.5" aria-hidden="true" />}
          label={t("agentChat.composer.contextPreview.audit.budget.label")}
          value={t("agentChat.composer.contextPreview.audit.budget.value", {
            count: summary.budget.totalContextChars,
          })}
          detail={t("agentChat.composer.contextPreview.audit.budget.detail", {
            input: summary.budget.inputChars,
          })}
        />
      </div>
    </section>
  )
}

function AuditSummaryItem({
  detail,
  icon,
  label,
  muted = false,
  value,
}: {
  detail: string
  icon: ReactNode
  label: string
  muted?: boolean
  value: string
}) {
  return (
    <div className="min-w-0 rounded-md border border-border bg-card p-2.5">
      <div className="flex min-w-0 items-center gap-2">
        <span
          className={
            muted
              ? "shrink-0 text-muted-foreground"
              : "shrink-0 text-primary"
          }
        >
          {icon}
        </span>
        <span className="truncate text-[11px] font-medium text-muted-foreground">
          {label}
        </span>
      </div>
      <div className="mt-1 truncate text-xs font-semibold text-foreground">
        {value}
      </div>
      <p className="mt-1 truncate text-[11px] text-muted-foreground">
        {detail}
      </p>
    </div>
  )
}

function getReadinessLabel(
  readiness: AgentContextPreviewReadiness,
  t: TFunction
) {
  return t(`agentChat.composer.contextPreview.audit.readiness.${readiness}`)
}
