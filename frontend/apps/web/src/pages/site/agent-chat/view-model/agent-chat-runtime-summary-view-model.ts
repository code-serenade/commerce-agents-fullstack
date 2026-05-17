import type { TFunction } from "i18next"
import type {
  AgentChatRuntimeSelection,
  AgentRuntimeOptions,
} from "../domain/agent-runtime-options"
import type { AgentChatStrategyId } from "../domain/agent-chat-strategy"

export function buildCompactRuntimeSummary({
  depthLabel,
  roleModeLabel,
  runtimeOptions,
  runtimeSelection,
  strategyId,
  t,
}: {
  depthLabel: string
  roleModeLabel: string
  runtimeOptions: AgentRuntimeOptions | null
  runtimeSelection: AgentChatRuntimeSelection
  strategyId: AgentChatStrategyId
  t: TFunction
}) {
  const strategyShort = t(`agentChat.strategy.options.${strategyId}.shortHint`)
  const modelLabel = shortModelName(runtimeSelection.modelName)
  const roleLabel = selectedRoleSummary({
    runtimeOptions,
    runtimeSelection,
    t,
  })

  return [strategyShort, roleModeLabel, depthLabel, modelLabel, roleLabel]
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

export function shortModelName(modelName: string | undefined) {
  if (!modelName) return ""
  if (/doubao/i.test(modelName)) return "Doubao"
  return modelName.replace(/^llm[._-]/i, "").replace(/[-_]?model$/i, "")
}
