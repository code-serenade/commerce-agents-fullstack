import { Check, Megaphone } from "lucide-react"
import { useTranslation } from "react-i18next"
import { campaignStatusTranslationKey } from "../domain/agent-chat-campaign-context"
import type { AgentChatCampaignContextSelection } from "../domain/agent-chat-context-selection"
import { ComposerPanelEmptyState } from "./composer-panel-layout"

export function CampaignPickerContent({
  availableCampaigns,
  disabled,
  isLoadingCampaigns,
  onToggleCampaignContext,
  query,
  selectedCampaigns,
}: {
  availableCampaigns: AgentChatCampaignContextSelection[]
  disabled: boolean
  isLoadingCampaigns: boolean
  onToggleCampaignContext: (campaign: AgentChatCampaignContextSelection) => void
  query: string
  selectedCampaigns: AgentChatCampaignContextSelection[]
}) {
  const { t } = useTranslation()

  if (isLoadingCampaigns) {
    return (
      <span className="rounded-full border border-(--agent-chat-soft-border) bg-(--agent-chat-elevated-strong) px-3 py-1.5 text-xs text-muted-foreground">
        {t("agentChat.composer.campaignContext.loading")}
      </span>
    )
  }

  if (!availableCampaigns.length) {
    return (
      <ComposerPanelEmptyState>
        {query.trim()
          ? t("agentChat.composer.campaignContext.noSearchResult")
          : t("agentChat.composer.campaignContext.empty")}
      </ComposerPanelEmptyState>
    )
  }

  return (
    <>
      {selectedCampaigns.length === 0 ? (
        <ComposerPanelEmptyState icon={<Megaphone className="size-4" />}>
          {t("agentChat.composer.campaignContext.noneHint")}
        </ComposerPanelEmptyState>
      ) : null}
      {availableCampaigns.map((campaign) => (
        <CampaignContextButton
          key={campaign.id}
          campaign={campaign}
          disabled={disabled}
          onToggleCampaignContext={onToggleCampaignContext}
          selected={selectedCampaigns.some((item) => item.id === campaign.id)}
        />
      ))}
    </>
  )
}

export function CampaignContextButton({
  campaign,
  disabled,
  onToggleCampaignContext,
  selected,
}: {
  campaign: AgentChatCampaignContextSelection
  disabled: boolean
  onToggleCampaignContext: (campaign: AgentChatCampaignContextSelection) => void
  selected: boolean
}) {
  const { t } = useTranslation()

  return (
    <button
      type="button"
      disabled={disabled}
      onClick={() => onToggleCampaignContext(campaign)}
      aria-pressed={selected}
      className={`flex min-w-0 items-start gap-2 rounded-lg border px-3 py-2 text-left text-xs transition-colors disabled:cursor-not-allowed disabled:opacity-50 ${
        selected
          ? "border-primary/50 bg-primary/10 text-primary"
          : "border-(--agent-chat-soft-border) bg-(--agent-chat-elevated-strong) text-muted-foreground hover:bg-(--agent-chat-accent-soft) hover:text-foreground"
      }`}
      title={campaign.name}
    >
      {selected ? (
        <span className="mt-0.5 flex size-4 shrink-0 items-center justify-center rounded-[4px] border border-primary bg-primary text-primary-foreground">
          <Check className="size-3" />
        </span>
      ) : (
        <span className="mt-0.5 size-4 shrink-0 rounded-[4px] border border-(--agent-chat-soft-border) bg-(--agent-chat-elevated-strong) shadow-inner" />
      )}
      <span className="min-w-0">
        <span className="block truncate font-medium text-foreground">
          {campaign.name}
        </span>
        <span className="mt-0.5 flex min-w-0 flex-wrap gap-1">
          {readCampaignMeta(campaign, t).map((item) => (
            <span
              key={item}
              className="rounded-full bg-(--agent-chat-subtle-surface) px-1.5 py-0.5 text-[10px] text-muted-foreground"
            >
              {item}
            </span>
          ))}
        </span>
      </span>
    </button>
  )
}

function readCampaignMeta(
  campaign: AgentChatCampaignContextSelection,
  t: ReturnType<typeof useTranslation>["t"]
) {
  const dateRange =
    campaign.startDate || campaign.endDate
      ? `${campaign.startDate ?? "-"} ~ ${campaign.endDate ?? "-"}`
      : null
  return [
    campaign.status ? t(campaignStatusTranslationKey(campaign.status)) : null,
    campaign.campaignType,
    campaign.budget === undefined || campaign.budget === null
      ? null
      : t("agentChat.composer.campaignContext.budget", {
          value: campaign.budget,
        }),
    dateRange,
  ]
    .filter((value): value is string => Boolean(value))
    .slice(0, 4)
}
