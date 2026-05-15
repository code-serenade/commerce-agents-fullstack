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
    options?.collaborationModes.find((mode) => mode.isDefault)?.id ?? "manual"
  const defaultLearning =
    options?.learningLevels.find((level) => level.isDefault)?.id ??
    "higher_vocational"
  const defaultRoles = options?.roles
    .filter((role) => role.runtimeRole !== "chat")
    .slice(0, 3)
    .map((role) => role.runtimeRole) ?? ["data", "ops", "creative"]

  return {
    collaborationMode: defaultCollaboration,
    learningLevel: defaultLearning,
    modelName: defaultModel ?? undefined,
    primaryRole: defaultRoles[0],
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
  selection: AgentChatRuntimeSelection,
  strategyId: string
): AgentChatRuntimeSelection {
  if (strategyId === "execution" || strategyId === "learning") {
    return {
      ...selection,
      collaborationMode: "single",
      primaryRole: "chat",
      selectedRoles: ["chat"],
    }
  }
  if (strategyId === "collab") {
    return {
      ...selection,
      collaborationMode: "manual",
    }
  }
  return selection
}

export function setRuntimePrimaryRole(
  selection: AgentChatRuntimeSelection,
  primaryRole: string
): AgentChatRuntimeSelection {
  return {
    ...selection,
    primaryRole,
    selectedRoles: selection.selectedRoles.includes(primaryRole)
      ? selection.selectedRoles
      : [primaryRole, ...selection.selectedRoles],
  }
}

export function toggleRuntimeRole(
  selection: AgentChatRuntimeSelection,
  runtimeRole: string
): AgentChatRuntimeSelection {
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
  const primaryRole = nextRoles.includes(selection.primaryRole ?? "")
    ? selection.primaryRole
    : nextRoles[0]

  return {
    ...selection,
    primaryRole,
    selectedRoles: nextRoles,
  }
}
