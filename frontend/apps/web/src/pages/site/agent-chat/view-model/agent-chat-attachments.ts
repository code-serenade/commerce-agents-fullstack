import type { Dispatch, SetStateAction } from "react"
import { AGENT_ID, agentChatData } from "../agent-chat-data"
import { mapConversationItem } from "../domain/agent-chat-message-mapping"
import type {
  AgentChatAttachmentItem,
  AgentChatConversationItem,
} from "../types"

const MAX_INLINE_TEXT_CHARS = 100_000
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
    name: file.name,
    readable: isReadableTextFile(file),
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
    const readableText = await readAttachmentText(file)
    if (readableText) {
      await agentChatData.updateConversationFileExtraction(registered.id, {
        contentKind: contentKindForFile(file),
        metadata: { sourceFileName: file.name },
        parser: "browser_text",
        source: "browser_upload",
        status: "extracted",
        text: readableText,
      })
    }
    setAttachments((current) =>
      current.map((item) =>
        item.id === localId
          ? {
              ...item,
              fileId: registered.id,
              readable: Boolean(readableText),
              status: "uploaded",
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

async function readAttachmentText(file: File) {
  if (!isReadableTextFile(file)) return null
  const text = await file.text()
  const trimmed = text.trim()
  if (!trimmed) return null
  return trimmed.length > MAX_INLINE_TEXT_CHARS
    ? trimmed.slice(0, MAX_INLINE_TEXT_CHARS)
    : trimmed
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

function createAttachmentId(file: File) {
  return `attachment-${file.name}-${file.size}-${Date.now()}-${Math.random()
    .toString(36)
    .slice(2, 8)}`
}
