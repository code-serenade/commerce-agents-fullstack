// The production Nginx proxy uses its 1 MiB default request-body limit.
export const MAX_AI_READABLE_UPLOAD_BYTES = 1024 * 1024

export const AI_READABLE_FILE_ACCEPT = [
  ".png",
  ".jpg",
  ".jpeg",
  ".webp",
  ".pdf",
  ".docx",
  ".xls",
  ".xlsx",
  ".csv",
  ".txt",
  ".md",
  ".markdown",
  ".json",
  ".tsv",
  ".html",
  ".htm",
].join(",")

type UploadValidationErrorCode =
  | "empty"
  | "mimeMismatch"
  | "tooLarge"
  | "unsupportedFormat"

export interface UploadValidationError {
  code: UploadValidationErrorCode
  file: File
}

export interface UploadValidationResult {
  acceptedFiles: File[]
  errors: UploadValidationError[]
}

const ALLOWED_MIME_TYPES_BY_EXTENSION: Record<string, ReadonlySet<string>> = {
  csv: new Set([
    "application/csv",
    "application/vnd.ms-excel",
    "text/csv",
    "text/plain",
    "text/x-csv",
  ]),
  docx: new Set([
    "application/vnd.openxmlformats-officedocument.wordprocessingml.document",
  ]),
  htm: new Set(["application/xhtml+xml", "text/html", "text/plain"]),
  html: new Set(["application/xhtml+xml", "text/html", "text/plain"]),
  jpeg: new Set(["image/jpeg", "image/jpg", "image/pjpeg"]),
  jpg: new Set(["image/jpeg", "image/jpg", "image/pjpeg"]),
  json: new Set([
    "application/json",
    "application/ld+json",
    "text/json",
    "text/plain",
  ]),
  markdown: new Set([
    "application/markdown",
    "text/markdown",
    "text/plain",
    "text/x-markdown",
  ]),
  md: new Set([
    "application/markdown",
    "text/markdown",
    "text/plain",
    "text/x-markdown",
  ]),
  pdf: new Set(["application/pdf"]),
  png: new Set(["image/png"]),
  tsv: new Set(["text/plain", "text/tab-separated-values", "text/tsv"]),
  txt: new Set(["text/plain"]),
  webp: new Set(["image/webp"]),
  xls: new Set(["application/vnd.ms-excel", "application/octet-stream"]),
  xlsx: new Set([
    "application/vnd.ms-excel",
    "application/vnd.openxmlformats-officedocument.spreadsheetml.sheet",
  ]),
}

export function validateAiReadableUploads(files: File[]): UploadValidationResult {
  const acceptedFiles: File[] = []
  const errors: UploadValidationError[] = []

  for (const file of files) {
    const extension = fileExtension(file.name)
    const allowedMimeTypes = extension
      ? ALLOWED_MIME_TYPES_BY_EXTENSION[extension]
      : undefined

    if (!allowedMimeTypes) {
      errors.push({ code: "unsupportedFormat", file })
      continue
    }
    if (file.size === 0) {
      errors.push({ code: "empty", file })
      continue
    }
    if (file.size > MAX_AI_READABLE_UPLOAD_BYTES) {
      errors.push({ code: "tooLarge", file })
      continue
    }

    const mimeType = normalizeMimeType(file.type)
    if (
      mimeType &&
      mimeType !== "application/octet-stream" &&
      !allowedMimeTypes.has(mimeType)
    ) {
      errors.push({ code: "mimeMismatch", file })
      continue
    }

    acceptedFiles.push(file)
  }

  return { acceptedFiles, errors }
}

function fileExtension(fileName: string) {
  const match = fileName.trim().toLowerCase().match(/\.([^.]+)$/)
  return match?.[1] ?? null
}

function normalizeMimeType(value: string) {
  return value.split(";", 1)[0]?.trim().toLowerCase() ?? ""
}
