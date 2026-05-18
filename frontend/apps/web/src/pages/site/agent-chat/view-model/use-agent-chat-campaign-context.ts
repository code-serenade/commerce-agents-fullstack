import { useCallback, useEffect, useMemo, useRef, useState } from "react"
import type { AgentChatCampaignContextSelection } from "../domain/agent-chat-context-selection"
import {
  loadAgentChatCampaignContext,
  resetAgentChatCampaignContext,
  restoreAgentChatCampaignContext,
  toggleAgentChatCampaignContext,
} from "./agent-chat-campaign-context-actions"

export function useAgentChatCampaignContext(input: {
  enabled: boolean
  isStreaming: boolean
}) {
  const [availableCampaigns, setAvailableCampaigns] = useState<
    AgentChatCampaignContextSelection[]
  >([])
  const [selectedCampaigns, setSelectedCampaigns] = useState<
    AgentChatCampaignContextSelection[]
  >([])
  const [isLoadingCampaigns, setIsLoadingCampaigns] = useState(false)
  const pendingSelectedCampaignIdsRef = useRef<string[]>([])
  const state = useMemo(
    () => ({
      setAvailableCampaigns,
      setIsLoadingCampaigns,
      setSelectedCampaigns,
    }),
    []
  )

  useEffect(() => {
    let cancelled = false

    if (!input.enabled) {
      resetAgentChatCampaignContext(state)
      pendingSelectedCampaignIdsRef.current = []
      return
    }

    void loadAgentChatCampaignContext(state, () => cancelled)

    return () => {
      cancelled = true
    }
  }, [input.enabled, state])

  useEffect(() => {
    const pendingSelectedCampaignIds = pendingSelectedCampaignIdsRef.current
    if (isLoadingCampaigns || pendingSelectedCampaignIds.length === 0) return
    restoreAgentChatCampaignContext(
      state,
      availableCampaigns,
      pendingSelectedCampaignIds
    )
    pendingSelectedCampaignIdsRef.current = []
  }, [availableCampaigns, isLoadingCampaigns, state])

  const toggleCampaignContext = useCallback(
    (campaign: AgentChatCampaignContextSelection) => {
      if (input.isStreaming) return
      toggleAgentChatCampaignContext(state, campaign)
    },
    [input.isStreaming, state]
  )

  const clearSelectedCampaigns = useCallback(() => {
    if (input.isStreaming) return
    pendingSelectedCampaignIdsRef.current = []
    setSelectedCampaigns([])
  }, [input.isStreaming])

  const restoreSelectedCampaigns = useCallback(
    (campaignIds: string[]) => {
      if (input.isStreaming) return
      const uniqueCampaignIds = Array.from(new Set(campaignIds))
      pendingSelectedCampaignIdsRef.current = uniqueCampaignIds
      restoreAgentChatCampaignContext(
        state,
        availableCampaigns,
        uniqueCampaignIds
      )
    },
    [availableCampaigns, input.isStreaming, state]
  )

  return {
    availableCampaigns,
    clearSelectedCampaigns,
    isLoadingCampaigns,
    restoreSelectedCampaigns,
    selectedCampaigns,
    toggleCampaignContext,
  }
}
