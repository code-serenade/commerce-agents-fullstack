import type {
  ConversationFileExtractionJobResponse,
  UpdateConversationFileExtractionRequest,
} from "@/api"
import type { AgentChatAttachmentItem } from "../types"

interface ResolveAttachmentExtractionStateInput {
  browserReadable: boolean
  fallbackReason?: string
  imageDataUrl?: string
  serverExtraction: ConversationFileExtractionJobResponse | null
  status?: UpdateConversationFileExtractionRequest["status"]
}

export function resolveAttachmentExtractionState({
  browserReadable,
  fallbackReason,
  imageDataUrl,
  serverExtraction,
  status,
}: ResolveAttachmentExtractionStateInput): Pick<
  AgentChatAttachmentItem,
  "readable" | "status" | "unsupportedReason"
> {
  if (browserReadable) {
    return {
      readable: true,
      status: "uploaded",
      unsupportedReason: fallbackReason,
    }
  }

  if (imageDataUrl) {
    return {
      readable: true,
      status: "uploaded",
      unsupportedReason: serverExtraction?.message ?? fallbackReason,
    }
  }

  if (serverExtraction?.status === "pending") {
    return {
      readable: false,
      status: "pending",
      unsupportedReason: serverExtraction.message || fallbackReason,
    }
  }

  if (serverExtraction?.status === "unsupported") {
    return {
      readable: false,
      status: "unsupported",
      unsupportedReason: serverExtraction.message || fallbackReason,
    }
  }

  if (status === "unsupported") {
    return {
      readable: false,
      status: "unsupported",
      unsupportedReason: fallbackReason,
    }
  }

  if (status === "failed") {
    return {
      readable: false,
      status: "failed",
      unsupportedReason: fallbackReason,
    }
  }

  return {
    readable: false,
    status: "pending",
    unsupportedReason: fallbackReason,
  }
}
