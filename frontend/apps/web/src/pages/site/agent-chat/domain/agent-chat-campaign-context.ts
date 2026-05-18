import type { AgentChatCampaignContextSelection } from "./agent-chat-context-selection"

const CAMPAIGN_STATUS_KEYS = new Set([
  "active",
  "draft",
  "paused",
  "ended",
  "archived",
])

export function filterCampaigns(
  campaigns: AgentChatCampaignContextSelection[],
  query: string
) {
  const normalizedQuery = query.trim().toLowerCase()
  if (!normalizedQuery) return campaigns

  return campaigns.filter((campaign) =>
    [
      campaign.name,
      campaign.campaignType,
      campaign.status,
      campaign.startDate,
      campaign.endDate,
      campaign.budget === undefined || campaign.budget === null
        ? null
        : String(campaign.budget),
    ]
      .filter(Boolean)
      .some((value) => String(value).toLowerCase().includes(normalizedQuery))
  )
}

export function summarizeCampaigns(
  campaigns: AgentChatCampaignContextSelection[]
) {
  const counts = new Map<string, number>()
  campaigns.forEach((campaign) => {
    const status = campaign.status || "unknown"
    counts.set(status, (counts.get(status) ?? 0) + 1)
  })

  return Array.from(counts.entries())
    .map(([status, count]) => ({ count, status }))
    .sort((left, right) => left.status.localeCompare(right.status))
}

export function campaignStatusTranslationKey(status: string) {
  const normalized = status.trim().toLowerCase()
  return `agentChat.composer.campaignContext.status.${
    CAMPAIGN_STATUS_KEYS.has(normalized) ? normalized : "unknown"
  }`
}
