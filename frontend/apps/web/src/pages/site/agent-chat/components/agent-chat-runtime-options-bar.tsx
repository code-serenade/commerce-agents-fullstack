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
  localizedRoleOption,
  roleModeToOption,
  sortRoleModes,
} from "../view-model/agent-chat-runtime-options-view-model"
import { OptionSegment } from "./agent-chat-runtime-shared"

export interface AgentChatRuntimeOptionsBarProps {
  disabled: boolean
  isLoadingRuntimeOptions: boolean
  onCollaborationModeChange: (value: string) => void
  onLearningLevelChange: (value: string) => void
  onPrimaryRoleChange: (value: string) => void
  onRoutingModeChange: (value: string) => void
  onToggleRole: (value: string) => void
  runtimeOptions: AgentRuntimeOptions | null
  runtimeSelection: AgentChatRuntimeSelection
  strategyId?: AgentChatStrategyId
}

export function AgentChatRuntimeOptionsBar({
  disabled,
  isLoadingRuntimeOptions,
  onCollaborationModeChange,
  onLearningLevelChange,
  onPrimaryRoleChange,
  onRoutingModeChange,
  onToggleRole,
  runtimeOptions,
  runtimeSelection,
}: AgentChatRuntimeOptionsBarProps) {
  const { t } = useTranslation()

  if (!runtimeOptions && !isLoadingRuntimeOptions) return null

  const controlDisabled = disabled || isLoadingRuntimeOptions
  const roleModes = sortRoleModes(runtimeOptions?.collaborationModes ?? [])
  const specialistRoles = (runtimeOptions?.roles ?? []).filter(
    (role) => role.runtimeRole !== "chat"
  ).map((role) => localizedRoleOption(role, t))

  return (
    <section className="grid gap-2">
      {!runtimeOptions && isLoadingRuntimeOptions ? (
        <div className="rounded-lg border border-border bg-card px-3 py-2 text-xs text-muted-foreground">
          {t("agentChat.composer.runtime.loading")}
        </div>
      ) : null}
      {runtimeOptions ? (
        <>
          <AdvancedRuntimeOptions
            disabled={controlDisabled}
            learningLevels={runtimeOptions.learningLevels}
            onLearningLevelChange={onLearningLevelChange}
            runtimeSelection={runtimeSelection}
          />
          <OptionSegment
            disabled={controlDisabled}
            columns={2}
            label={t("agentChat.composer.runtime.routingMode")}
            onValueChange={onRoutingModeChange}
            options={runtimeOptions.routingModes.map((choice) =>
              choiceToOption(choice, t, "routingModes")
            )}
            value={runtimeSelection.routingMode}
          />
          <OptionSegment
            disabled={controlDisabled}
            label={t("agentChat.composer.runtime.roleMode")}
            onValueChange={onCollaborationModeChange}
            options={roleModes.map((mode) => roleModeToOption(mode, t))}
            value={runtimeSelection.collaborationMode}
          />
          <RoleSelector
            disabled={controlDisabled}
            onPrimaryRoleChange={onPrimaryRoleChange}
            onToggleRole={onToggleRole}
            roles={specialistRoles}
            runtimeSelection={runtimeSelection}
          />
        </>
      ) : null}
    </section>
  )
}
