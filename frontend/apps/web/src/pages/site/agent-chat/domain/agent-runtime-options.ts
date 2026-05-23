import {
  ensureManualRoleTeam,
  manualSupportFallbackRoles,
  orderDefaultCommerceTeamRoles,
  preferredRuntimePrimaryRole,
  selectedSupportRoleCount,
  uniqueRuntimeRoles,
} from "./agent-runtime-role-utils"
import { normalizeRoutingMode, type AgentRoutingMode } from "./agent-routing-mode"

export interface AgentRuntimeChoice {
  id: string
  label: string
  description: string
  isDefault: boolean
}

export interface AgentRuntimeRoleOption {
  id: string
  name: string
  runtimeRole: string
  capabilities: string[]
  isDefault: boolean
}

export interface AgentRuntimeOptions {
  collaborationModes: AgentRuntimeChoice[]
  learningLevels: AgentRuntimeChoice[]
  responseModes: AgentRuntimeChoice[]
  routingModes: AgentRuntimeChoice[]
  roles: AgentRuntimeRoleOption[]
  strategies: AgentRuntimeChoice[]
}

export interface AgentChatRuntimeSelection {
  collaborationMode: string
  learningLevel: string
  primaryRole?: string
  routingMode: AgentRoutingMode
  selectedRoles: string[]
}

export type AgentChatRuntimeSelectionRestore = Partial<
  Omit<AgentChatRuntimeSelection, "selectedRoles">
> & {
  selectedRoles?: string[]
}

export function defaultRuntimeSelection(options: AgentRuntimeOptions | null): AgentChatRuntimeSelection {
  const defaultCollaboration =
    options?.collaborationModes.find((mode) => mode.id === "manual")?.id ??
    options?.collaborationModes.find((mode) => mode.id === "auto")?.id ??
    options?.collaborationModes.find((mode) => mode.isDefault)?.id ??
    "manual"
  const defaultLearning =
    options?.learningLevels.find((level) => level.isDefault)?.id ??
    "higher_vocational"
  const defaultRouting = normalizeRoutingMode(
    options?.routingModes.find((mode) => mode.isDefault)?.id
  )
  const defaultRoles = orderDefaultCommerceTeamRoles(
    options?.roles
      .filter((role) => role.runtimeRole !== "chat")
      .map((role) => role.runtimeRole) ?? ["ops", "data", "creative"]
  )

  return {
    collaborationMode: defaultCollaboration,
    learningLevel: defaultLearning,
    primaryRole: preferredRuntimePrimaryRole(defaultRoles),
    routingMode: defaultRouting,
    selectedRoles: defaultRoles,
  }
}

export function normalizeRuntimeSelection(
  current: AgentChatRuntimeSelection,
  options: AgentRuntimeOptions | null
): AgentChatRuntimeSelection {
  if (!options) return current
  const roleNames = new Set(options.roles.map((role) => role.runtimeRole))
  const collaborationModes = new Set(
    options.collaborationModes.map((mode) => mode.id)
  )
  const learningLevels = new Set(options.learningLevels.map((level) => level.id))
  const fallback = defaultRuntimeSelection(options)
  const selectedRoles = current.selectedRoles.filter((role) =>
    roleNames.has(role)
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
    primaryRole,
    routingMode: normalizeRoutingMode(current.routingMode),
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
    primaryRole: restored.primaryRole ?? current.primaryRole,
    routingMode: normalizeRoutingMode(restored.routingMode ?? current.routingMode),
    selectedRoles:
      restored.selectedRoles && restored.selectedRoles.length > 0
        ? restored.selectedRoles
        : current.selectedRoles,
  }

  return normalizeRuntimeSelection(nextSelection, options)
}

export function setRuntimeRoutingMode(
  selection: AgentChatRuntimeSelection,
  routingMode: string
): AgentChatRuntimeSelection {
  return {
    ...selection,
    routingMode: normalizeRoutingMode(routingMode),
  }
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
  const nonChatRoles = roles.filter((role) => role !== "chat")
  if (nonChatRoles.length === 0) {
    return {
      ...selection,
      primaryRole: preferredRuntimePrimaryRole(roles),
      selectedRoles: roles.length ? roles : [],
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
    return {
      ...selection,
      primaryRole,
      selectedRoles: uniqueRuntimeRoles([
        primaryRole,
        ...selection.selectedRoles.filter((role) => role !== primaryRole),
        ...manualSupportFallbackRoles(primaryRole),
      ]).slice(0, 3),
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
      selectedRoles: uniqueRuntimeRoles([
        primaryRole,
        "ops",
        "data",
        "creative",
      ]).slice(0, 3),
    }
  }

  const roles = uniqueRuntimeRoles(selection.selectedRoles).filter(
    (role) => role !== "chat"
  )
  const fallbackRoles = uniqueRuntimeRoles([
    selection.primaryRole ?? "",
    ...roles,
    "ops",
    "data",
    "creative",
  ])
  const selectedRoles = roles.length >= 2 ? roles : fallbackRoles.slice(0, 3)
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
