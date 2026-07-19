type CommerceTranslator = (key: string) => string

export const DEFAULT_WORKSPACE_TYPE = "general"

export const WORKSPACE_TYPE_DEFINITIONS = [
  {
    aliases: [],
    labelKey: "commerce.workspaces.typeOptions.general",
    value: "general",
  },
  {
    aliases: [],
    labelKey: "commerce.workspaces.typeOptions.agentChat",
    value: "agent_chat",
  },
  {
    aliases: ["product"],
    labelKey: "commerce.workspaces.typeOptions.productAnalysis",
    value: "product_analysis",
  },
  {
    aliases: ["campaign"],
    labelKey: "commerce.workspaces.typeOptions.campaignPlanning",
    value: "campaign_planning",
  },
  {
    aliases: ["operations"],
    labelKey: "commerce.workspaces.typeOptions.dataDiagnosis",
    value: "data_diagnosis",
  },
] as const

export type WorkspaceTypeValue =
  (typeof WORKSPACE_TYPE_DEFINITIONS)[number]["value"]

const workspaceTypeLabelKeyByValue = WORKSPACE_TYPE_DEFINITIONS.reduce<
  Record<string, string>
>((lookup, definition) => {
  lookup[definition.value] = definition.labelKey
  definition.aliases.forEach((alias) => {
    lookup[alias] = definition.labelKey
  })

  return lookup
}, {})

export function workspaceTypeOptions(t: CommerceTranslator) {
  return WORKSPACE_TYPE_DEFINITIONS.map((definition) => ({
    label: t(definition.labelKey),
    value: definition.value,
  }))
}

export function getWorkspaceTypeLabel(
  t: CommerceTranslator,
  value: string | null | undefined
) {
  const normalizedValue = value || DEFAULT_WORKSPACE_TYPE
  const labelKey = workspaceTypeLabelKeyByValue[normalizedValue]

  return labelKey ? t(labelKey) : normalizedValue.replace(/[_-]+/g, " ")
}
