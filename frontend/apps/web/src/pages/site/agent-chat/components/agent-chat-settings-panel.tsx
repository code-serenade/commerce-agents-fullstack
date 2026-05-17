import { ChevronsUpDown, SlidersHorizontal } from "lucide-react"
import { useState } from "react"
import { useTranslation } from "react-i18next"
import type {
  AgentChatRuntimeSelection,
  AgentRuntimeOptions,
} from "../domain/agent-runtime-options"
import type { AgentChatStrategyId } from "../domain/agent-chat-strategy"
import { buildCompactRuntimeSummary } from "../view-model/agent-chat-runtime-summary-view-model"
import { AgentChatSettingsControls } from "./agent-chat-settings-controls"

export interface AgentChatSettingsPanelProps {
  isLoadingPlaybooks?: boolean
  isLoadingRuntimeOptions: boolean
  isOpen: boolean
  isStreaming: boolean
  onCollaborationModeChange: (value: string) => void
  onLearningLevelChange: (value: string) => void
  onModelChange: (value: string) => void
  onPrimaryRoleChange: (value: string) => void
  onSelectPlaybook?: unknown
  onStrategyChange: (strategyId: AgentChatStrategyId) => void
  onToggleOpen: () => void
  onToggleRole: (value: string) => void
  playbooks?: unknown[]
  runtimeOptions: AgentRuntimeOptions | null
  runtimeSelection: AgentChatRuntimeSelection
  strategyId: AgentChatStrategyId
  variant?: "bar" | "compact" | "panel"
}

export function AgentChatSettingsPanel({
  isLoadingRuntimeOptions,
  isOpen,
  isStreaming,
  onCollaborationModeChange,
  onLearningLevelChange,
  onModelChange,
  onPrimaryRoleChange,
  onStrategyChange,
  onToggleOpen,
  onToggleRole,
  runtimeOptions,
  runtimeSelection,
  strategyId,
  variant = "bar",
}: AgentChatSettingsPanelProps) {
  const { t } = useTranslation()
  const [summaryVisible, setSummaryVisible] = useState(true)
  const activeStrategyLabel = t(`agentChat.strategy.options.${strategyId}.label`)
  const depthLabel =
    runtimeOptions?.learningLevels.find(
      (level) => level.id === runtimeSelection.learningLevel
    )?.label ?? runtimeSelection.learningLevel
  const modelLabel = runtimeSelection.modelName
  const roleModeLabel = roleModeCopyKey(runtimeSelection.collaborationMode)
    ? t(roleModeCopyKey(runtimeSelection.collaborationMode))
    : runtimeSelection.collaborationMode
  const runtimeLabel = [
    modelLabel,
    t("agentChat.composer.settings.depthSummary", { depth: depthLabel }),
    roleModeLabel,
    runtimeSelection.primaryRole,
    runtimeSelection.selectedRoles.length
      ? t("agentChat.composer.settings.rolesCount", {
          count: runtimeSelection.selectedRoles.length,
        })
      : null,
  ]
    .filter(Boolean)
    .join(" · ")
  const compactSummary = buildCompactRuntimeSummary({
    depthLabel,
    roleModeLabel,
    runtimeOptions,
    runtimeSelection,
    strategyId,
    t,
  })

  if (variant === "panel") {
    return (
      <>
        <div className="flex min-w-0 flex-wrap items-center gap-1.5 text-xs text-muted-foreground">
          <span className="font-medium text-foreground">
            {activeStrategyLabel}
          </span>
          {runtimeLabel ? (
            <>
              <span>·</span>
              <span className="truncate">{runtimeLabel}</span>
            </>
          ) : null}
        </div>
        <AgentChatSettingsControls
          isLoadingRuntimeOptions={isLoadingRuntimeOptions}
          isStreaming={isStreaming}
          onCollaborationModeChange={onCollaborationModeChange}
          onLearningLevelChange={onLearningLevelChange}
          onModelChange={onModelChange}
          onPrimaryRoleChange={onPrimaryRoleChange}
          onStrategyChange={onStrategyChange}
          onToggleRole={onToggleRole}
          runtimeOptions={runtimeOptions}
          runtimeSelection={runtimeSelection}
          strategyId={strategyId}
        />
      </>
    )
  }

  if (variant === "compact") {
    const compactTitle = [
      t("agentChat.composer.settings.compactTitle"),
      activeStrategyLabel,
      runtimeLabel,
    ]
      .filter(Boolean)
      .join(" - ")

    return (
      <div
        className="flex min-w-0 shrink items-center gap-1.5"
        data-composer-interactive="true"
      >
        <button
          type="button"
          disabled={isStreaming}
          onClick={onToggleOpen}
          className={`flex h-7 max-w-36 shrink-0 items-center gap-1.5 rounded-full border px-2.5 text-xs font-medium transition-colors disabled:cursor-not-allowed disabled:opacity-50 ${
            isOpen
              ? "border-blue-300 bg-blue-50 text-blue-700 dark:border-blue-700/70 dark:bg-blue-950/40 dark:text-blue-100"
              : "border-border bg-card text-muted-foreground hover:bg-muted hover:text-foreground"
          }`}
          aria-expanded={isOpen}
          aria-label={compactTitle}
          title={compactTitle}
        >
          <SlidersHorizontal className="size-3.5 shrink-0" />
          <span className="truncate">
            {t("agentChat.composer.settings.compactLabel")}
          </span>
          <ChevronIndicator open={isOpen} />
        </button>
        {summaryVisible ? (
          <div className="flex min-w-0 shrink items-center gap-1 text-xs text-muted-foreground">
            <span className="min-w-0 max-w-[min(48vw,26rem)] truncate">
              <span className="text-[11px] text-muted-foreground/75">
                {t("agentChat.composer.settings.selectedPrefix")}
              </span>
              <span className="ml-1 text-foreground/85">{compactSummary}</span>
            </span>
            <button
              type="button"
              className="shrink-0 text-[11px] text-muted-foreground/75 underline-offset-2 transition-colors hover:text-foreground hover:underline"
              onClick={() => setSummaryVisible(false)}
              title={t("agentChat.composer.settings.hideSelection")}
            >
              {t("agentChat.composer.settings.hideSelection")}
            </button>
          </div>
        ) : (
          <button
            type="button"
            className="shrink-0 text-[11px] text-muted-foreground underline-offset-2 transition-colors hover:text-foreground hover:underline"
            onClick={() => setSummaryVisible(true)}
            title={compactSummary}
          >
            {t("agentChat.composer.settings.showSelection")}
          </button>
        )}
      </div>
    )
  }

  return (
    <>
      <div className="flex min-w-0 items-center justify-between gap-2 rounded-xl bg-background px-3 py-2">
        <div className="flex min-w-0 flex-wrap items-center gap-1.5 text-xs text-muted-foreground">
          <span className="font-medium text-foreground">
            {activeStrategyLabel}
          </span>
          {runtimeLabel ? (
            <>
              <span>·</span>
              <span className="hidden sm:inline">{runtimeLabel}</span>
            </>
          ) : null}
        </div>
        <button
          type="button"
          disabled={isStreaming}
          onClick={onToggleOpen}
          className="flex shrink-0 items-center gap-1.5 rounded-full border border-border bg-card px-3 py-1.5 text-xs text-muted-foreground transition-colors hover:bg-muted hover:text-foreground disabled:cursor-not-allowed disabled:opacity-50"
          aria-expanded={isOpen}
        >
          <ChevronsUpDown className="size-3.5" />
          {t("agentChat.composer.settings.toggle")}
        </button>
      </div>
      {isOpen ? (
        <div className="grid gap-2 rounded-xl border border-border bg-background/80 p-2">
          <AgentChatSettingsControls
            isLoadingRuntimeOptions={isLoadingRuntimeOptions}
            isStreaming={isStreaming}
            onCollaborationModeChange={onCollaborationModeChange}
            onLearningLevelChange={onLearningLevelChange}
            onModelChange={onModelChange}
            onPrimaryRoleChange={onPrimaryRoleChange}
            onStrategyChange={onStrategyChange}
            onToggleRole={onToggleRole}
            runtimeOptions={runtimeOptions}
            runtimeSelection={runtimeSelection}
            strategyId={strategyId}
          />
        </div>
      ) : null}
    </>
  )
}

function roleModeCopyKey(mode: string) {
  if (mode === "auto" || mode === "single" || mode === "manual") {
    return `agentChat.composer.runtime.roleModes.${mode}.label`
  }
  return ""
}

function ChevronIndicator({ open }: { open: boolean }) {
  return (
    <ChevronsUpDown
      className={`size-3.5 shrink-0 transition-transform ${open ? "rotate-180" : ""}`}
    />
  )
}
