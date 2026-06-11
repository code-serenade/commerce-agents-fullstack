import type { AgentContextPreviewResponse } from "../agent-chat-data"
import {
  createAgentContextPreviewAuditSummary,
  createEmptyAgentContextPreviewAuditSummary,
  type AgentContextPreviewAuditSummary,
} from "./agent-context-preview-audit-summary-model"

export type {
  AgentContextPreviewAuditSummary,
  AgentContextPreviewReadiness,
} from "./agent-context-preview-audit-summary-model"

export type AgentContextPreviewRefStatus =
  | "included"
  | "metadataOnly"
  | "omitted"

export interface AgentContextPreviewMemoryRow {
  charCount: number
  id: string
  kind: string
  name: string
  tags: string[]
  truncated: boolean
}

export interface AgentContextPreviewRefRow {
  id: string
  ref: string
  status: AgentContextPreviewRefStatus
}

export interface AgentContextPreviewModel {
  auditSummary: AgentContextPreviewAuditSummary
  hasPreview: boolean
  hiddenRefCount: number
  memoryRows: AgentContextPreviewMemoryRow[]
  readCounts: {
    included: number
    memory: number
    metadataOnly: number
    omitted: number
  }
  refRows: AgentContextPreviewRefRow[]
  warnings: string[]
}

export function getAgentContextPreviewModel(
  preview: AgentContextPreviewResponse | null,
  {
    memoryLimit = 4,
    refLimit = 8,
    warningLimit = 4,
  }: {
    memoryLimit?: number
    refLimit?: number
    warningLimit?: number
  } = {}
): AgentContextPreviewModel {
  if (!preview) {
    return {
      auditSummary: createEmptyAgentContextPreviewAuditSummary(),
      hasPreview: false,
      hiddenRefCount: 0,
      memoryRows: [],
      readCounts: {
        included: 0,
        memory: 0,
        metadataOnly: 0,
        omitted: 0,
      },
      refRows: [],
      warnings: [],
    }
  }

  const allRefRows = [
    ...preview.includedRefs.map((ref) => createRefRow(ref, "included")),
    ...preview.metadataOnlyRefs.map((ref) =>
      createRefRow(ref, "metadataOnly")
    ),
    ...preview.omittedRefs.map((ref) => createRefRow(ref, "omitted")),
  ]
  const refRows = allRefRows.slice(0, refLimit)

  return {
    auditSummary: createAgentContextPreviewAuditSummary(preview),
    hasPreview: true,
    hiddenRefCount: Math.max(allRefRows.length - refRows.length, 0),
    memoryRows: preview.memoryRecords.slice(0, memoryLimit).map(
      (record, index): AgentContextPreviewMemoryRow => {
        const key = record.key?.trim()
        const kind = record.kind.trim() || "memory"
        return {
          charCount: Math.max(record.charCount ?? record.char_count ?? 0, 0),
          id: `${kind}:${key || index}`,
          kind,
          name: key || kind,
          tags: normalizeTags(record.tags).slice(0, 2),
          truncated: record.truncated,
        }
      }
    ),
    readCounts: {
      included: preview.includedRefs.length,
      memory: preview.memoryRecords.length,
      metadataOnly: preview.metadataOnlyRefs.length,
      omitted: preview.omittedRefs.length,
    },
    refRows,
    warnings: preview.warnings
      .map((warning) => warning.trim())
      .filter(Boolean)
      .slice(0, warningLimit),
  }
}

function createRefRow(
  ref: string,
  status: AgentContextPreviewRefStatus
): AgentContextPreviewRefRow {
  return {
    id: `${status}:${ref}`,
    ref,
    status,
  }
}

function normalizeTags(tags: string[]) {
  return tags.map((tag) => tag.trim()).filter(Boolean)
}
