import type { Dispatch, SetStateAction } from "react"
import type {
  ConversationFileResponse,
  UpdateConversationFileExtractionRequest,
} from "@/api"
import { AGENT_ID, agentChatData } from "../agent-chat-data"
import {
  mapConversationFile,
  mapConversationItem,
} from "../domain/agent-chat-message-mapping"
import type {
  AgentChatAttachmentItem,
  AgentChatConversationItem,
} from "../types"
import { resolveAttachmentExtractionState } from "./agent-chat-attachment-extraction-state"
import {
  createAttachmentId,
  createBrowserExtractionRequest,
  isReadableTextFile,
  isUploadImageFile,
} from "./agent-chat-browser-extraction"
export {
  createBrowserExtractionRequest,
  type BrowserExtractionRequestResult,
} from "./agent-chat-browser-extraction"
import {
  isVisionImageFile,
} from "./agent-chat-image-attachments"

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
}: UploadAgentChatAttachmentsInput): Promise<{
  conversationId: string | null
  fileIds: string[]
}> {
  if (!files.length) return { conversationId, fileIds: [] }
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

  const fileIds = (
    await Promise.all(
    files.map((file, index) =>
      uploadOneAttachment(activeConversationId!, file, pendingItems[index].id, {
        setAttachments,
      })
    )
    )
  ).filter((fileId): fileId is string => Boolean(fileId))

  return { conversationId: activeConversationId, fileIds }
}

export async function deleteAgentChatAttachments({
  fileIds,
  setAttachments,
}: {
  fileIds: string[]
  setAttachments: Dispatch<SetStateAction<AgentChatAttachmentItem[]>>
}) {
  const uniqueFileIds = Array.from(new Set(fileIds.filter(Boolean)))
  if (!uniqueFileIds.length) return

  await Promise.all(
    uniqueFileIds.map((fileId) => agentChatData.deleteConversationFile(fileId))
  )
  setAttachments((current) =>
    current.filter((item) => !item.fileId || !uniqueFileIds.includes(item.fileId))
  )
}

export function mapConversationFileToAttachment(
  file: ConversationFileResponse
): AgentChatAttachmentItem {
  return mapConversationFile(file)
}

export function mergeConversationFileAttachment(
  current: AgentChatAttachmentItem[],
  file: ConversationFileResponse
) {
  const next = mapConversationFileToAttachment(file)
  return [...current.filter((item) => item.fileId !== file.id), next]
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
    const upload = isUploadImageFile(file)
      ? await agentChatData.uploadImage(file)
      : await agentChatData.uploadDocument(file)
    logAttachmentUploadResult(file, upload)
    const registered = await agentChatData.registerConversationFile(conversationId, {
      fileName: file.name || upload.key.split("/").at(-1),
      fileSize: upload.size || file.size,
      mimeType: file.type || "application/octet-stream",
      objectKey: upload.key,
      sha256: upload.sha256,
    })
    const extraction = await createBrowserExtractionRequest(file)
    const extractionUpdated = extraction.request
      ? await updateRegisteredFileExtraction(registered.id, extraction.request)
      : false
    const shouldTriggerServerExtraction =
      !extraction.request && !isUploadImageFile(file)
    const serverExtraction =
      extractionUpdated && extraction.readable
        ? null
        : shouldTriggerServerExtraction
          ? await triggerServerExtractionStatus(registered.id)
          : null
    const resolved = resolveAttachmentExtractionState({
      browserReadable: extraction.readable && extractionUpdated,
      fallbackReason: extraction.request?.failureReason ?? undefined,
      imageDataUrl: extraction.imageDataUrl,
      serverExtraction,
      status: extraction.request?.status,
    })
    setAttachments((current) =>
      current.map((item) =>
        item.id === localId
          ? {
              ...item,
              bucket: registered.bucket,
              fileId: registered.id,
              imageDataUrl: extraction.imageDataUrl,
              imageDetail: extraction.imageDataUrl ? "auto" : undefined,
              objectKey: registered.objectKey,
              readable: resolved.readable,
              status: resolved.status,
              unsupportedReason: resolved.unsupportedReason,
            }
          : item
      )
    )
    return registered.id
  } catch (error) {
    logAttachmentUploadFailure(file, error)
    setAttachments((current) =>
      current.map((item) =>
        item.id === localId ? { ...item, status: "failed" } : item
      )
    )
    return null
  }
}

async function updateRegisteredFileExtraction(
  fileId: string,
  request: UpdateConversationFileExtractionRequest
) {
  try {
    await agentChatData.updateConversationFileExtraction(fileId, request)
    return true
  } catch (error) {
    console.warn("[agent-chat] attachment extraction update failed", {
      error,
      fileId,
      status: request.status,
    })
    return false
  }
}

async function triggerServerExtractionStatus(fileId: string) {
  try {
    return await agentChatData.triggerConversationFileExtractionJob(fileId, {
      mode: "auto",
      force: false,
    })
  } catch (error) {
    console.warn("[agent-chat] attachment server extraction job failed", {
      error,
      fileId,
    })
    return null
  }
}

function logAttachmentUploadResult(
  file: File,
  upload: { already_uploaded: boolean; key: string; sha256: string; size: number }
) {
  console.info("[agent-chat] attachment uploaded through file-service", {
    alreadyUploaded: upload.already_uploaded,
    fileName: file.name,
    fileSize: upload.size || file.size,
    fileType: file.type || null,
    objectKey: upload.key,
    sha256: upload.sha256,
  })
}

function logAttachmentUploadFailure(file: File, error: unknown) {
  console.error("[agent-chat] attachment upload failed", {
    error,
    fileName: file.name,
    fileSize: file.size,
    fileType: file.type || null,
  })
}
