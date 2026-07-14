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
import {
  ComposerPanelFrame,
  composerPanelListClassName,
  composerPanelSelectedAreaClassName,
  composerPanelTinyButtonClassName,
} from "./composer-panel-layout"

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
      <ComposerPanelFrame
        title={
          selectedCampaigns.length
            ? t("agentChat.composer.campaignContext.selectedCount", {
                count: selectedCampaigns.length,
              })
            : t("agentChat.composer.campaignContext.noneSelected")
        }
        description={
          selectedCampaigns.length
            ? t("agentChat.composer.campaignContext.selectedHint")
            : t("agentChat.composer.campaignContext.manualHint")
        }
        actions={
          <>
            {statusSummary.map(({ count, status }) => (
              <Badge key={status} variant="outline">
                {t(campaignStatusTranslationKey(status))}
                {count}
              </Badge>
            ))}
          </>
        }
      >
        <div className="mb-3 grid gap-0">
          <SearchInput
            value={query}
            onValueChange={(value) => {
              setQuery(value)
              setVisibleCount(CAMPAIGN_PAGE_SIZE)
            }}
            placeholder={t(
              "agentChat.composer.campaignContext.searchPlaceholder"
            )}
            disabled={disabled || isLoadingCampaigns}
          />
          {selectedCampaigns.length ? (
            <div className={composerPanelSelectedAreaClassName}>
              <div className="flex min-w-0 items-center justify-between gap-2">
                <p className="text-[11px] font-semibold text-foreground">
                  {t("agentChat.composer.campaignContext.selectedSection")}
                </p>
                <button
                  type="button"
                  disabled={disabled}
                  onClick={onClearCampaignContext}
                  className={composerPanelTinyButtonClassName}
                >
                  {t("agentChat.composer.campaignContext.clear")}
                </button>
              </div>
              <div className={composerPanelListClassName}>
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
        <div className={composerPanelListClassName}>
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
              className={composerPanelTinyButtonClassName}
            >
              {t("agentChat.composer.campaignContext.showMore", {
                count: Math.min(hiddenCampaignCount, CAMPAIGN_PAGE_SIZE),
              })}
            </button>
          </div>
        ) : null}
      </ComposerPanelFrame>
    </>
  )
}
