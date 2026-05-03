import { useCallback, useEffect, useRef, useState } from "react"
import { useTranslation } from "react-i18next"
import { useSelector } from "react-redux"
import { streamAgentSessionApi } from "@/api"
import type { RootState } from "@/store"
import type { AgentChatMessage } from "./types"

const AGENT_ID = "web-agent"
const WORKSPACE_ID = "workspace-001"

export function useAgentChatPage() {
  const { t } = useTranslation()
  const user = useSelector((state: RootState) => state.auth.user)
  const [input, setInput] = useState("")
  const [messages, setMessages] = useState<AgentChatMessage[]>(() => [
    {
      id: "welcome",
      role: "assistant",
      content: t("agentChat.welcomeMessage"),
    },
  ])
  const [isStreaming, setIsStreaming] = useState(false)
  const [error, setError] = useState<string | null>(null)
  const abortRef = useRef<AbortController | null>(null)
  const sessionIdRef = useRef(createSessionId())

  useEffect(() => {
    return () => abortRef.current?.abort()
  }, [])

  const canSend = Boolean(user) && input.trim().length > 0 && !isStreaming

  const sendMessage = useCallback(async () => {
    const content = input.trim()
    if (!content || !user || isStreaming) return

    const assistantId = createMessageId("assistant")
    setInput("")
    setError(null)
    setIsStreaming(true)
    setMessages((current) => [
      ...current,
      {
        id: createMessageId("user"),
        role: "user",
        content,
      },
      {
        id: assistantId,
        role: "assistant",
        content: "",
        status: "streaming",
      },
    ])

    const controller = new AbortController()
    abortRef.current = controller

    try {
      await streamAgentSessionApi(
        {
          input: content,
          agentId: AGENT_ID,
          sessionId: sessionIdRef.current,
          workspaceId: WORKSPACE_ID,
        },
        {
          signal: controller.signal,
          onDelta: (chunk) => {
            setMessages((current) =>
              current.map((message) =>
                message.id === assistantId
                  ? { ...message, content: message.content + chunk }
                  : message
              )
            )
          },
          onDone: () => {
            setMessages((current) =>
              current.map((message) =>
                message.id === assistantId
                  ? { ...message, status: undefined }
                  : message
              )
            )
            setIsStreaming(false)
            abortRef.current = null
          },
          onError: (message) => {
            setError(message)
            setMessages((current) =>
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
            setIsStreaming(false)
            abortRef.current = null
          },
        }
      )
    } catch (err) {
      if (controller.signal.aborted) return
      const message = err instanceof Error ? err.message : t("agentChat.error")
      setError(message)
      setIsStreaming(false)
      abortRef.current = null
    }
  }, [input, isStreaming, t, user])

  const stopStreaming = useCallback(() => {
    abortRef.current?.abort()
    abortRef.current = null
    setIsStreaming(false)
    setMessages((current) =>
      current.map((message) =>
        message.status === "streaming"
          ? { ...message, status: undefined }
          : message
      )
    )
  }, [])

  return {
    canSend,
    error,
    input,
    isLoggedIn: Boolean(user),
    isStreaming,
    messages,
    sendMessage,
    setInput,
    stopStreaming,
  }
}

function createSessionId() {
  return `web-${Date.now()}-${Math.random().toString(36).slice(2, 10)}`
}

function createMessageId(prefix: string) {
  return `${prefix}-${Date.now()}-${Math.random().toString(36).slice(2, 10)}`
}
