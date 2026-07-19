import { useCallback, useEffect, useRef } from "react"
import type { MutableRefObject } from "react"
import {
  loadAgentConversationById,
  loadAgentConversationList,
  resetActiveConversation,
} from "./agent-chat-conversation-actions"
import { resetGuestAgentChatState } from "./agent-chat-page-actions"
import type { useAgentChatActionState } from "./use-agent-chat-action-state"

type AgentChatActionState = ReturnType<
  typeof useAgentChatActionState
>["actionState"]
type AgentChatNavigate = (to: string, options?: { replace?: boolean }) => void

export function useAgentChatBootstrap({
  actionState,
  conversationId,
  conversationIdRef,
  isAgentChatRoute,
  routeConversationId,
  tError,
  userBootstrapKey,
}: {
  actionState: AgentChatActionState
  conversationId: string | null
  conversationIdRef: MutableRefObject<string | null>
  isAgentChatRoute: boolean
  routeConversationId?: string
  tError: string
  userBootstrapKey: string | null
}) {
  const actionStateRef = useRef(actionState)

  useEffect(() => {
    actionStateRef.current = actionState
  }, [actionState])

  useEffect(() => {
    conversationIdRef.current = conversationId
  }, [conversationId, conversationIdRef])

  useEffect(() => {
    let cancelled = false
    const latestActionState = actionStateRef.current

    if (!userBootstrapKey) {
      resetGuestAgentChatState(latestActionState, () => cancelled)
    } else {
      void loadUserAgentChat({
        activeConversationId: conversationIdRef.current,
        isAgentChatRoute,
        isCancelled: () => cancelled,
        latestActionState,
        routeConversationId,
        tError,
      })
    }

    return () => {
      cancelled = true
    }
  }, [
    conversationIdRef,
    isAgentChatRoute,
    routeConversationId,
    tError,
    userBootstrapKey,
  ])
}

export function useAgentChatRouteActions({
  isAgentChatRoute,
  navigate,
}: {
  isAgentChatRoute: boolean
  navigate: AgentChatNavigate
}) {
  const syncConversationRoute = useCallback(
    (nextConversationId: string) => {
      if (!isAgentChatRoute) return
      navigate(`/commerce/${nextConversationId}`, { replace: true })
    },
    [isAgentChatRoute, navigate]
  )

  const replaceWithNewConversationRoute = useCallback(() => {
    if (!isAgentChatRoute) return
    navigate("/commerce", { replace: true })
  }, [isAgentChatRoute, navigate])

  return { replaceWithNewConversationRoute, syncConversationRoute }
}

async function loadUserAgentChat({
  activeConversationId,
  isAgentChatRoute,
  isCancelled,
  latestActionState,
  routeConversationId,
  tError,
}: {
  activeConversationId: string | null
  isAgentChatRoute: boolean
  isCancelled: () => boolean
  latestActionState: AgentChatActionState
  routeConversationId?: string
  tError: string
}) {
  try {
    if (isCancelled()) return
    if (!isAgentChatRoute) {
      await loadAgentConversationList(latestActionState, isCancelled)
      return
    }
    if (routeConversationId) {
      if (routeConversationId === activeConversationId) {
        await loadAgentConversationList(latestActionState, isCancelled)
        return
      }
      await loadRouteConversation(routeConversationId, {
        isCancelled,
        latestActionState,
      })
      return
    }
    await loadAgentConversationList(latestActionState, isCancelled)
    if (!isCancelled()) resetActiveConversation(latestActionState)
  } catch (err) {
    if (isCancelled()) return
    latestActionState.setError(err instanceof Error ? err.message : tError)
  }
}

async function loadRouteConversation(
  routeConversationId: string,
  {
    isCancelled,
    latestActionState,
  }: {
    isCancelled: () => boolean
    latestActionState: AgentChatActionState
  }
) {
  await Promise.all([
    loadAgentConversationList(latestActionState, isCancelled),
    loadAgentConversationById(routeConversationId, latestActionState, isCancelled),
  ])
}
