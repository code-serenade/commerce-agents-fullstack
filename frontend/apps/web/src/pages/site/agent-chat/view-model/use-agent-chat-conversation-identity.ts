import { useCallback, useRef } from "react"
import type { Dispatch, MutableRefObject, SetStateAction } from "react"
import { AGENT_ID, agentChatData } from "../agent-chat-data"
import { mapConversationItem } from "../domain/agent-chat-message-mapping"
import type { AgentChatConversationItem } from "../types"

export type EnsureAgentChatConversation = (input: {
  title: string
}) => Promise<string | null>

export type PromoteAgentChatConversationTitle = (input: {
  conversationId: string | null
  title: string
}) => Promise<void>

export function useAgentChatConversationIdentity({
  conversationIdRef,
  fallbackConversationTitle,
  onConversationCreated,
  setConversationId,
  setConversations,
}: {
  conversationIdRef: MutableRefObject<string | null>
  fallbackConversationTitle: string
  onConversationCreated?: (conversationId: string) => void
  setConversationId: Dispatch<SetStateAction<string | null>>
  setConversations: Dispatch<SetStateAction<AgentChatConversationItem[]>>
}) {
  const conversationCreationRef = useRef<Promise<string | null> | null>(null)
  const conversationTitleRef = useRef(new Map<string, string | null>())

  const ensureConversation = useCallback<EnsureAgentChatConversation>(
    async ({ title }) => {
      if (conversationIdRef.current) return conversationIdRef.current
      if (conversationCreationRef.current) return conversationCreationRef.current

      const creation = agentChatData
        .createConversation({
          title,
          agentId: AGENT_ID,
        })
        .then((detail) => {
          const nextConversationId = detail.conversation.id
          conversationIdRef.current = nextConversationId
          conversationTitleRef.current.set(
            nextConversationId,
            detail.conversation.title
          )
          setConversationId(nextConversationId)
          onConversationCreated?.(nextConversationId)
          setConversations((current) => [
            mapConversationItem(detail.conversation, fallbackConversationTitle),
            ...current.filter((item) => item.id !== nextConversationId),
          ])
          return nextConversationId
        })
        .finally(() => {
          if (conversationCreationRef.current === creation) {
            conversationCreationRef.current = null
          }
        })

      conversationCreationRef.current = creation
      return creation
    },
    [
      conversationIdRef,
      fallbackConversationTitle,
      onConversationCreated,
      setConversationId,
      setConversations,
    ]
  )

  const promoteConversationTitle =
    useCallback<PromoteAgentChatConversationTitle>(
      async ({ conversationId, title }) => {
        const nextTitle = title.trim()
        if (!conversationId || !nextTitle) return
        if (
          conversationTitleRef.current.get(conversationId) !==
          fallbackConversationTitle
        ) {
          return
        }

        const conversation = await agentChatData.updateConversation(
          conversationId,
          { title: nextTitle }
        )
        conversationTitleRef.current.set(conversationId, conversation.title)
        setConversations((current) =>
          current.map((item) =>
            item.id === conversationId
              ? mapConversationItem(conversation, fallbackConversationTitle)
              : item
          )
        )
      },
      [fallbackConversationTitle, setConversations]
    )

  return {
    ensureConversation,
    promoteConversationTitle,
  }
}
