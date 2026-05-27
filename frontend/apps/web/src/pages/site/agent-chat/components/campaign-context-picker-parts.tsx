import { Check, Megaphone } from "lucide-react"
import { useTranslation } from "react-i18next"
import { campaignStatusTranslationKey } from "../domain/agent-chat-campaign-context"
import type { AgentChatCampaignContextSelection } from "../domain/agent-chat-context-selection"

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
      <span className="rounded-full border border-border bg-background/70 px-3 py-1.5 text-xs text-muted-foreground">
        {t("agentChat.composer.campaignContext.loading")}
      </span>
    )
  }

  if (!availableCampaigns.length) {
    return (
      <div className="col-span-full rounded-lg border border-dashed border-border bg-background/60 p-3 text-xs text-muted-foreground">
        {query.trim()
          ? t("agentChat.composer.campaignContext.noSearchResult")
          : t("agentChat.composer.campaignContext.empty")}
      </div>
    )
  }

  return (
    <>
      {selectedCampaigns.length === 0 ? (
        <div className="col-span-full flex items-start gap-2 rounded-lg border border-dashed border-border bg-background/60 p-3 text-xs text-muted-foreground">
          <Megaphone className="mt-0.5 size-4 shrink-0" />
          <span>{t("agentChat.composer.campaignContext.noneHint")}</span>
        </div>
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
      className={`flex min-w-0 items-start gap-2 rounded-md border px-2.5 py-1.5 text-left text-xs transition-colors disabled:cursor-not-allowed disabled:opacity-50 ${
        selected
          ? "border-sky-500/70 bg-sky-50 text-sky-700 dark:bg-sky-500/15 dark:text-sky-100"
          : "border-border bg-background/70 text-muted-foreground hover:bg-muted hover:text-foreground"
      }`}
      title={campaign.name}
    >
      {selected ? (
        <span className="mt-0.5 flex size-4 shrink-0 items-center justify-center rounded-[4px] border border-sky-500 bg-sky-500 text-white">
          <Check className="size-3" />
        </span>
      ) : (
        <span className="mt-0.5 size-4 shrink-0 rounded-[4px] border border-border bg-background shadow-inner" />
      )}
      <span className="min-w-0">
        <span className="block truncate font-medium text-foreground">
          {campaign.name}
        </span>
        <span className="mt-0.5 flex min-w-0 flex-wrap gap-1">
          {readCampaignMeta(campaign, t).map((item) => (
            <span
              key={item}
              className="rounded-full bg-muted px-1.5 py-0.5 text-[10px] text-muted-foreground"
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
