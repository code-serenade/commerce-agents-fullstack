import { ChevronsUpDown, SlidersHorizontal } from "lucide-react"
import { useTranslation } from "react-i18next"
import { Button } from "@workspace/ui-components"
import type {
  AgentChatRuntimeSelection,
  AgentRuntimeOptions,
} from "../domain/agent-runtime-options"
import type { AgentChatStrategyId } from "../domain/agent-chat-strategy"
import { AgentChatSettingsControls } from "./agent-chat-settings-controls"
import { ComposerPanelFrame } from "./composer-panel-layout"

export interface AgentChatSettingsPanelProps {
  isLoadingPlaybooks?: boolean
  isLoadingRuntimeOptions: boolean
  isOpen: boolean
  isStreaming: boolean
  onCollaborationModeChange: (value: string) => void
  onLearningLevelChange: (value: string) => void
  onPrimaryRoleChange: (value: string) => void
  onRoutingModeChange: (value: string) => void
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
  onPrimaryRoleChange,
  onRoutingModeChange,
  onStrategyChange,
  onToggleOpen,
  onToggleRole,
  runtimeOptions,
  runtimeSelection,
  strategyId,
  variant = "bar",
}: AgentChatSettingsPanelProps) {
  const { t } = useTranslation()
  const activeStrategyLabel = t(
    `agentChat.strategy.options.${strategyId}.label`
  )
  const depthChoice = runtimeOptions?.learningLevels.find(
    (level) => level.id === runtimeSelection.learningLevel
  )
  const depthLabel = depthChoice
    ? translateOrFallback(
        t,
        `agentChat.composer.runtime.learningLevels.${runtimeSelection.learningLevel}.label`,
        depthChoice.label
      )
    : runtimeSelection.learningLevel
  const roleModeLabel = roleModeCopyKey(runtimeSelection.collaborationMode)
    ? t(roleModeCopyKey(runtimeSelection.collaborationMode))
    : runtimeSelection.collaborationMode
  const runtimeLabel = [
    t("agentChat.composer.settings.depthSummary", { depth: depthLabel }),
    roleModeLabel,
    runtimeSelection.primaryRole
      ? translateOrFallback(
          t,
          `agentChat.composer.runtime.roleLabels.${runtimeSelection.primaryRole}.name`,
          runtimeSelection.primaryRole
        )
      : null,
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
      <ComposerPanelFrame
        title={t("agentChat.composer.settings.panelTitle")}
        description={t("agentChat.composer.settings.panelHint", {
          mode: activeStrategyLabel,
          summary: runtimeLabel,
        })}
      >
        <AgentChatSettingsControls
          isLoadingRuntimeOptions={isLoadingRuntimeOptions}
          isStreaming={isStreaming}
          onCollaborationModeChange={onCollaborationModeChange}
          onLearningLevelChange={onLearningLevelChange}
          onPrimaryRoleChange={onPrimaryRoleChange}
          onRoutingModeChange={onRoutingModeChange}
          onStrategyChange={onStrategyChange}
          onToggleRole={onToggleRole}
          runtimeOptions={runtimeOptions}
          runtimeSelection={runtimeSelection}
          size="compact"
          strategyId={strategyId}
        />
      </ComposerPanelFrame>
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
        className="flex min-w-0 shrink-0 items-center gap-1.5 overflow-hidden sm:flex-1"
        data-composer-interactive="true"
      >
        <Button
          type="button"
          disabled={isStreaming}
          onClick={onToggleOpen}
          classNameMode="replace"
          className={`flex h-7 max-w-28 shrink-0 items-center gap-1.5 rounded-full border px-2.5 text-xs font-medium transition-colors disabled:cursor-not-allowed disabled:opacity-50 sm:max-w-36 ${
            isOpen
              ? "border-primary/30 bg-(--agent-chat-accent-soft) text-(--agent-chat-accent-foreground) shadow-sm"
              : "border-(--agent-chat-soft-border) bg-(--agent-chat-muted-control) text-muted-foreground hover:bg-(--agent-chat-accent-soft) hover:text-(--agent-chat-accent-foreground)"
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
        </Button>
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
        <Button
          type="button"
          disabled={isStreaming}
          onClick={onToggleOpen}
          classNameMode="replace"
          className="flex shrink-0 items-center gap-1.5 rounded-full border border-border bg-card px-3 py-1.5 text-xs text-muted-foreground transition-colors hover:bg-muted hover:text-foreground disabled:cursor-not-allowed disabled:opacity-50"
          aria-expanded={isOpen}
        >
          <ChevronsUpDown className="size-3.5" />
          {t("agentChat.composer.settings.toggle")}
        </Button>
      </div>
      {isOpen ? (
        <div className="grid gap-2 rounded-xl border border-border bg-background/80 p-2">
          <AgentChatSettingsControls
            isLoadingRuntimeOptions={isLoadingRuntimeOptions}
            isStreaming={isStreaming}
            onCollaborationModeChange={onCollaborationModeChange}
            onLearningLevelChange={onLearningLevelChange}
            onPrimaryRoleChange={onPrimaryRoleChange}
            onRoutingModeChange={onRoutingModeChange}
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

function translateOrFallback(
  t: (key: string) => string,
  key: string,
  fallback: string
) {
  const translated = t(key)
  return translated === key ? fallback : translated
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
