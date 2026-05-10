import type { Dispatch, MutableRefObject, SetStateAction } from "react"
import { AGENT_ID, agentChatData, type AgentRuntimeEvent } from "../agent-chat-data"
import {
  createConversationTitle,
  createMessageId,
  formatAgentUiMessage,
  mapConversationItem,
} from "../domain/agent-chat-message-mapping"
import {
  createContextRefs,
  type AgentChatProductContextSelection,
} from "../domain/agent-chat-context-selection"
import type { AgentChatRuntimeSelection } from "../domain/agent-runtime-options"
import type { AgentChatStrategy } from "../domain/agent-chat-strategy"
import type { AgentChatConversationItem, AgentChatMessage } from "../types"

interface SendAgentChatMessageInput {
  content: string
  conversationId: string | null
  fallbackConversationTitle: string
  missingStructuredOutput: string
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
  runtimeSelection: AgentChatRuntimeSelection
  strategy: AgentChatStrategy
  tError: string
}

export async function sendAgentChatMessage(
  input: SendAgentChatMessageInput
): Promise<AbortController | null> {
  const assistantId = createMessageId("assistant")
  input.setInput("")
  input.setError(null)
  input.setIsStreaming(true)
  input.setRuntimeEvents([])
  input.setMessages((current) => [
    ...current,
    { id: createMessageId("user"), role: "user", content: input.content },
    { id: assistantId, role: "assistant", content: "", status: "streaming" },
  ])

  const controller = new AbortController()
  input.onController(controller)

  try {
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
      contextRefs: createContextRefs(input.selectedProducts),
      productIds: input.selectedProducts.map((product) => product.id),
    }
    let activeConversationId = input.conversationId
    if (!activeConversationId) {
      const detail = await agentChatData.createConversation({
        title: createConversationTitle(input.content),
        firstMessage: input.content,
        agentId: AGENT_ID,
      })
      activeConversationId = detail.conversation.id
      input.setConversationId(activeConversationId)
      input.setConversations((current) => [
        mapConversationItem(detail.conversation, input.fallbackConversationTitle),
        ...current.filter((item) => item.id !== detail.conversation.id),
      ])
      input.sessionIdRef.current = activeConversationId
    } else {
      await agentChatData.addConversationMessage(activeConversationId, {
        role: "user",
        contentText: input.content,
        contentJson: { context: contextMeta, strategy: strategyMeta },
      })
    }

    let assistantContent = ""
    let persistAssistant: Promise<void> | null = null
    await agentChatData.streamAgentSession(
      {
        input: input.content,
        agentId: AGENT_ID,
        metadata: {
          context: contextMeta,
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
          input.setRuntimeEvents((current) => [...current, event])
        },
        onDone: () => {
          input.setMessages((current) =>
            current.map((message) =>
              message.id === assistantId
                ? { ...message, status: undefined }
                : message
            )
          )
          input.setIsStreaming(false)
          if (assistantContent) {
            persistAssistant = agentChatData
              .addConversationMessage(activeConversationId, {
                role: "assistant",
                contentText: assistantContent,
                contentJson: null,
              })
              .then(() => undefined)
          }
        },
        onError: (message) => {
          const displayMessage = formatAgentUiMessage(
            message,
            input.missingStructuredOutput
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
    await persistAssistant
  } catch (err) {
    if (controller.signal.aborted) return null
    const message = formatAgentUiMessage(
      err instanceof Error ? err.message : input.tError,
      input.missingStructuredOutput
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
