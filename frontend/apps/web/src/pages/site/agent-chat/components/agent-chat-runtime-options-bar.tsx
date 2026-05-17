import { Cpu, GitBranch } from "lucide-react"
import { useTranslation } from "react-i18next"
import type {
  AgentChatRuntimeSelection,
  AgentRuntimeOptions,
} from "../domain/agent-runtime-options"
import type { AgentChatStrategyId } from "../domain/agent-chat-strategy"
import { AdvancedRuntimeOptions } from "./agent-chat-advanced-runtime-options"
import { RoleSelector } from "./agent-chat-role-selector"
import {
  choiceToOption,
  modelToSelectOption,
  sortRoleModes,
} from "../view-model/agent-chat-runtime-options-view-model"
import {
  OptionSegment,
  RuntimeSelect,
} from "./agent-chat-runtime-shared"

export interface AgentChatRuntimeOptionsBarProps {
  disabled: boolean
  isLoadingRuntimeOptions: boolean
  onCollaborationModeChange: (value: string) => void
  onLearningLevelChange: (value: string) => void
  onModelChange: (value: string) => void
  onPrimaryRoleChange: (value: string) => void
  onToggleRole: (value: string) => void
  runtimeOptions: AgentRuntimeOptions | null
  runtimeSelection: AgentChatRuntimeSelection
  strategyId: AgentChatStrategyId
}

export function AgentChatRuntimeOptionsBar({
  disabled,
  isLoadingRuntimeOptions,
  onCollaborationModeChange,
  onLearningLevelChange,
  onModelChange,
  onPrimaryRoleChange,
  onToggleRole,
  runtimeOptions,
  runtimeSelection,
  strategyId,
}: AgentChatRuntimeOptionsBarProps) {
  const { t } = useTranslation()

  if (!runtimeOptions && !isLoadingRuntimeOptions) return null

  const controlDisabled = disabled || isLoadingRuntimeOptions
  const roleModes = sortRoleModes(runtimeOptions?.collaborationModes ?? [])
  const specialistRoles = (runtimeOptions?.roles ?? []).filter(
    (role) => role.runtimeRole !== "chat"
  )

  return (
    <div className="grid gap-3 rounded-xl border border-border bg-background p-3">
      <div className="grid gap-2 sm:grid-cols-[minmax(0,1fr)_auto] sm:items-center">
        <div className="min-w-0">
          <div className="flex min-w-0 items-center gap-1.5 text-xs font-semibold text-foreground">
            <Cpu className="size-3.5 shrink-0 text-muted-foreground" />
            <span>{t("agentChat.composer.runtime.label")}</span>
          </div>
          <p className="mt-1 text-xs/5 text-muted-foreground">
            {t(`agentChat.composer.runtime.strategyNotes.${strategyId}`)}
          </p>
        </div>
        <div className="flex min-w-0 flex-wrap items-center justify-start gap-2 sm:justify-end">
          {isLoadingRuntimeOptions ? (
            <span className="rounded-full border border-border bg-background/70 px-3 py-1.5 text-xs text-muted-foreground">
              {t("agentChat.composer.runtime.loading")}
            </span>
          ) : null}
          {runtimeOptions ? (
            <RuntimeSelect
              disabled={controlDisabled}
              label={t("agentChat.composer.runtime.model")}
              onValueChange={onModelChange}
              options={runtimeOptions.models.map(modelToSelectOption)}
              value={runtimeSelection.modelName ?? ""}
            />
          ) : null}
        </div>
      </div>

      {runtimeOptions ? (
        <>
          <OptionSegment
            disabled={controlDisabled}
            icon={<GitBranch className="size-3.5" />}
            label={t("agentChat.composer.runtime.roleMode")}
            onValueChange={onCollaborationModeChange}
            options={roleModes.map(choiceToOption)}
            value={runtimeSelection.collaborationMode}
          />
          <RoleSelector
            disabled={controlDisabled}
            onPrimaryRoleChange={onPrimaryRoleChange}
            onToggleRole={onToggleRole}
            roles={specialistRoles}
            runtimeSelection={runtimeSelection}
          />
          <AdvancedRuntimeOptions
            disabled={controlDisabled}
            learningLevels={runtimeOptions.learningLevels}
            onLearningLevelChange={onLearningLevelChange}
            runtimeSelection={runtimeSelection}
          />
        </>
      ) : null}
    </div>
  )
}
