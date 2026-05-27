import type { TrainingFileItem } from "../types"

export interface TrainingFileReadSummary {
  contentKind?: string
  failureReason?: string
  parser?: string
  summary?: string
  textChars?: number
  truncated?: boolean
}

export function readTrainingFileSummary(
  file: TrainingFileItem
): TrainingFileReadSummary {
  const extraction = metadataRecord(file.metadata, "extraction")
  const extractionMetadata = extraction
    ? metadataRecord(extraction, "metadata")
    : undefined

  return {
    contentKind:
      metadataString(file.metadata, "contentKind") ||
      (extraction ? metadataString(extraction, "contentKind") : undefined),
    failureReason:
      metadataString(file.metadata, "failureReason") ||
      (extraction ? metadataString(extraction, "failureReason") : undefined),
    parser:
      metadataString(file.metadata, "parser") ||
      (extraction ? metadataString(extraction, "parser") : undefined),
    summary:
      metadataString(file.metadata, "summary") ||
      (extraction ? metadataString(extraction, "summary") : undefined) ||
      metadataString(file.metadata, "text"),
    textChars:
      metadataNumber(file.metadata, "textChars") ||
      (extraction ? metadataNumber(extraction, "textChars") : undefined) ||
      (extractionMetadata
        ? metadataNumber(extractionMetadata, "originalTextChars")
        : undefined) ||
      metadataNumber(file.metadata, "originalTextChars"),
    truncated:
      metadataBoolean(file.metadata, "truncated") ||
      (extractionMetadata
        ? metadataBoolean(extractionMetadata, "truncated")
        : undefined),
  }
}

function metadataBoolean(
  metadata: Record<string, unknown>,
  key: string
): boolean | undefined {
  const value = metadata[key]
  return typeof value === "boolean" ? value : undefined
}

function metadataNumber(
  metadata: Record<string, unknown>,
  key: string
): number | undefined {
  const value = metadata[key]
  return typeof value === "number" && Number.isFinite(value) ? value : undefined
}

function metadataString(
  metadata: Record<string, unknown>,
  key: string
): string | undefined {
  const value = metadata[key]
  return typeof value === "string" && value.trim() ? value : undefined
}

function metadataRecord(
  metadata: Record<string, unknown>,
  key: string
): Record<string, unknown> | undefined {
  const value = metadata[key]
  return typeof value === "object" && value !== null
    ? (value as Record<string, unknown>)
    : undefined
}
