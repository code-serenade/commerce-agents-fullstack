import type { SimpleFormDraftResult } from "./commerce-form-actions"

export function processEventDraftResult({
  agentContext,
  generationMode,
  stage,
}: {
  agentContext?: Record<string, unknown>
  generationMode?: string
  stage: string
}): SimpleFormDraftResult {
  return {
    agentContext,
    generationMode,
    stages: [stage],
    values: {},
  }
}

export function mergeDraftProcessResult(
  current: SimpleFormDraftResult | null,
  next: SimpleFormDraftResult | null
): SimpleFormDraftResult | null {
  if (!next) return null

  return {
    ...current,
    ...next,
    stages: [...(current?.stages ?? []), ...(next.stages ?? [])],
    values: next.values,
  }
}
