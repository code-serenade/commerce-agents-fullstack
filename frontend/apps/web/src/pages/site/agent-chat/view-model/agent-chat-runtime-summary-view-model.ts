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
  const roleModeChoice = runtimeOptions?.collaborationModes.find(
    (mode) => mode.id === runtimeSelection.collaborationMode
  )
  const roleModeLabel = roleModeChoice
    ? translateOrFallback(
        t,
        `agentChat.composer.runtime.roleModes.${runtimeSelection.collaborationMode}.label`,
        roleModeChoice.label
      )
    : runtimeSelection.collaborationMode
  const routingChoice = runtimeOptions?.routingModes.find(
    (mode) => mode.id === runtimeSelection.routingMode
  )
  const routingLabel = routingChoice
    ? translateOrFallback(
        t,
        `agentChat.composer.runtime.routingModes.${runtimeSelection.routingMode}.label`,
        routingChoice.label
      )
    : runtimeSelection.routingMode
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
    const role = runtimeOptions?.roles.find(
      (role) => role.runtimeRole === runtimeRole
    )
    return role
      ? translateOrFallback(
          t,
          `agentChat.composer.runtime.roleLabels.${runtimeRole}.name`,
          role.name
        )
      : runtimeRole
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

function translateOrFallback(
  t: TFunction,
  key: string,
  fallback: string
) {
  const translated = t(key)
  return translated === key ? fallback : translated
}
