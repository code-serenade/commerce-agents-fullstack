import type { Dispatch, SetStateAction } from "react"
import type { UpdateConversationFileExtractionRequest } from "@/api"
import { AGENT_ID, agentChatData } from "../agent-chat-data"
import { mapConversationItem } from "../domain/agent-chat-message-mapping"
import type {
  AgentChatAttachmentItem,
  AgentChatConversationItem,
} from "../types"
import { resolveAttachmentExtractionState } from "./agent-chat-attachment-extraction-state"
import {
  createBrowserImageExtractionRequest,
  isImageFile,
  isVisionImageFile,
} from "./agent-chat-image-attachments"

const MAX_INLINE_TEXT_CHARS = 100_000
const UNSUPPORTED_TEXT_EXTRACTION_REASON =
  "Browser text extraction supports plain text, CSV, JSON, Markdown, and HTML files only."
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

export interface UploadAgentChatAttachmentsInput {
  conversationId: string | null
  fallbackConversationTitle: string
  files: File[]
  setAttachments: Dispatch<SetStateAction<AgentChatAttachmentItem[]>>
  setConversationId: Dispatch<SetStateAction<string | null>>
  setConversations: Dispatch<SetStateAction<AgentChatConversationItem[]>>
}

export async function uploadAgentChatAttachments({
  conversationId,
  fallbackConversationTitle,
  files,
  setAttachments,
  setConversationId,
  setConversations,
}: UploadAgentChatAttachmentsInput): Promise<string | null> {
  if (!files.length) return conversationId
  let activeConversationId = conversationId
  if (!activeConversationId) {
    const detail = await agentChatData.createConversation({
      title: fallbackConversationTitle,
      agentId: AGENT_ID,
    })
    activeConversationId = detail.conversation.id
    setConversationId(activeConversationId)
    setConversations((current) => [
      mapConversationItem(detail.conversation, fallbackConversationTitle),
      ...current.filter((item) => item.id !== detail.conversation.id),
    ])
  }

  const pendingItems = files.map((file) => ({
    id: createAttachmentId(file),
    kind: isVisionImageFile(file) ? ("image" as const) : ("file" as const),
    name: file.name,
    readable: isReadableTextFile(file) || isVisionImageFile(file),
    status: "uploading" as const,
  }))
  setAttachments((current) => [...current, ...pendingItems])

  await Promise.all(
    files.map((file, index) =>
      uploadOneAttachment(activeConversationId!, file, pendingItems[index].id, {
        setAttachments,
      })
    )
  )

  return activeConversationId
}

async function uploadOneAttachment(
  conversationId: string,
  file: File,
  localId: string,
  {
    setAttachments,
  }: {
    setAttachments: Dispatch<SetStateAction<AgentChatAttachmentItem[]>>
  }
) {
  try {
    const sign = await agentChatData.presignConversationFileUpload(conversationId, {
      contentType: file.type || "application/octet-stream",
      extension: extensionFromFile(file),
    })
    await agentChatData.uploadConversationFile(file, sign)
    const registered = await agentChatData.registerConversationFile(conversationId, {
      fileName: file.name || sign.objectKey.split("/").at(-1),
      fileSize: file.size,
      mimeType: file.type || sign.contentType || "application/octet-stream",
      objectKey: sign.objectKey,
    })
    const extraction = await createBrowserExtractionRequest(file)
    const extractionUpdated = await updateRegisteredFileExtraction(
      registered.id,
      extraction.request
    )
    const shouldTriggerServerExtraction =
      extraction.request.status === "unsupported"
    const serverExtraction =
      extractionUpdated && extraction.readable
        ? null
        : shouldTriggerServerExtraction
          ? await triggerServerExtractionStatus(registered.id)
          : null
    const resolved = resolveAttachmentExtractionState({
      browserReadable: extraction.readable && extractionUpdated,
      fallbackReason: extraction.request.failureReason ?? undefined,
      imageDataUrl: extraction.imageDataUrl,
      serverExtraction,
      status: extraction.request.status,
    })
    setAttachments((current) =>
      current.map((item) =>
        item.id === localId
          ? {
              ...item,
              fileId: registered.id,
              imageDataUrl: extraction.imageDataUrl,
              imageDetail: extraction.imageDataUrl ? "auto" : undefined,
              readable: resolved.readable,
              status: resolved.status,
              unsupportedReason: resolved.unsupportedReason,
            }
          : item
      )
    )
  } catch {
    setAttachments((current) =>
      current.map((item) =>
        item.id === localId ? { ...item, status: "failed" } : item
      )
    )
  }
}

export interface BrowserExtractionRequestResult {
  imageDataUrl?: string
  readable: boolean
  request: UpdateConversationFileExtractionRequest
}

export async function createBrowserExtractionRequest(
  file: File
): Promise<BrowserExtractionRequestResult> {
  const metadata = browserExtractionMetadata(file)
  if (!isReadableTextFile(file)) {
    if (isImageFile(file)) {
      return createBrowserImageExtractionRequest(file, metadata)
    }
    return {
      readable: false,
      request: {
        contentKind: "unsupported",
        failureReason: UNSUPPORTED_TEXT_EXTRACTION_REASON,
        metadata,
        parser: "browser_text",
        source: "browser_upload",
        status: "unsupported",
      },
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

async function updateRegisteredFileExtraction(
  fileId: string,
  request: UpdateConversationFileExtractionRequest
) {
  try {
    await agentChatData.updateConversationFileExtraction(fileId, request)
    return true
  } catch {
    return false
  }
}

async function triggerServerExtractionStatus(fileId: string) {
  try {
    return await agentChatData.triggerConversationFileExtractionJob(fileId, {
      mode: "auto",
      force: false,
    })
  } catch {
    return null
  }
}

function isReadableTextFile(file: File) {
  const type = file.type.toLowerCase()
  return (
    type.startsWith("text/") ||
    READABLE_TEXT_TYPES.has(type) ||
    /\.(csv|json|md|markdown|txt)$/i.test(file.name)
  )
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

function createAttachmentId(file: File) {
  return `attachment-${file.name}-${file.size}-${Date.now()}-${Math.random()
    .toString(36)
    .slice(2, 8)}`
}
