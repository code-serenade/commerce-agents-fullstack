export interface AgentRuntimeChoice {
  id: string
  label: string
  description: string
  isDefault: boolean
}

export interface AgentRuntimeModelOption {
  name: string
  model: string
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
  defaultModelName?: string | null
  learningLevels: AgentRuntimeChoice[]
  models: AgentRuntimeModelOption[]
  responseModes: AgentRuntimeChoice[]
  roles: AgentRuntimeRoleOption[]
  strategies: AgentRuntimeChoice[]
}

export interface AgentChatRuntimeSelection {
  collaborationMode: string
  learningLevel: string
  modelName?: string
  primaryRole?: string
  selectedRoles: string[]
}

export type AgentChatRuntimeSelectionRestore = Partial<
  Omit<AgentChatRuntimeSelection, "selectedRoles">
> & {
  selectedRoles?: string[]
}

export function defaultRuntimeSelection(
  options: AgentRuntimeOptions | null
): AgentChatRuntimeSelection {
  const defaultModel =
    options?.defaultModelName ??
    options?.models.find((model) => model.isDefault)?.name ??
    options?.models[0]?.name
  const defaultCollaboration =
    options?.collaborationModes.find((mode) => mode.id === "auto")?.id ??
    options?.collaborationModes.find((mode) => mode.isDefault)?.id ??
    "auto"
  const defaultLearning =
    options?.learningLevels.find((level) => level.isDefault)?.id ??
    "higher_vocational"
  const defaultRoles = orderDefaultCommerceTeamRoles(
    options?.roles
      .filter((role) => role.runtimeRole !== "chat")
      .map((role) => role.runtimeRole) ?? ["ops", "data", "creative"]
  )

  return {
    collaborationMode: defaultCollaboration,
    learningLevel: defaultLearning,
    modelName: defaultModel ?? undefined,
    primaryRole: preferredRuntimePrimaryRole(defaultRoles),
    selectedRoles: defaultRoles,
  }
}

export function normalizeRuntimeSelection(
  current: AgentChatRuntimeSelection,
  options: AgentRuntimeOptions | null
): AgentChatRuntimeSelection {
  if (!options) return current
  const modelNames = new Set(options.models.map((model) => model.name))
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
    modelName:
      current.modelName && modelNames.has(current.modelName)
        ? current.modelName
        : fallback.modelName,
    primaryRole,
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
  const normalizedSelection = normalizeRoleRuntimeSelection(selection)
  return normalizedSelection
}

function normalizeRoleRuntimeSelection(
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
      : nonChatRoles.includes("ops")
        ? "ops"
        : nonChatRoles[0]

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
    selection.selectedRoles.length > 1
  ) {
    return selection
  }

  const exists = selection.selectedRoles.includes(runtimeRole)
  const selectedRoles = exists
    ? selection.selectedRoles.filter((role) => role !== runtimeRole)
    : [...selection.selectedRoles, runtimeRole]
  const nextRoles = selectedRoles.length ? selectedRoles : [runtimeRole]
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

function preferredRuntimePrimaryRole(roles: string[]) {
  const nonChatRoles = roles.filter((role) => role !== "chat")
  if (nonChatRoles.includes("ops")) return "ops"
  return nonChatRoles[0] ?? roles[0] ?? "chat"
}

function orderDefaultCommerceTeamRoles(roles: string[]) {
  const preferred = ["ops", "data", "creative", "service", "engineering"]
  const uniqueRoles = uniqueRuntimeRoles(roles)
  const ordered = preferred.filter((role) => uniqueRoles.includes(role))
  const rest = uniqueRoles.filter((role) => !ordered.includes(role))
  const team = [...ordered, ...rest].slice(0, 5)
  return team.length ? team : ["ops", "data", "creative"]
}

function uniqueRuntimeRoles(roles: string[]) {
  return roles.reduce<string[]>((unique, role) => {
    const normalized = role.trim()
    if (normalized && !unique.includes(normalized)) unique.push(normalized)
    return unique
  }, [])
}
