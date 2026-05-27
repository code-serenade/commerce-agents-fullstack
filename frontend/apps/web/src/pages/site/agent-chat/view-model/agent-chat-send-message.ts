import type { Dispatch, MutableRefObject, SetStateAction } from "react"
import { AGENT_ID, agentChatData, type AgentRuntimeEvent } from "../agent-chat-data"
import {
  createConversationTitle,
  createMessageId,
  formatAgentUiMessage,
  mapConversationItem,
  type AgentChatMessageFormatCopy,
} from "../domain/agent-chat-message-mapping"
import { createMessageContextSummary } from "../domain/agent-chat-message-context-summary"
import { sourceReadinessKey } from "../domain/agent-chat-source-readiness"
import {
  createContextRefs,
  type AgentChatCampaignContextSelection,
  type AgentChatProductContextSelection,
  type AgentChatSourceContextSelection,
} from "../domain/agent-chat-context-selection"
import type { AgentChatRuntimeSelection } from "../domain/agent-runtime-options"
import type { AgentChatStrategy } from "../domain/agent-chat-strategy"
import type {
  AgentChatAttachmentItem,
  AgentChatConversationItem,
  AgentChatMessage,
} from "../types"
import { MAX_VISION_IMAGE_ATTACHMENTS } from "./agent-chat-image-attachments"

interface SendAgentChatMessageInput {
  attachments: AgentChatAttachmentItem[]
  content: string
  conversationId: string | null
  fallbackConversationTitle: string
  messageFormatCopy: AgentChatMessageFormatCopy
  onController: (controller: AbortController) => void
  sessionIdRef: MutableRefObject<string>
  setConversationId: Dispatch<SetStateAction<string | null>>
  setConversations: Dispatch<SetStateAction<AgentChatConversationItem[]>>
  setError: Dispatch<SetStateAction<string | null>>
  setInput: Dispatch<SetStateAction<string>>
  setIsStreaming: Dispatch<SetStateAction<boolean>>
  setMessages: Dispatch<SetStateAction<AgentChatMessage[]>>
  setRuntimeEvents: Dispatch<SetStateAction<AgentRuntimeEvent[]>>
  selectedCampaigns: AgentChatCampaignContextSelection[]
  selectedProducts: AgentChatProductContextSelection[]
  selectedSources: AgentChatSourceContextSelection[]
  runtimeSelection: AgentChatRuntimeSelection
  strategy: AgentChatStrategy
  tError: string
}

export async function sendAgentChatMessage(
  input: SendAgentChatMessageInput
): Promise<AbortController | null> {
  const assistantId = createMessageId("assistant")
  const strategyMeta = {
    strategyId: input.strategy.id,
    responseMode: input.strategy.responseMode,
    routingMode: input.runtimeSelection.routingMode,
    collaborationMode: input.runtimeSelection.collaborationMode,
    learningLevel: input.runtimeSelection.learningLevel,
    primaryRole: input.runtimeSelection.primaryRole,
    selectedRoles: input.runtimeSelection.selectedRoles,
  }
  const contextMeta = {
    campaignIds: input.selectedCampaigns.map((campaign) => campaign.id),
    contextRefs: createContextRefs(
      input.selectedProducts,
      input.selectedSources,
      input.selectedCampaigns
    ),
    productIds: input.selectedProducts.map((product) => product.id),
  }
  const sourceContextDetails = input.selectedSources.map((source) => ({
    readiness: sourceReadinessKey(source),
    ref: source.ref,
    title: source.title,
  }))
  const imageAttachments = createImageAttachments(
    input.attachments,
    contextMeta.contextRefs
  )
  const contextSummary = createMessageContextSummary({
    ...contextMeta,
    collaborationMode: strategyMeta.collaborationMode,
    learningLevel: strategyMeta.learningLevel,
    routingMode: strategyMeta.routingMode,
    primaryRole: strategyMeta.primaryRole,
    campaignIds: contextMeta.campaignIds,
    campaigns: input.selectedCampaigns,
    products: input.selectedProducts,
    selectedRoles: strategyMeta.selectedRoles,
    sources: sourceContextDetails,
    strategyId: strategyMeta.strategyId,
  })
  input.setInput("")
  input.setError(null)
  input.setIsStreaming(true)
  input.setRuntimeEvents([])
  input.setMessages((current) => [
    ...current,
    {
      id: createMessageId("user"),
      role: "user",
      content: input.content,
      ...(contextSummary ? { contextSummary } : {}),
    },
    { id: assistantId, role: "assistant", content: "", status: "streaming" },
  ])

  const controller = new AbortController()
  input.onController(controller)

  try {
    let activeConversationId = input.conversationId
    if (!activeConversationId) {
      const detail = await agentChatData.createConversation({
        title: createConversationTitle(input.content),
        agentId: AGENT_ID,
      })
      activeConversationId = detail.conversation.id
      input.setConversationId(activeConversationId)
      input.setConversations((current) => [
        mapConversationItem(detail.conversation, input.fallbackConversationTitle),
        ...current.filter((item) => item.id !== detail.conversation.id),
      ])
    }
    input.sessionIdRef.current = activeConversationId
    await agentChatData.addConversationMessage(activeConversationId, {
      role: "user",
      contentText: input.content,
      contentJson: {
        context: { ...contextMeta, sources: sourceContextDetails },
        strategy: strategyMeta,
      },
      strategyId: strategyMeta.strategyId,
      responseMode: strategyMeta.responseMode,
      routingMode: strategyMeta.routingMode,
      collaborationMode: strategyMeta.collaborationMode,
      learningLevel: strategyMeta.learningLevel,
    })

    let assistantContent = ""
    let assistantDone = false
    const runtimeEvents: AgentRuntimeEvent[] = []
    await agentChatData.streamAgentSession(
      {
        input: input.content,
        agentId: AGENT_ID,
        metadata: {
          ...(imageAttachments.length
            ? { attachments: { images: imageAttachments } }
            : {}),
          strategy: {
            id: input.strategy.id,
            responseMode: input.strategy.responseMode,
            routingMode: input.runtimeSelection.routingMode,
            collaborationMode: input.runtimeSelection.collaborationMode,
            outputFormat: "structured_markdown",
            learningLevel: input.runtimeSelection.learningLevel,
            primaryRole: input.runtimeSelection.primaryRole,
            selectedRoles: input.runtimeSelection.selectedRoles,
          },
          context: {
            ...contextMeta,
            conversationId: activeConversationId ?? undefined,
          },
        },
        sessionId: input.sessionIdRef.current,
      },
      {
        signal: controller.signal,
        onDelta: (chunk) => {
          assistantContent += chunk
          const displayContent = formatAgentUiMessage(
            assistantContent,
            input.messageFormatCopy
          )
          input.setMessages((current) =>
            current.map((message) =>
              message.id === assistantId
                ? { ...message, content: displayContent }
                : message
            )
          )
        },
        onAgentEvent: (event) => {
          runtimeEvents.push(event)
          if (!assistantDone) {
            input.setRuntimeEvents((current) => [...current, event])
          }
          input.setMessages((current) =>
            current.map((message) =>
              message.id === assistantId
                ? {
                    ...message,
                    runtimeEvents: [...(message.runtimeEvents ?? []), event],
                  }
                : message
            )
          )
        },
        onDone: () => {
          if (assistantDone) return
          assistantDone = true
          input.setRuntimeEvents([...runtimeEvents])
          const displayContent = formatAgentUiMessage(
            assistantContent,
            input.messageFormatCopy
          )
          input.setMessages((current) =>
            current.map((message) =>
              message.id === assistantId
                ? {
                    ...message,
                    content: displayContent,
                    runtimeEvents: [...runtimeEvents],
                    status: undefined,
                  }
                : message
            )
          )
          input.setIsStreaming(false)
        },
        onError: (message) => {
          const displayMessage = formatAgentUiMessage(
            message,
            input.messageFormatCopy
          )
          input.setError(displayMessage)
          input.setMessages((current) =>
            current.map((item) =>
              item.id === assistantId
                ? {
                    ...item,
                    content: item.content || displayMessage,
                    status: "error",
                  }
                : item
            )
          )
          input.setIsStreaming(false)
        },
      }
    )
    const finalAssistantContent = formatAgentUiMessage(
      assistantContent,
      input.messageFormatCopy
    )
    if (finalAssistantContent) {
      await agentChatData.addConversationMessage(activeConversationId, {
        role: "assistant",
        contentText: finalAssistantContent,
        contentJson: runtimeEvents.length ? { runtimeEvents } : null,
      })
    }
    input.setIsStreaming(false)
  } catch (err) {
    if (controller.signal.aborted) return null
    input.setIsStreaming(false)
    const message = formatAgentUiMessage(
      err instanceof Error ? err.message : input.tError,
      input.messageFormatCopy
    )
    input.setError(message)
    input.setMessages((current) =>
      current.map((item) =>
        item.id === assistantId
          ? {
              ...item,
              content: item.content || message,
              status: "error",
            }
          : item
      )
    )
  }

  return controller
}

function createImageAttachments(
  attachments: AgentChatAttachmentItem[],
  contextRefs: string[]
) {
  return attachments
    .filter(
      (attachment) =>
        attachment.status === "uploaded" &&
        Boolean(attachment.fileId) &&
        Boolean(attachment.imageDataUrl)
    )
    .filter((attachment) =>
      contextRefs.includes(`conversation_file:${attachment.fileId}`)
    )
    .slice(0, MAX_VISION_IMAGE_ATTACHMENTS)
    .map((attachment) => ({
      detail: attachment.imageDetail ?? "auto",
      sourceRef: `conversation_file:${attachment.fileId}`,
      url: attachment.imageDataUrl!,
    }))
}
