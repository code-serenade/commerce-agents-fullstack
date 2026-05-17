import type {
  AgentRuntimeChoice,
  AgentRuntimeModelOption,
  AgentRuntimeRoleOption,
} from "../domain/agent-runtime-options"

const ROLE_MODE_ORDER = ["auto", "single", "manual"]

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

export function choiceToSelectOption(choice: AgentRuntimeChoice) {
  return {
    label: choice.label,
    value: choice.id,
  }
}

export function modelToSelectOption(model: AgentRuntimeModelOption) {
  return {
    label: `${model.name} / ${model.model}`,
    value: model.name,
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
