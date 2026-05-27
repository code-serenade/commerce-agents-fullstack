import { useMemo, useState } from "react"
import { useTranslation } from "react-i18next"
import { Badge, SearchInput } from "@workspace/ui-components"
import {
  campaignStatusTranslationKey,
  filterCampaigns,
  summarizeCampaigns,
} from "../domain/agent-chat-campaign-context"
import type { AgentChatCampaignContextSelection } from "../domain/agent-chat-context-selection"
import {
  CampaignContextButton,
  CampaignPickerContent,
} from "./campaign-context-picker-parts"

const CAMPAIGN_PAGE_SIZE = 6

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
  const [visibleCount, setVisibleCount] = useState(CAMPAIGN_PAGE_SIZE)
  const filteredCampaigns = useMemo(
    () => filterCampaigns(availableCampaigns, query),
    [availableCampaigns, query]
  )
  const visibleCampaigns = filteredCampaigns.slice(0, visibleCount)
  const hiddenCampaignCount = Math.max(
    filteredCampaigns.length - visibleCampaigns.length,
    0
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
          onValueChange={(value) => {
            setQuery(value)
            setVisibleCount(CAMPAIGN_PAGE_SIZE)
          }}
          placeholder={t("agentChat.composer.campaignContext.searchPlaceholder")}
          disabled={disabled || isLoadingCampaigns}
        />
        {selectedCampaigns.length ? (
          <div className="grid gap-2 rounded-lg border border-border bg-muted/25 p-2">
            <div className="flex min-w-0 items-center justify-between gap-2">
              <p className="text-[11px] font-semibold text-foreground">
                {t("agentChat.composer.campaignContext.selectedSection")}
              </p>
              <button
                type="button"
                disabled={disabled}
                onClick={onClearCampaignContext}
                className="rounded-full border border-border bg-background px-2.5 py-1 text-[11px] text-muted-foreground transition-colors hover:bg-muted hover:text-foreground disabled:cursor-not-allowed disabled:opacity-50"
              >
                {t("agentChat.composer.campaignContext.clear")}
              </button>
            </div>
            <div className="grid min-w-0 gap-1.5 sm:grid-cols-2">
              {selectedCampaigns.map((campaign) => (
                <CampaignContextButton
                  key={campaign.id}
                  campaign={campaign}
                  disabled={disabled}
                  onToggleCampaignContext={onToggleCampaignContext}
                  selected
                />
              ))}
            </div>
          </div>
        ) : null}
      </div>
      <div className="grid min-w-0 gap-2 sm:grid-cols-2">
        <CampaignPickerContent
          availableCampaigns={visibleCampaigns}
          disabled={disabled}
          isLoadingCampaigns={isLoadingCampaigns}
          onToggleCampaignContext={onToggleCampaignContext}
          query={query}
          selectedCampaigns={selectedCampaigns}
        />
      </div>
      {hiddenCampaignCount > 0 ? (
        <div className="mt-2 flex justify-center">
          <button
            type="button"
            disabled={disabled || isLoadingCampaigns}
            onClick={() =>
              setVisibleCount((current) => current + CAMPAIGN_PAGE_SIZE)
            }
            className="rounded-full border border-border bg-background px-3 py-1 text-[11px] font-medium text-muted-foreground transition-colors hover:bg-muted hover:text-foreground disabled:cursor-not-allowed disabled:opacity-50"
          >
            {t("agentChat.composer.campaignContext.showMore", {
              count: Math.min(hiddenCampaignCount, CAMPAIGN_PAGE_SIZE),
            })}
          </button>
        </div>
      ) : null}
    </>
  )
}

