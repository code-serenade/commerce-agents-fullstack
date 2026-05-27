import type { UpdateConversationFileExtractionRequest } from "@/api"
import { createBrowserImageExtractionRequest } from "./agent-chat-image-attachments"

const MAX_INLINE_TEXT_CHARS = 100_000
const EMPTY_TEXT_EXTRACTION_REASON = "File contains no readable text."
const FAILED_TEXT_EXTRACTION_REASON = "Browser text extraction failed."
const READABLE_TEXT_TYPES = new Set([
  "application/json",
  "application/ld+json",
  "application/markdown",
  "text/csv",
  "text/html",
  "text/markdown",
  "text/plain",
  "text/tab-separated-values",
])

export interface BrowserExtractionRequestResult {
  imageDataUrl?: string
  readable: boolean
  request: UpdateConversationFileExtractionRequest | null
}

export async function createBrowserExtractionRequest(
  file: File
): Promise<BrowserExtractionRequestResult> {
  const metadata = browserExtractionMetadata(file)
  if (!isReadableTextFile(file)) {
    if (isUploadImageFile(file)) {
      return createBrowserImageExtractionRequest(file, metadata)
    }
    return {
      readable: false,
      request: null,
    }
  }

  let text = ""
  try {
    text = await file.text()
  } catch {
    return {
      readable: false,
      request: {
        contentKind: contentKindForFile(file),
        failureReason: FAILED_TEXT_EXTRACTION_REASON,
        metadata,
        parser: "browser_text",
        source: "browser_upload",
        status: "failed",
      },
    }
  }

  const trimmed = text.trim()
  if (!trimmed) {
    return {
      readable: false,
      request: {
        contentKind: contentKindForFile(file),
        failureReason: EMPTY_TEXT_EXTRACTION_REASON,
        metadata,
        parser: "browser_text",
        source: "browser_upload",
        status: "failed",
      },
    }
  }

  const truncated = trimmed.length > MAX_INLINE_TEXT_CHARS
  return {
    readable: true,
    request: {
      contentKind: contentKindForFile(file),
      metadata: {
        ...metadata,
        originalTextChars: trimmed.length,
        truncated,
      },
      parser: "browser_text",
      source: "browser_upload",
      status: "extracted",
      text: truncated ? trimmed.slice(0, MAX_INLINE_TEXT_CHARS) : trimmed,
    },
  }
}

export function isReadableTextFile(file: File) {
  const type = file.type.toLowerCase()
  return (
    type.startsWith("text/") ||
    READABLE_TEXT_TYPES.has(type) ||
    /\.(csv|json|md|markdown|txt)$/i.test(file.name)
  )
}

export function isUploadImageFile(file: File) {
  return (
    file.type.startsWith("image/") || /\.(gif|jpe?g|png|webp)$/i.test(file.name)
  )
}

export function createAttachmentId(file: File) {
  return `attachment-${file.name}-${file.size}-${Date.now()}-${Math.random()
    .toString(36)
    .slice(2, 8)}`
}

function contentKindForFile(file: File) {
  const extension = extensionFromFile(file)
  if (extension === "json") return "json"
  if (extension === "csv") return "csv"
  if (extension === "md" || extension === "markdown") return "markdown"
  return "plain_text"
}

function extensionFromFile(file: File) {
  const extension = file.name.match(/\.([^.]+)$/)?.[1]
  if (extension) return extension.toLowerCase()
  const subtype = file.type.split("/")[1]?.split(";")[0]?.trim()
  return subtype || "bin"
}

function browserExtractionMetadata(file: File) {
  return {
    fileSize: file.size,
    mimeType: file.type || null,
    sourceFileName: file.name,
  }
}
