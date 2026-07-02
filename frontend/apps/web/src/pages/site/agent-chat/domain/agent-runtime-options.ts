import {
  ensureManualRoleTeam,
  limitRuntimeRoles,
  manualSupportFallbackRoles,
  orderDefaultCommerceTeamRoles,
  preferredRuntimePrimaryRole,
  selectedSupportRoleCount,
  uniqueRuntimeRoles,
} from "./agent-runtime-role-utils"
import { normalizeRuntimeOptions } from "./agent-runtime-option-normalization"
import type {
  AgentChatRuntimeSelection,
  AgentChatRuntimeSelectionRestore,
  AgentRuntimeOptions,
} from "./agent-runtime-types"

export type {
  AgentChatRuntimeSelection,
  AgentChatRuntimeSelectionRestore,
  AgentRuntimeChoice,
  AgentRuntimeModelOption,
  AgentRuntimeOptions,
  AgentRuntimeRoleOption,
} from "./agent-runtime-types"
export { normalizeRuntimeOptions } from "./agent-runtime-option-normalization"

export function defaultRuntimeSelection(options: AgentRuntimeOptions | null): AgentChatRuntimeSelection {
  const normalizedOptions = normalizeRuntimeOptions(options)
  const models = options?.models ?? []
  const routingModes = normalizedOptions?.routingModes ?? []
  const defaultModel =
    options?.defaultModelName ??
    models.find((model) => model.isDefault)?.name ??
    models[0]?.name
  const defaultCollaboration =
    normalizedOptions?.collaborationModes.find((mode) => mode.id === "auto")?.id ??
    normalizedOptions?.collaborationModes.find((mode) => mode.isDefault)?.id ??
    normalizedOptions?.collaborationModes.find((mode) => mode.id === "manual")?.id ??
    "auto"
  const defaultLearning =
    normalizedOptions?.learningLevels.find((level) => level.isDefault)?.id ??
    "higher_vocational"
  const defaultRouting =
    routingModes.find((mode) => mode.id === "commander")?.id ??
    routingModes.find((mode) => mode.isDefault)?.id ??
    routingModes[0]?.id ??
    "commander"
  const defaultRoles = orderDefaultCommerceTeamRoles(
    normalizedOptions?.roles
      .filter((role) => role.runtimeRole !== "chat")
      .map((role) => role.runtimeRole) ?? ["ops", "data", "creative"]
  )

  return {
    collaborationMode: defaultCollaboration,
    learningLevel: defaultLearning,
    modelName: defaultModel ?? undefined,
    primaryRole: preferredRuntimePrimaryRole(defaultRoles),
    routingMode: defaultRouting,
    selectedRoles: defaultCollaboration === "auto" ? [] : defaultRoles,
  }
}

export function normalizeRuntimeSelection(
  current: AgentChatRuntimeSelection,
  options: AgentRuntimeOptions | null
): AgentChatRuntimeSelection {
  const normalizedOptions = normalizeRuntimeOptions(options)
  if (!normalizedOptions) return current
  const modelNames = new Set((normalizedOptions.models ?? []).map((model) => model.name))
  const roleNames = new Set(normalizedOptions.roles.map((role) => role.runtimeRole))
  const collaborationModes = new Set(
    normalizedOptions.collaborationModes.map((mode) => mode.id)
  )
  const learningLevels = new Set(normalizedOptions.learningLevels.map((level) => level.id))
  const routingModes = new Set((normalizedOptions.routingModes ?? []).map((mode) => mode.id))
  const fallback = defaultRuntimeSelection(normalizedOptions)
  const selectedRoles = limitRuntimeRoles(
    current.selectedRoles.filter((role) => roleNames.has(role))
  )
  const primaryRole =
    current.primaryRole && selectedRoles.includes(current.primaryRole)
      ? current.primaryRole
      : selectedRoles[0] ?? fallback.primaryRole

  return {
    collaborationMode: collaborationModes.has(current.collaborationMode)
      ? current.collaborationMode
      : fallback.collaborationMode,
    learningLevel: learningLevels.has(current.learningLevel)
      ? current.learningLevel
      : fallback.learningLevel,
    modelName:
      current.modelName && modelNames.has(current.modelName)
        ? current.modelName
        : fallback.modelName,
    primaryRole,
    routingMode: current.routingMode && routingModes.has(current.routingMode)
      ? current.routingMode
      : fallback.routingMode,
    selectedRoles: selectedRoles.length ? selectedRoles : fallback.selectedRoles,
  }
}

export function restoreRuntimeSelection(
  current: AgentChatRuntimeSelection,
  restored: AgentChatRuntimeSelectionRestore,
  options: AgentRuntimeOptions | null
): AgentChatRuntimeSelection {
  const nextSelection: AgentChatRuntimeSelection = {
    ...current,
    collaborationMode: restored.collaborationMode ?? current.collaborationMode,
    learningLevel: restored.learningLevel ?? current.learningLevel,
    modelName: restored.modelName ?? current.modelName,
    primaryRole: restored.primaryRole ?? current.primaryRole,
    selectedRoles:
      restored.selectedRoles && restored.selectedRoles.length > 0
        ? restored.selectedRoles
        : current.selectedRoles,
  }

  return normalizeRuntimeSelection(nextSelection, options)
}

export function runtimeSelectionForStrategy(
  selection: AgentChatRuntimeSelection
): AgentChatRuntimeSelection {
  if (selection.collaborationMode === "auto") {
    return {
      ...selection,
      primaryRole: selection.primaryRole === "chat" ? "ops" : selection.primaryRole,
      selectedRoles: [],
    }
  }

  const roles = uniqueRuntimeRoles(selection.selectedRoles)
  const nonChatRoles = limitRuntimeRoles(roles.filter((role) => role !== "chat"))
  if (nonChatRoles.length === 0) {
    return {
      ...selection,
      primaryRole: preferredRuntimePrimaryRole(roles),
      selectedRoles: roles.length ? limitRuntimeRoles(roles) : [],
    }
  }

  const primaryRole =
    selection.primaryRole && selection.primaryRole !== "chat"
      ? selection.primaryRole
      : preferredRuntimePrimaryRole(nonChatRoles)

  return {
    ...selection,
    primaryRole,
    selectedRoles: nonChatRoles,
  }
}

export function setRuntimePrimaryRole(
  selection: AgentChatRuntimeSelection,
  primaryRole: string
): AgentChatRuntimeSelection {
  if (selection.collaborationMode === "single") {
    return {
      ...selection,
      primaryRole,
      selectedRoles: [primaryRole],
    }
  }

  if (selection.collaborationMode === "manual") {
    const selectedRoles = limitRuntimeRoles([
      primaryRole,
      ...selection.selectedRoles.filter((role) => role !== primaryRole),
    ])
    const manualRoles =
      selectedSupportRoleCount(selectedRoles, primaryRole) > 0
        ? selectedRoles
        : limitRuntimeRoles([
            ...selectedRoles,
            ...manualSupportFallbackRoles(primaryRole),
          ])
    return {
      ...selection,
      primaryRole,
      selectedRoles: manualRoles,
    }
  }

  return {
    ...selection,
    primaryRole,
    selectedRoles: selection.selectedRoles.includes(primaryRole)
      ? selection.selectedRoles
      : [primaryRole, ...selection.selectedRoles],
  }
}

export function setRuntimeCollaborationMode(
  selection: AgentChatRuntimeSelection,
  collaborationMode: string
): AgentChatRuntimeSelection {
  if (collaborationMode === "auto") {
    return {
      ...selection,
      collaborationMode,
      primaryRole: selection.primaryRole === "chat" ? "ops" : selection.primaryRole,
      selectedRoles: [],
    }
  }

  if (collaborationMode === "single") {
    const primaryRole =
      selection.primaryRole && selection.primaryRole !== "chat"
        ? selection.primaryRole
        : preferredRuntimePrimaryRole(selection.selectedRoles)
    return {
      ...selection,
      collaborationMode,
      primaryRole,
      selectedRoles: [primaryRole],
    }
  }

  if (selection.collaborationMode !== "manual") {
    const primaryRole =
      selection.primaryRole && selection.primaryRole !== "chat"
        ? selection.primaryRole
        : preferredRuntimePrimaryRole(selection.selectedRoles)

    return {
      ...selection,
      collaborationMode,
      primaryRole,
      selectedRoles: limitRuntimeRoles([
        primaryRole,
        "ops",
        "data",
        "creative",
      ]),
    }
  }

  const roles = limitRuntimeRoles(selection.selectedRoles).filter(
    (role) => role !== "chat"
  )
  const fallbackRoles = limitRuntimeRoles([
    selection.primaryRole ?? "",
    ...roles,
    "ops",
    "data",
    "creative",
  ])
  const selectedRoles = roles.length >= 2 ? roles : fallbackRoles
  const primaryRole =
    selection.primaryRole &&
    selection.primaryRole !== "chat" &&
    selectedRoles.includes(selection.primaryRole)
      ? selection.primaryRole
      : preferredRuntimePrimaryRole(selectedRoles)

  return {
    ...selection,
    collaborationMode,
    primaryRole,
    selectedRoles,
  }
}

export function toggleRuntimeRole(
  selection: AgentChatRuntimeSelection,
  runtimeRole: string
): AgentChatRuntimeSelection {
  if (selection.collaborationMode === "single") {
    return setRuntimePrimaryRole(selection, runtimeRole)
  }

  if (
    selection.collaborationMode === "manual" &&
    selection.primaryRole === runtimeRole &&
    selectedSupportRoleCount(selection.selectedRoles, selection.primaryRole) > 0
  ) {
    return selection
  }

  const exists = selection.selectedRoles.includes(runtimeRole)
  const selectedRoles = exists
    ? selection.selectedRoles.filter((role) => role !== runtimeRole)
    : [...selection.selectedRoles, runtimeRole]
  const primaryCandidate =
    selection.primaryRole && selection.primaryRole !== "chat"
      ? selection.primaryRole
      : runtimeRole
  const nextRoles =
    selection.collaborationMode === "manual"
      ? ensureManualRoleTeam(selectedRoles, primaryCandidate)
      : selectedRoles.length
        ? selectedRoles
        : [runtimeRole]
  const primaryRole =
    selection.primaryRole &&
    selection.primaryRole !== "chat" &&
    nextRoles.includes(selection.primaryRole)
      ? selection.primaryRole
      : preferredRuntimePrimaryRole(nextRoles)

  return {
    ...selection,
    primaryRole,
    selectedRoles: nextRoles,
  }
}

export function setRuntimeRoutingMode(
  selection: AgentChatRuntimeSelection,
  routingMode: string
): AgentChatRuntimeSelection {
  return { ...selection, routingMode }
}
