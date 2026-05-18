import { Check, Megaphone } from "lucide-react"
import { useMemo, useState } from "react"
import { useTranslation } from "react-i18next"
import { Badge, SearchInput } from "@workspace/ui-components"
import {
  campaignStatusTranslationKey,
  filterCampaigns,
  summarizeCampaigns,
} from "../domain/agent-chat-campaign-context"
import type { AgentChatCampaignContextSelection } from "../domain/agent-chat-context-selection"

export function CampaignContextPicker({
  availableCampaigns,
  disabled,
  isLoadingCampaigns,
  onClearCampaignContext,
  onToggleCampaignContext,
  selectedCampaigns,
}: {
  availableCampaigns: AgentChatCampaignContextSelection[]
  disabled: boolean
  isLoadingCampaigns: boolean
  onClearCampaignContext: () => void
  onToggleCampaignContext: (campaign: AgentChatCampaignContextSelection) => void
  selectedCampaigns: AgentChatCampaignContextSelection[]
}) {
  const { t } = useTranslation()
  const [query, setQuery] = useState("")
  const filteredCampaigns = useMemo(
    () => filterCampaigns(availableCampaigns, query),
    [availableCampaigns, query]
  )
  const statusSummary = useMemo(
    () => summarizeCampaigns(availableCampaigns),
    [availableCampaigns]
  )

  return (
    <>
      <div className="mb-3 grid gap-2">
        <div className="flex min-w-0 flex-wrap items-center justify-between gap-2">
          <div className="min-w-0">
            <p className="text-xs font-medium text-foreground">
              {selectedCampaigns.length
                ? t("agentChat.composer.campaignContext.selectedCount", {
                    count: selectedCampaigns.length,
                  })
                : t("agentChat.composer.campaignContext.noneSelected")}
            </p>
            <p className="mt-0.5 text-[11px] text-muted-foreground">
              {selectedCampaigns.length
                ? t("agentChat.composer.campaignContext.selectedHint")
                : t("agentChat.composer.campaignContext.manualHint")}
            </p>
          </div>
          <div className="flex shrink-0 flex-wrap items-center gap-1.5">
            {statusSummary.map(({ count, status }) => (
              <Badge key={status} variant="outline">
                {t(campaignStatusTranslationKey(status))}
                {count}
              </Badge>
            ))}
          </div>
        </div>
        <SearchInput
          value={query}
          onValueChange={setQuery}
          placeholder={t("agentChat.composer.campaignContext.searchPlaceholder")}
          disabled={disabled || isLoadingCampaigns}
        />
        {selectedCampaigns.length ? (
          <div className="flex justify-end">
            <button
              type="button"
              disabled={disabled}
              onClick={onClearCampaignContext}
              className="rounded-full border border-border bg-background px-2.5 py-1 text-[11px] text-muted-foreground transition-colors hover:bg-muted hover:text-foreground disabled:cursor-not-allowed disabled:opacity-50"
            >
              {t("agentChat.composer.campaignContext.clear")}
            </button>
          </div>
        ) : null}
      </div>
      <div className="grid max-h-72 min-w-0 gap-2 overflow-y-auto pr-1 sm:grid-cols-2">
        <CampaignPickerContent
          availableCampaigns={filteredCampaigns}
          disabled={disabled}
          isLoadingCampaigns={isLoadingCampaigns}
          onToggleCampaignContext={onToggleCampaignContext}
          query={query}
          selectedCampaigns={selectedCampaigns}
        />
      </div>
    </>
  )
}

function CampaignPickerContent({
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

function CampaignContextButton({
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
        <span className="mt-1 flex min-w-0 flex-wrap gap-1.5">
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
