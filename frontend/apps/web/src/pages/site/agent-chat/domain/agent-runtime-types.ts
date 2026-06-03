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
  models?: AgentRuntimeModelOption[]
  responseModes: AgentRuntimeChoice[]
  roles: AgentRuntimeRoleOption[]
  routingModes?: AgentRuntimeChoice[]
  strategies: AgentRuntimeChoice[]
}

export interface AgentChatRuntimeSelection {
  collaborationMode: string
  learningLevel: string
  modelName?: string
  primaryRole?: string
  routingMode?: string
  selectedRoles: string[]
}

export type AgentChatRuntimeSelectionRestore = Partial<
  Omit<AgentChatRuntimeSelection, "selectedRoles">
> & {
  selectedRoles?: string[]
}
