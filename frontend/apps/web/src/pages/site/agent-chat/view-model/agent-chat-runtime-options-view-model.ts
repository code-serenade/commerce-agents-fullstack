import type {
  AgentRuntimeChoice,
  AgentRuntimeRoleOption,
} from "../domain/agent-runtime-options"

const ROLE_MODE_ORDER = ["auto", "single", "manual"]
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

export function choiceToOption(choice: AgentRuntimeChoice) {
  return {
    description: choice.description,
    label: choice.label,
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

function normalizedOrder(index: number) {
  return index === -1 ? Number.MAX_SAFE_INTEGER : index
}
