import type { TrainingFileItem } from "../types"
import type { AgentChatSourceContextSelection } from "../../../agent-chat/domain/agent-chat-context-selection"
import type { TrainingFileUsageFilter } from "./usage-filter"

export type TrainingFileKindFilter =
  | "all"
  | "image"
  | "pdf"
  | "word"
  | "sheet"
  | "text"
  | "other"

export type TrainingFileReadStatusFilter =
  | "all"
  | "readable"
  | "pending"
  | "failed"
  | "unsupported"

export interface TrainingFileFilters {
  keyword: string
  kind: TrainingFileKindFilter
  readStatus: TrainingFileReadStatusFilter
  usage?: TrainingFileUsageFilter
}

export function filterTrainingFiles(
  files: TrainingFileItem[],
  filters: string | TrainingFileFilters
) {
  const normalizedFilters = typeof filters === "string"
    ? { keyword: filters, kind: "all", readStatus: "all" } satisfies TrainingFileFilters
    : filters
  const normalizedKeyword = normalizedFilters.keyword.trim().toLowerCase()

  return files.filter((file) =>
    matchesFileKind(file, normalizedFilters.kind) &&
    matchesReadStatus(file, normalizedFilters.readStatus) &&
    matchesKeyword(file, normalizedKeyword)
  )
}

export function hasActiveTrainingFileFilters(filters: TrainingFileFilters) {
  return (
    filters.keyword.trim().length > 0 ||
    filters.kind !== "all" ||
    filters.readStatus !== "all" ||
    filters.usage !== undefined && filters.usage !== "all"
  )
}

export function fileKind(file: TrainingFileItem): Exclude<TrainingFileKindFilter, "all"> {
  const mimeType = file.mimeType?.toLowerCase() ?? ""
  const name = (file.fileName || file.objectKey).toLowerCase()
  if (mimeType.startsWith("image/") || /\.(png|jpe?g|webp|gif|bmp|svg|heic)$/.test(name)) {
    return "image"
  }
  if (mimeType === "application/pdf" || /\.pdf$/.test(name)) {
    return "pdf"
  }
  if (
    mimeType.includes("word") ||
    mimeType === "application/msword" ||
    /\.(docx?|rtf)$/.test(name)
  ) {
    return "word"
  }
  if (
    mimeType === "text/csv" ||
    mimeType.includes("spreadsheet") ||
    mimeType.includes("ms-excel") ||
    /\.(xlsx?|csv)$/.test(name)
  ) {
    return "sheet"
  }
  if (
    mimeType.startsWith("text/") ||
    ["application/json", "application/xml"].includes(mimeType) ||
    /\.(txt|md|json|xml)$/.test(name)
  ) {
    return "text"
  }
  return "other"
}

function matchesKeyword(file: TrainingFileItem, normalizedKeyword: string) {
  if (!normalizedKeyword) return true
  return (
    [
      file.fileName,
      file.mimeType,
      file.objectKey,
      file.status,
      file.createdAt,
    ]
      .filter(Boolean)
      .some((value) => String(value).toLowerCase().includes(normalizedKeyword))
  )
}

function matchesFileKind(file: TrainingFileItem, kind: TrainingFileKindFilter) {
  return kind === "all" || fileKind(file) === kind
}

function matchesReadStatus(
  file: TrainingFileItem,
  readStatus: TrainingFileReadStatusFilter
) {
  return readStatus === "all" || fileReadableState(file) === readStatus
}

export function fileDisplayName(file: TrainingFileItem) {
  return file.fileName || file.objectKey.split("/").at(-1) || file.id
}

export function fileReadableState(file: TrainingFileItem) {
  const extraction = metadataRecord(file.metadata, "extraction")
  const status =
    metadataString(file.metadata, "extractionStatus") ||
    (extraction ? metadataString(extraction, "status") : undefined)
  if (status === "extracted" || status === "ready") return "readable"
  if (status === "failed") return "failed"
  if (status === "unsupported") return "unsupported"
  return "pending"
}

export function formatFileSize(size: number | null | undefined) {
  if (!size || size <= 0) return "-"
  if (size < 1024) return `${size} B`
  if (size < 1024 * 1024) return `${(size / 1024).toFixed(1)} KB`
  return `${(size / 1024 / 1024).toFixed(1)} MB`
}

export function fileObjectKey(file: TrainingFileItem) {
  return `${file.bucket}:${file.objectKey}`
}

export type FileConversationUseState =
  | "attached"
  | "available"
  | "selectedForTurn"

export interface TrainingFilesUsageSummary {
  attached: number
  readable: number
  selectedForTurn: number
  total: number
}

export interface TrainingFilesDeleteImpact {
  attached: number
  hiddenNames: number
  readable: number
  selectedForTurn: number
  targetNames: string[]
  total: number
}

export interface TrainingFilesFilterSummary {
  active: boolean
  filtered: number
  total: number
}

export function fileConversationUseState({
  attachedObjectKeys,
  file,
  selectedSources,
}: {
  attachedObjectKeys: Set<string>
  file: TrainingFileItem
  selectedSources: AgentChatSourceContextSelection[]
}): FileConversationUseState {
  if (
    selectedSources.some(
      (source) =>
        source.id === file.id ||
        (source.bucket === file.bucket && source.objectKey === file.objectKey)
    )
  ) {
    return "selectedForTurn"
  }
  if (attachedObjectKeys.has(fileObjectKey(file))) return "attached"
  return "available"
}

export function summarizeTrainingFilesUsage({
  attachedObjectKeys,
  files,
  selectedSources,
}: {
  attachedObjectKeys: Set<string>
  files: TrainingFileItem[]
  selectedSources: AgentChatSourceContextSelection[]
}): TrainingFilesUsageSummary {
  return files.reduce<TrainingFilesUsageSummary>(
    (summary, file) => {
      const useState = fileConversationUseState({
        attachedObjectKeys,
        file,
        selectedSources,
      })

      summary.total += 1
      if (fileReadableState(file) === "readable") summary.readable += 1
      if (useState === "attached" || useState === "selectedForTurn") {
        summary.attached += 1
      }
      if (useState === "selectedForTurn") summary.selectedForTurn += 1

      return summary
    },
    {
      attached: 0,
      readable: 0,
      selectedForTurn: 0,
      total: 0,
    }
  )
}

export function summarizeTrainingFilesDeleteImpact({
  attachedObjectKeys,
  files,
  maxTargetNames = 3,
  selectedSources,
}: {
  attachedObjectKeys: Set<string>
  files: TrainingFileItem[]
  maxTargetNames?: number
  selectedSources: AgentChatSourceContextSelection[]
}): TrainingFilesDeleteImpact {
  const impact = files.reduce<TrainingFilesDeleteImpact>(
    (summary, file) => {
      const useState = fileConversationUseState({
        attachedObjectKeys,
        file,
        selectedSources,
      })

      summary.total += 1
      if (fileReadableState(file) === "readable") summary.readable += 1
      if (useState === "attached" || useState === "selectedForTurn") {
        summary.attached += 1
      }
      if (useState === "selectedForTurn") summary.selectedForTurn += 1
      if (summary.targetNames.length < maxTargetNames) {
        summary.targetNames.push(fileDisplayName(file))
      }

      return summary
    },
    {
      attached: 0,
      hiddenNames: 0,
      readable: 0,
      selectedForTurn: 0,
      targetNames: [],
      total: 0,
    }
  )
  impact.hiddenNames = Math.max(impact.total - impact.targetNames.length, 0)
  return impact
}

export function summarizeTrainingFilesFilterResult({
  filters,
  filteredFiles,
  totalFiles,
}: {
  filters: TrainingFileFilters
  filteredFiles: TrainingFileItem[]
  totalFiles: TrainingFileItem[]
}): TrainingFilesFilterSummary {
  return {
    active: hasActiveTrainingFileFilters(filters),
    filtered: filteredFiles.length,
    total: totalFiles.length,
  }
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
