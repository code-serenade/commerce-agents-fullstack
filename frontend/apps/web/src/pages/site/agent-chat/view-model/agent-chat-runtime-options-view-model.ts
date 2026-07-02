import type {
  AgentRuntimeChoice,
  AgentRuntimeRoleOption,
} from "../domain/agent-runtime-options"

const ROLE_MODE_ORDER = ["auto", "single", "manual"]
const CHOICE_COPY_KEYS = {
  learningLevels: {
    beginner: {
      descriptionKey:
        "agentChat.composer.runtime.learningLevels.beginner.description",
      labelKey: "agentChat.composer.runtime.learningLevels.beginner.label",
    },
    higher_vocational: {
      descriptionKey:
        "agentChat.composer.runtime.learningLevels.higher_vocational.description",
      labelKey:
        "agentChat.composer.runtime.learningLevels.higher_vocational.label",
    },
    professional: {
      descriptionKey:
        "agentChat.composer.runtime.learningLevels.professional.description",
      labelKey: "agentChat.composer.runtime.learningLevels.professional.label",
    },
  },
  routingModes: {
    commander: {
      descriptionKey:
        "agentChat.composer.runtime.routingModes.commander.description",
      labelKey: "agentChat.composer.runtime.routingModes.commander.label",
    },
    fast: {
      descriptionKey: "agentChat.composer.runtime.routingModes.fast.description",
      labelKey: "agentChat.composer.runtime.routingModes.fast.label",
    },
  },
} as const
type ChoiceCopyKey = {
  descriptionKey: string
  labelKey: string
}
const ROLE_MODE_COPY_KEYS = {
  auto: {
    descriptionKey: "agentChat.composer.runtime.roleModes.auto.description",
    labelKey: "agentChat.composer.runtime.roleModes.auto.label",
  },
  manual: {
    descriptionKey: "agentChat.composer.runtime.roleModes.manual.description",
    labelKey: "agentChat.composer.runtime.roleModes.manual.label",
  },
  single: {
    descriptionKey: "agentChat.composer.runtime.roleModes.single.description",
    labelKey: "agentChat.composer.runtime.roleModes.single.label",
  },
} as const

export function sortRoleModes(choices: AgentRuntimeChoice[]) {
  return [...choices].sort((left, right) => {
    const leftIndex = ROLE_MODE_ORDER.indexOf(left.id)
    const rightIndex = ROLE_MODE_ORDER.indexOf(right.id)
    return normalizedOrder(leftIndex) - normalizedOrder(rightIndex)
  })
}

export function choiceToOption(
  choice: AgentRuntimeChoice,
  t?: (key: string) => string,
  group?: keyof typeof CHOICE_COPY_KEYS
) {
  const copyKeys: Record<string, ChoiceCopyKey> | undefined = group
    ? CHOICE_COPY_KEYS[group]
    : undefined
  const copyKey = copyKeys?.[choice.id]
  return {
    description: copyKey && t ? t(copyKey.descriptionKey) : choice.description,
    label: copyKey && t ? t(copyKey.labelKey) : choice.label,
    value: choice.id,
  }
}

export function roleModeToOption(
  choice: AgentRuntimeChoice,
  t: (key: string) => string
) {
  const copyKey =
    ROLE_MODE_COPY_KEYS[choice.id as keyof typeof ROLE_MODE_COPY_KEYS]

  return {
    description: copyKey ? t(copyKey.descriptionKey) : choice.description,
    label: copyKey ? t(copyKey.labelKey) : choice.label,
    value: choice.id,
  }
}

export function choiceToSelectOption(choice: AgentRuntimeChoice) {
  return {
    label: choice.label,
    value: choice.id,
  }
}

export function roleToSelectOption(role: AgentRuntimeRoleOption) {
  return {
    label: role.name,
    value: role.runtimeRole,
  }
}

export function localizedRoleOption(
  role: AgentRuntimeRoleOption,
  t: (key: string) => string
): AgentRuntimeRoleOption {
  const displayRuntimeRole = normalizeRoleLabelRuntimeRole(role.runtimeRole)
  return {
    ...role,
    capabilities: role.capabilities.map((capability) =>
      translateOrFallback(
        t,
        `agentChat.composer.runtime.capabilityLabels.${capability}`,
        capability
      )
    ),
    name: translateOrFallback(
      t,
      `agentChat.composer.runtime.roleLabels.${displayRuntimeRole}.name`,
      role.name
    ),
  }
}

function normalizeRoleLabelRuntimeRole(runtimeRole: string) {
  const normalized = runtimeRole.toLowerCase()
  if (normalized.includes("douyin") || normalized.includes("qianchuan")) {
    return "ops"
  }
  return runtimeRole
}

function translateOrFallback(
  t: (key: string) => string,
  key: string,
  fallback: string
) {
  const translated = t(key)
  return translated === key ? fallback : translated
}

function normalizedOrder(index: number) {
  return index === -1 ? Number.MAX_SAFE_INTEGER : index
}
