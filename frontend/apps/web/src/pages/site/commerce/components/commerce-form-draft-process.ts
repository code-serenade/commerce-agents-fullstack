import type { CommerceTranslate } from "./commerce-types"

export interface FormDraftProcessSource {
  agentContext?: Record<string, unknown>
  generationMode?: string
  stages?: string[]
}

export function buildFormDraftProcessNotice(
  source: FormDraftProcessSource | null,
  t: CommerceTranslate
): string | null {
  if (!source) return null

  const context = source.agentContext ?? {}
  const generationSource = readString(context, "generationSource") ?? source.generationMode
  const llm = readRecord(context, "llm")
  const draftQuality = readRecord(context, "draftQuality")
  const saveBoundary = readRecord(context, "saveBoundary")
  const lines: string[] = []
  const stages = source.stages ?? []

  if (generationSource === "agent_llm" || readBoolean(llm, "used")) {
    lines.push(t("commerce.common.smartFillProcess.agentRoute"))
  } else if (
    generationSource === "template_smart_fill" ||
    generationSource === "deterministic_template"
  ) {
    lines.push(t("commerce.common.smartFillProcess.templateRoute"))
  }

  if (readBoolean(saveBoundary, "requiresUserReview") || readBoolean(context, "requiresUserReview")) {
    lines.push(t("commerce.common.smartFillProcess.reviewRequired"))
  }

  if (readString(draftQuality, "invalidModelJsonPolicy") === "reject_before_form_fill") {
    lines.push(t("commerce.common.smartFillProcess.invalidJsonRejected"))
  }

  for (const stage of stages) {
    const line = processStageText(stage, t)
    if (line && !lines.includes(line)) lines.push(line)
  }

  return lines.length > 0 ? lines.join(t("commerce.common.smartFillProcess.separator")) : null
}

function processStageText(stage: string, t: CommerceTranslate): string | null {
  if (stage === "form_draft_started") {
    return t("commerce.common.smartFillProcess.streamStarted")
  }
  if (stage === "form_draft_input_preflighted") {
    return t("commerce.common.smartFillProcess.streamInputPreflighted")
  }
  if (stage === "form_draft_agent_started") {
    return t("commerce.common.smartFillProcess.streamAgentStarted")
  }
  if (stage === "form_draft_output_validation_started") {
    return t("commerce.common.smartFillProcess.streamValidationStarted")
  }
  if (stage === "form_draft_output_validation_passed") {
    return t("commerce.common.smartFillProcess.streamValidationPassed")
  }
  if (stage === "form_draft_review_ready") {
    return t("commerce.common.smartFillProcess.streamReviewReady")
  }
  if (stage === "form_draft_failed") {
    return t("commerce.common.smartFillProcess.streamFailed")
  }
  return null
}

function readRecord(source: Record<string, unknown>, key: string): Record<string, unknown> | null {
  const value = source[key]

  return value && typeof value === "object" && !Array.isArray(value)
    ? value as Record<string, unknown>
    : null
}

function readBoolean(source: Record<string, unknown> | null, key: string): boolean {
  return source?.[key] === true
}

function readString(source: Record<string, unknown> | null, key: string): string | null {
  const value = source?.[key]

  return typeof value === "string" ? value : null
}
