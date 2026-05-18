import type { Dispatch, SetStateAction } from "react"
import { agentChatData } from "../agent-chat-data"
import {
  mapCampaignToContextSelection,
  restoreCampaignContextSelection,
  toggleCampaignContextSelection,
  type AgentChatCampaignContextSelection,
} from "../domain/agent-chat-context-selection"

export interface AgentChatCampaignContextState {
  setAvailableCampaigns: Dispatch<
    SetStateAction<AgentChatCampaignContextSelection[]>
  >
  setIsLoadingCampaigns: Dispatch<SetStateAction<boolean>>
  setSelectedCampaigns: Dispatch<
    SetStateAction<AgentChatCampaignContextSelection[]>
  >
}

export function resetAgentChatCampaignContext(
  state: AgentChatCampaignContextState
) {
  state.setAvailableCampaigns([])
  state.setSelectedCampaigns([])
  state.setIsLoadingCampaigns(false)
}

export function restoreAgentChatCampaignContext(
  state: Pick<AgentChatCampaignContextState, "setSelectedCampaigns">,
  availableCampaigns: AgentChatCampaignContextSelection[],
  campaignIds: string[]
) {
  state.setSelectedCampaigns(
    restoreCampaignContextSelection(availableCampaigns, campaignIds)
  )
}

export function loadAgentChatCampaignContext(
  state: AgentChatCampaignContextState,
  isCancelled: () => boolean
) {
  state.setIsLoadingCampaigns(true)
  return Promise.all([
    agentChatData.listCampaigns({ status: "active" }),
    agentChatData.listCampaigns({ status: "draft" }),
    agentChatData.listCampaigns({ status: "paused" }),
  ])
    .then((campaignGroups) => {
      if (isCancelled()) return
      const mapped = campaignGroups
        .flatMap((group) => group.campaigns)
        .map(mapCampaignToContextSelection)
        .filter(
          (campaign, index, items) =>
            items.findIndex((item) => item.id === campaign.id) === index
        )
      state.setAvailableCampaigns(mapped)
      state.setSelectedCampaigns((current) =>
        current.filter((selected) =>
          mapped.some((campaign) => campaign.id === selected.id)
        )
      )
    })
    .catch(() => {
      if (isCancelled()) return
      state.setAvailableCampaigns([])
      state.setSelectedCampaigns([])
    })
    .finally(() => {
      if (!isCancelled()) state.setIsLoadingCampaigns(false)
    })
}

export function toggleAgentChatCampaignContext(
  state: Pick<AgentChatCampaignContextState, "setSelectedCampaigns">,
  campaign: AgentChatCampaignContextSelection
) {
  state.setSelectedCampaigns((current) =>
    toggleCampaignContextSelection(current, campaign)
  )
}
