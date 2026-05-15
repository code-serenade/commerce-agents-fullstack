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
import {
  createContextRefs,
  type AgentChatProductContextSelection,
  type AgentChatSourceContextSelection,
} from "../domain/agent-chat-context-selection"
import type { AgentChatRuntimeSelection } from "../domain/agent-runtime-options"
import type { AgentChatStrategy } from "../domain/agent-chat-strategy"
import type { AgentChatConversationItem, AgentChatMessage } from "../types"

interface SendAgentChatMessageInput {
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
    collaborationMode: input.runtimeSelection.collaborationMode,
    learningLevel: input.runtimeSelection.learningLevel,
    primaryRole: input.runtimeSelection.primaryRole,
    selectedRoles: input.runtimeSelection.selectedRoles,
    modelName: input.runtimeSelection.modelName,
  }
  const contextMeta = {
    contextRefs: createContextRefs(input.selectedProducts, input.selectedSources),
    productIds: input.selectedProducts.map((product) => product.id),
  }
  const contextSummary = createMessageContextSummary({
    ...contextMeta,
    collaborationMode: strategyMeta.collaborationMode,
    learningLevel: strategyMeta.learningLevel,
    modelName: strategyMeta.modelName,
    primaryRole: strategyMeta.primaryRole,
    products: input.selectedProducts,
    selectedRoles: strategyMeta.selectedRoles,
    sources: input.selectedSources.map((source) => ({
      ref: source.ref,
      title: source.title,
    })),
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
      contentJson: { context: contextMeta, strategy: strategyMeta },
    })

    let assistantContent = ""
    const runtimeEvents: AgentRuntimeEvent[] = []
    await agentChatData.streamAgentSession(
      {
        input: input.content,
        agentId: AGENT_ID,
        metadata: {
          model: input.runtimeSelection.modelName
            ? { name: input.runtimeSelection.modelName }
            : undefined,
          strategy: {
            id: input.strategy.id,
            responseMode: input.strategy.responseMode,
            collaborationMode: input.runtimeSelection.collaborationMode,
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
          input.setMessages((current) =>
            current.map((message) =>
              message.id === assistantId
                ? { ...message, content: message.content + chunk }
                : message
            )
          )
        },
        onAgentEvent: (event) => {
          runtimeEvents.push(event)
          input.setRuntimeEvents((current) => [...current, event])
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
          input.setMessages((current) =>
            current.map((message) =>
              message.id === assistantId
                ? {
                    ...message,
                    runtimeEvents,
                    status: undefined,
                  }
                : message
            )
          )
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
    const finalAssistantContent = assistantContent.trim()
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
    const message = formatAgentUiMessage(
      err instanceof Error ? err.message : input.tError,
      input.messageFormatCopy
    )
    input.setError(message)
    input.setIsStreaming(false)
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
