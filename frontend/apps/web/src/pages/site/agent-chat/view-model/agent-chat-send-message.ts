import { AGENT_ID, agentChatData, type AgentRuntimeEvent } from "../agent-chat-data"
import {
  createConversationTitle,
  createMessageId,
  formatAgentUiMessage,
} from "../domain/agent-chat-message-mapping"
import { createMessageContextSummary } from "../domain/agent-chat-message-context-summary"
import { sourceReadinessKey } from "../domain/agent-chat-source-readiness"
import { createContextRefs } from "../domain/agent-chat-context-selection"
import { createImageAttachments } from "./agent-chat-image-attachments"
import {
  markAssistantAnswerComplete,
  replaceAssistantAnswerComplete,
} from "./agent-chat-message-state"
import { saveAssistantConversationMessage } from "./agent-chat-save-message"
import type { SendAgentChatMessageInput } from "./agent-chat-send-message-types"
import { hasDisplayableAnswer, isAbortLikeStreamError } from "./agent-chat-stream-errors"
import { formatStreamErrorMessage } from "./agent-chat-stream-error-message"
import { createAssistantRevealController } from "./agent-chat-assistant-reveal"

export async function sendAgentChatMessage(
  input: SendAgentChatMessageInput
): Promise<AbortController | null> {
  const assistantId = createMessageId("assistant")
  const routingMode =
    input.strategy.id === "quick" ? "fast" : input.runtimeSelection.routingMode
  const collaborationMode =
    input.strategy.id === "quick" ? "auto" : input.runtimeSelection.collaborationMode
  const strategyMeta = {
    strategyId: input.strategy.id,
    responseMode: input.strategy.responseMode,
    routingMode,
    collaborationMode,
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
  const imageAttachments = createImageAttachments(input.attachments, contextMeta.contextRefs)
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

  let assistantContent = ""
  let assistantDone = false
  let assistantMessageSaved = false
  let activeConversationId = input.conversationId
  const nextConversationTitle = createConversationTitle(input.content)
  const runtimeEvents: AgentRuntimeEvent[] = []

  function renderAssistantContent(content: string) {
    return formatAgentUiMessage(content, input.messageFormatCopy)
  }

  function renderVisibleAssistantContent() {
    return renderAssistantContent(assistantReveal.getVisibleContent())
  }

  function renderFullAssistantContent() {
    return renderAssistantContent(assistantContent)
  }

  function setAssistantContent(content: string) {
    input.setMessages((current) =>
      current.map((message) =>
        message.id === assistantId ? { ...message, content } : message
      )
    )
  }

  const assistantReveal = createAssistantRevealController({
    onVisibleContentChange: (content) =>
      setAssistantContent(renderAssistantContent(content)),
  })

  function queueAssistantContent(chunk: string) {
    assistantContent += chunk
    assistantReveal.queue(chunk)
  }

  async function saveAssistantIfReady(content: string) {
    if (!activeConversationId || !content || assistantMessageSaved) return
    const savedMessage = await saveAssistantConversationMessage({
      content,
      conversationId: activeConversationId,
      runtimeEvents,
    })
    assistantMessageSaved = true
    if (savedMessage.id) {
      input.setMessages((current) =>
        current.map((message) =>
          message.id === assistantId
            ? { ...message, persistedId: savedMessage.id }
            : message
        )
      )
    }
  }

  try {
    if (!activeConversationId) {
      activeConversationId = await input.ensureConversation({
        title: nextConversationTitle,
      })
    }
    await input.promoteConversationTitle({
      conversationId: activeConversationId,
      currentTitle: input.conversationTitle,
      title: nextConversationTitle,
    })
    if (!activeConversationId) return controller
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
            routingMode,
            collaborationMode,
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
          queueAssistantContent(chunk)
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
        },
        onError: (message) => {
          assistantReveal.flush()
          if (assistantDone) {
            input.setIsStreaming(false)
            return
          }
          const displayContent = renderVisibleAssistantContent()
          if (
            hasDisplayableAnswer(displayContent) &&
            isAbortLikeStreamError(message)
          ) {
            assistantDone = true
            input.setRuntimeEvents([...runtimeEvents])
            markAssistantAnswerComplete({
              assistantId,
              content: displayContent,
              runtimeEvents,
              setMessages: input.setMessages,
            })
            input.setIsStreaming(false)
            return
          }
          const displayMessage = formatStreamErrorMessage(message, input)
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
    await assistantReveal.waitForIdle()
    if (assistantDone) {
      input.setRuntimeEvents([...runtimeEvents])
      replaceAssistantAnswerComplete({
        assistantId,
        content: renderVisibleAssistantContent(),
        runtimeEvents,
        setMessages: input.setMessages,
      })
    }
    const finalAssistantContent = renderFullAssistantContent()
    await saveAssistantIfReady(finalAssistantContent)
    input.setIsStreaming(false)
  } catch (err) {
    assistantReveal.flush()
    if (controller.signal.aborted) return null
    if (assistantDone) {
      input.setIsStreaming(false)
      const displayContent = renderVisibleAssistantContent()
      replaceAssistantAnswerComplete({
        assistantId,
        content: displayContent,
        runtimeEvents,
        setMessages: input.setMessages,
      })
      try {
        await saveAssistantIfReady(displayContent)
      } catch {
        // The visible answer is already complete; do not convert it to a reply failure.
      }
      return controller
    }
    input.setIsStreaming(false)
    const displayContent = renderVisibleAssistantContent()
    if (hasDisplayableAnswer(displayContent) && isAbortLikeStreamError(err)) {
      input.setRuntimeEvents([...runtimeEvents])
      markAssistantAnswerComplete({
        assistantId,
        content: displayContent,
        runtimeEvents,
        setMessages: input.setMessages,
      })
      await saveAssistantIfReady(displayContent)
      return controller
    }
    const message = formatStreamErrorMessage(err, input)
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
