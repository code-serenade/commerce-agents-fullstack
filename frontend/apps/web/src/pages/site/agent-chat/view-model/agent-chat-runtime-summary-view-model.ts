import type { TFunction } from "i18next"
import type {
  AgentChatRuntimeSelection,
  AgentRuntimeOptions,
} from "../domain/agent-runtime-options"
import type { AgentChatStrategyId } from "../domain/agent-chat-strategy"

export function buildCompactRuntimeSummary({
  runtimeOptions,
  runtimeSelection,
  strategyId,
  t,
}: {
  runtimeOptions: AgentRuntimeOptions | null
  runtimeSelection: AgentChatRuntimeSelection
  strategyId: AgentChatStrategyId
  t: TFunction
}) {
  const strategyShort = t(`agentChat.strategy.options.${strategyId}.shortHint`)
  const depthLabel = runtimeOptions?.learningLevels.find(
    (level) => level.id === runtimeSelection.learningLevel
  )?.label ?? runtimeSelection.learningLevel
  const roleModeLabel =
    runtimeOptions?.collaborationModes.find(
      (mode) => mode.id === runtimeSelection.collaborationMode
    )?.label ?? runtimeSelection.collaborationMode
  const routingLabel =
    runtimeOptions?.routingModes.find(
      (mode) => mode.id === runtimeSelection.routingMode
    )?.label ?? runtimeSelection.routingMode
  const roleLabel = selectedRoleSummary({
    runtimeOptions,
    runtimeSelection,
    t,
  })

  return [strategyShort, routingLabel, roleModeLabel, depthLabel, roleLabel]
    .filter(Boolean)
    .join(" / ")
}

export function selectedRoleSummary({
  runtimeOptions,
  runtimeSelection,
  t,
}: {
  runtimeOptions: AgentRuntimeOptions | null
  runtimeSelection: AgentChatRuntimeSelection
  t: TFunction
}) {
  if (runtimeSelection.collaborationMode === "auto") {
    return t("agentChat.composer.settings.autoRolesSummary")
  }

  if (runtimeSelection.collaborationMode === "manual") {
    const count = runtimeSelection.selectedRoles.length
    return t("agentChat.composer.settings.multiAgentSummary", { count })
  }

  const roleName = (runtimeRole: string | undefined) => {
    if (!runtimeRole) return ""
    return (
      runtimeOptions?.roles.find((role) => role.runtimeRole === runtimeRole)
        ?.name ?? runtimeRole
    )
  }

  if (runtimeSelection.collaborationMode === "single") {
    return roleName(runtimeSelection.primaryRole)
  }

  const lead = roleName(runtimeSelection.primaryRole)
  const supportCount = runtimeSelection.selectedRoles.filter(
    (role) => role !== runtimeSelection.primaryRole
  ).length
  if (!lead) {
    return t("agentChat.composer.settings.rolesCount", {
      count: runtimeSelection.selectedRoles.length,
    })
  }
  if (supportCount <= 0) return lead
  return `${lead}+${supportCount}`
}
