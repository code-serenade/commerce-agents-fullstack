import type {
  SimpleFormAuxiliaryAction,
  SimpleFormValues,
} from "./commerce-form"

export interface SimpleFormSampleAction {
  label: string
  values: SimpleFormValues
}

export interface SimpleFormDraftAction {
  disabled?: boolean
  label: string
  onClick: (
    values: SimpleFormValues,
    onProcess?: (result: SimpleFormDraftResult) => void
  ) => Promise<SimpleFormDraftResult>
}

export interface SimpleFormDraftResult {
  agentContext?: Record<string, unknown>
  generationMode?: string
  stages?: string[]
  values: SimpleFormValues
}

export function buildSimpleFormAuxiliaryActions({
  agentSmartFillAction,
  onDraftResult,
  sampleAction,
  setValues,
  smartFillAction,
  values,
}: {
  agentSmartFillAction?: SimpleFormDraftAction
  onDraftResult?: (result: SimpleFormDraftResult | null) => void
  sampleAction?: SimpleFormSampleAction
  setValues: (values: SimpleFormValues) => void
  smartFillAction?: SimpleFormDraftAction
  values: SimpleFormValues
}): SimpleFormAuxiliaryAction[] {
  const actions: SimpleFormAuxiliaryAction[] = []

  if (sampleAction) {
    actions.push({
      label: sampleAction.label,
      onClick: () => {
        onDraftResult?.(null)
        setValues({ ...values, ...sampleAction.values })
      },
    })
  }

  for (const action of [smartFillAction, agentSmartFillAction]) {
    if (!action) continue

    actions.push({
      disabled: action.disabled,
      label: action.label,
      onClick: async () => {
        const result = await action.onClick(values, onDraftResult ?? undefined)
        setValues({ ...values, ...result.values })
        onDraftResult?.(result)
      },
    })
  }

  return actions
}
