import type { AgentContextPreviewResponse } from "../agent-chat-data"

export type AgentContextPreviewReadiness =
  | "empty"
  | "partial"
  | "ready"
  | "unknown"

export interface AgentContextPreviewAuditSummary {
  budget: {
    inputChars: number
    totalContextChars: number
  }
  conversation: {
    hasHistory: boolean
    historyChars: number
    memoryChars: number
    messageCount: number
    relevantOlderCount: number
    totalMessageCount: number
    truncated: boolean
  }
  conversationMemory: {
    available: boolean
    constraintCount: number
    decisionCount: number
    factCount: number
    openQuestionCount: number
    summaryAvailable: boolean
  }
  workspace: {
    missingCount: number
    readiness: AgentContextPreviewReadiness
    sourceCount: number
    warningCount: number
  }
  workspacePack: {
    charCount: number
    itemCount: number
    omittedCount: number
    readiness: AgentContextPreviewReadiness
    readableItemCount: number
    warningCount: number
  }
}

export function createEmptyAgentContextPreviewAuditSummary(): AgentContextPreviewAuditSummary {
  return {
    budget: {
      inputChars: 0,
      totalContextChars: 0,
    },
    conversation: {
      hasHistory: false,
      historyChars: 0,
      memoryChars: 0,
      messageCount: 0,
      relevantOlderCount: 0,
      totalMessageCount: 0,
      truncated: false,
    },
    conversationMemory: {
      available: false,
      constraintCount: 0,
      decisionCount: 0,
      factCount: 0,
      openQuestionCount: 0,
      summaryAvailable: false,
    },
    workspace: {
      missingCount: 0,
      readiness: "empty",
      sourceCount: 0,
      warningCount: 0,
    },
    workspacePack: {
      charCount: 0,
      itemCount: 0,
      omittedCount: 0,
      readiness: "empty",
      readableItemCount: 0,
      warningCount: 0,
    },
  }
}

export function createAgentContextPreviewAuditSummary(
  preview: AgentContextPreviewResponse
): AgentContextPreviewAuditSummary {
  const budget = getRecord(preview.budget)
  const conversationHistory = getRecord(preview.conversationHistory)
  const conversationMemory = getRecord(preview.conversationMemory)
  const workspaceContext = getRecord(preview.workspaceContext)
  const workspacePack = getRecord(preview.workspacePack)
  const historyChars = readNumber(budget, "conversationHistoryChars")
  const memoryChars = readNumber(budget, "conversationMemoryChars")
  const messageCount =
    readNumber(conversationHistory, "messageCount", "message_count") ||
    readNumber(budget, "historyMessageCount")
  const totalMessageCount =
    readNumber(conversationHistory, "totalMessageCount", "total_message_count") ||
    readNumber(budget, "historyTotalMessageCount") ||
    messageCount
  const workspacePackCharCount =
    readOptionalNumber(workspacePack, "textChars", "text_chars") ??
    readOptionalNumber(budget, "workspacePackChars") ??
    sumWorkspacePackTextChars(preview.workspacePack)
  const workspacePackItemCount =
    readOptionalNumber(workspacePack, "itemCount", "item_count") ??
    readOptionalNumber(budget, "workspacePackItems") ??
    (preview.workspacePack.items ?? []).length
  const workspacePackReadableItemCount =
    readOptionalNumber(
      workspacePack,
      "readableItemCount",
      "readable_item_count"
    ) ?? getWorkspacePackReadableCount(preview.workspacePack)
  const workspaceContextSourceCount =
    readOptionalNumber(workspaceContext, "sourceCount", "source_count") ??
    getWorkspaceContextSourceCount(preview.workspaceContext)
  const memoryConstraintCount =
    readOptionalNumber(conversationMemory, "constraintCount", "constraint_count") ??
    getArrayLength(conversationMemory?.constraints)
  const memoryDecisionCount =
    readOptionalNumber(conversationMemory, "decisionCount", "decision_count") ??
    getArrayLength(conversationMemory?.decisions)
  const memoryFactCount =
    readOptionalNumber(conversationMemory, "factCount", "fact_count") ??
    getArrayLength(conversationMemory?.facts)
  const memoryOpenQuestionCount =
    readOptionalNumber(
      conversationMemory,
      "openQuestionCount",
      "open_question_count"
    ) ??
    getArrayLength(
      conversationMemory?.openQuestions ?? conversationMemory?.open_questions
    )
  const memorySummaryAvailable =
    readOptionalBoolean(
      conversationMemory,
      "rollingSummaryAvailable",
      "rolling_summary_available"
    ) ??
    readNonEmptyString(conversationMemory, "rollingSummary", "rolling_summary")

  return {
    budget: {
      inputChars: readNumber(budget, "inputChars"),
      totalContextChars: historyChars + memoryChars + workspacePackCharCount,
    },
    conversation: {
      hasHistory:
        Boolean(conversationHistory) ||
        historyChars > 0 ||
        memoryChars > 0 ||
        totalMessageCount > 0,
      historyChars,
      memoryChars,
      messageCount,
      relevantOlderCount: readNumber(
        conversationHistory,
        "relevantOlderCount",
        "relevant_older_count"
      ),
      totalMessageCount,
      truncated:
        readBoolean(conversationHistory, "truncated") ||
        readBoolean(budget, "historyTruncated"),
    },
    conversationMemory: {
      available: Boolean(conversationMemory) || memoryChars > 0,
      constraintCount: memoryConstraintCount,
      decisionCount: memoryDecisionCount,
      factCount: memoryFactCount,
      openQuestionCount: memoryOpenQuestionCount,
      summaryAvailable: memorySummaryAvailable,
    },
    workspace: {
      missingCount: getArrayLength(preview.workspaceContext.missing),
      readiness: normalizeReadiness(preview.workspaceContext.readiness),
      sourceCount: workspaceContextSourceCount,
      warningCount: getArrayLength(preview.workspaceContext.warnings),
    },
    workspacePack: {
      charCount: workspacePackCharCount,
      itemCount: workspacePackItemCount,
      omittedCount: getWorkspacePackOmittedCount(preview.workspacePack),
      readiness: normalizeReadiness(preview.workspacePack.readiness),
      readableItemCount: workspacePackReadableItemCount,
      warningCount: getArrayLength(preview.workspacePack.warnings),
    },
  }
}

function getRecord(value: unknown): Record<string, unknown> | null {
  if (!value || typeof value !== "object" || Array.isArray(value)) {
    return null
  }
  return value as Record<string, unknown>
}

function readNumber(
  source: Record<string, unknown> | null,
  ...keys: string[]
): number {
  return readOptionalNumber(source, ...keys) ?? 0
}

function readOptionalNumber(
  source: Record<string, unknown> | null,
  ...keys: string[]
): number | null {
  for (const key of keys) {
    const value = source?.[key]
    if (typeof value === "number" && Number.isFinite(value)) {
      return Math.max(value, 0)
    }
    if (typeof value === "string") {
      const parsed = Number(value.trim())
      if (Number.isFinite(parsed)) {
        return Math.max(parsed, 0)
      }
    }
  }
  return null
}

function readBoolean(
  source: Record<string, unknown> | null,
  ...keys: string[]
): boolean {
  return readOptionalBoolean(source, ...keys) ?? false
}

function readOptionalBoolean(
  source: Record<string, unknown> | null,
  ...keys: string[]
): boolean | null {
  for (const key of keys) {
    const value = source?.[key]
    if (typeof value === "boolean") return value
  }
  return null
}

function readNonEmptyString(
  source: Record<string, unknown> | null,
  ...keys: string[]
) {
  return keys.some((key) => {
    const value = source?.[key]
    return typeof value === "string" && value.trim().length > 0
  })
}

function normalizeReadiness(
  readiness: string | null | undefined
): AgentContextPreviewReadiness {
  if (readiness === "ready" || readiness === "partial" || readiness === "empty") {
    return readiness
  }
  return "unknown"
}

function getWorkspaceContextSourceCount(
  context: AgentContextPreviewResponse["workspaceContext"]
) {
  const counts = getRecord(context.counts)
  const countFromSummary =
    readNumber(counts, "conversationFiles", "conversation_files") +
    readNumber(counts, "campaigns") +
    readNumber(counts, "productAssets", "product_assets") +
    readNumber(counts, "productMaterials", "product_materials") +
    readNumber(counts, "workspaceArtifacts", "workspace_artifacts")
  if (countFromSummary > 0) return countFromSummary

  return (
    getArrayLength(context.attachmentSources ?? context.attachment_sources) +
    getArrayLength(context.campaignSources ?? context.campaign_sources) +
    getArrayLength(context.productAssetSources ?? context.product_asset_sources) +
    getArrayLength(
      context.productMaterialSources ?? context.product_material_sources
    ) +
    getArrayLength(context.artifactSources ?? context.artifact_sources)
  )
}

function getWorkspacePackReadableCount(
  pack: AgentContextPreviewResponse["workspacePack"]
) {
  return (pack.items ?? []).filter((item) => {
    const textChars = item.textChars ?? item.text_chars ?? 0
    return textChars > 0 || Boolean(item.text)
  }).length
}

function getWorkspacePackOmittedCount(
  pack: AgentContextPreviewResponse["workspacePack"]
) {
  const omittedDetailsCount = getArrayLength(
    pack.omittedDetails ?? pack.omitted_details
  )
  if (omittedDetailsCount > 0) return omittedDetailsCount
  return getArrayLength(pack.omitted)
}

function sumWorkspacePackTextChars(
  pack: AgentContextPreviewResponse["workspacePack"]
) {
  return (pack.items ?? []).reduce(
    (total, item) => total + Math.max(item.textChars ?? item.text_chars ?? 0, 0),
    0
  )
}

function getArrayLength(value: unknown) {
  return Array.isArray(value) ? value.length : 0
}
