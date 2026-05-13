import { ChevronsUpDown, SlidersHorizontal } from "lucide-react"
import { useTranslation } from "react-i18next"
import type { AgentChatPlaybookSelection } from "../domain/agent-chat-playbook"
import type {
  AgentChatRuntimeSelection,
  AgentRuntimeOptions,
} from "../domain/agent-runtime-options"
import type { AgentChatStrategyId } from "../domain/agent-chat-strategy"
import { AgentChatPlaybookButtons } from "./agent-chat-playbook-buttons"
import { AgentChatRuntimeOptionsBar } from "./agent-chat-runtime-options-bar"
import { AgentChatStrategyBar } from "./agent-chat-strategy-bar"

export interface AgentChatSettingsPanelProps {
  isLoadingPlaybooks: boolean
  isLoadingRuntimeOptions: boolean
  isOpen: boolean
  isStreaming: boolean
  onCollaborationModeChange: (value: string) => void
  onLearningLevelChange: (value: string) => void
  onModelChange: (value: string) => void
  onPrimaryRoleChange: (value: string) => void
  onSelectPlaybook: (playbook: AgentChatPlaybookSelection) => void
  onStrategyChange: (strategyId: AgentChatStrategyId) => void
  onToggleOpen: () => void
  onToggleRole: (value: string) => void
  playbooks: AgentChatPlaybookSelection[]
  runtimeOptions: AgentRuntimeOptions | null
  runtimeSelection: AgentChatRuntimeSelection
  strategyId: AgentChatStrategyId
  variant?: "bar" | "compact" | "panel"
}

export function AgentChatSettingsPanel({
  isLoadingPlaybooks,
  isLoadingRuntimeOptions,
  isOpen,
  isStreaming,
  onCollaborationModeChange,
  onLearningLevelChange,
  onModelChange,
  onPrimaryRoleChange,
  onSelectPlaybook,
  onStrategyChange,
  onToggleOpen,
  onToggleRole,
  playbooks,
  runtimeOptions,
  runtimeSelection,
  strategyId,
  variant = "bar",
}: AgentChatSettingsPanelProps) {
  const { t } = useTranslation()
  const activeStrategyLabel = t(`agentChat.strategy.options.${strategyId}.label`)
  const runtimeLabel = [
    runtimeSelection.modelName,
    runtimeSelection.primaryRole,
    runtimeSelection.selectedRoles.length
      ? t("agentChat.composer.settings.rolesCount", {
          count: runtimeSelection.selectedRoles.length,
        })
      : null,
  ]
    .filter(Boolean)
    .join(" · ")

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
          isLoadingPlaybooks={isLoadingPlaybooks}
          isLoadingRuntimeOptions={isLoadingRuntimeOptions}
          isStreaming={isStreaming}
          onCollaborationModeChange={onCollaborationModeChange}
          onLearningLevelChange={onLearningLevelChange}
          onModelChange={onModelChange}
          onPrimaryRoleChange={onPrimaryRoleChange}
          onSelectPlaybook={onSelectPlaybook}
          onStrategyChange={onStrategyChange}
          onToggleRole={onToggleRole}
          playbooks={playbooks}
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
      <div className="relative shrink-0" data-composer-interactive="true">
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
            isLoadingPlaybooks={isLoadingPlaybooks}
            isLoadingRuntimeOptions={isLoadingRuntimeOptions}
            isStreaming={isStreaming}
            onCollaborationModeChange={onCollaborationModeChange}
            onLearningLevelChange={onLearningLevelChange}
            onModelChange={onModelChange}
            onPrimaryRoleChange={onPrimaryRoleChange}
            onSelectPlaybook={onSelectPlaybook}
            onStrategyChange={onStrategyChange}
            onToggleRole={onToggleRole}
            playbooks={playbooks}
            runtimeOptions={runtimeOptions}
            runtimeSelection={runtimeSelection}
            strategyId={strategyId}
          />
        </div>
      ) : null}
    </>
  )
}

function AgentChatSettingsControls({
  isLoadingPlaybooks,
  isLoadingRuntimeOptions,
  isStreaming,
  onCollaborationModeChange,
  onLearningLevelChange,
  onModelChange,
  onPrimaryRoleChange,
  onSelectPlaybook,
  onStrategyChange,
  onToggleRole,
  playbooks,
  runtimeOptions,
  runtimeSelection,
  strategyId,
}: {
  isLoadingPlaybooks: boolean
  isLoadingRuntimeOptions: boolean
  isStreaming: boolean
  onCollaborationModeChange: (value: string) => void
  onLearningLevelChange: (value: string) => void
  onModelChange: (value: string) => void
  onPrimaryRoleChange: (value: string) => void
  onSelectPlaybook: (playbook: AgentChatPlaybookSelection) => void
  onStrategyChange: (strategyId: AgentChatStrategyId) => void
  onToggleRole: (value: string) => void
  playbooks: AgentChatPlaybookSelection[]
  runtimeOptions: AgentRuntimeOptions | null
  runtimeSelection: AgentChatRuntimeSelection
  strategyId: AgentChatStrategyId
}) {
  return (
    <>
      <AgentChatStrategyBar
        disabled={isStreaming}
        onStrategyChange={onStrategyChange}
        strategyId={strategyId}
      />
      <AgentChatRuntimeOptionsBar
        disabled={isStreaming}
        isLoadingRuntimeOptions={isLoadingRuntimeOptions}
        onCollaborationModeChange={onCollaborationModeChange}
        onLearningLevelChange={onLearningLevelChange}
        onModelChange={onModelChange}
        onPrimaryRoleChange={onPrimaryRoleChange}
        onToggleRole={onToggleRole}
        runtimeOptions={runtimeOptions}
        runtimeSelection={runtimeSelection}
      />
      <AgentChatPlaybookButtons
        isLoading={isLoadingPlaybooks}
        isStreaming={isStreaming}
        onSelectPlaybook={onSelectPlaybook}
        playbooks={playbooks}
      />
    </>
  )
}

function ChevronIndicator({ open }: { open: boolean }) {
  return (
    <ChevronsUpDown
      className={`size-3.5 shrink-0 transition-transform ${open ? "rotate-180" : ""}`}
    />
  )
}
