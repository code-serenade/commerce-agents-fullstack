import { ChevronDown, Megaphone } from "lucide-react"
import { useState } from "react"
import { useTranslation } from "react-i18next"
import type { AgentChatCampaignContextSelection } from "../domain/agent-chat-context-selection"
import { CampaignContextPicker } from "./campaign-context-picker"

export interface AgentChatCampaignContextBarProps {
  availableCampaigns: AgentChatCampaignContextSelection[]
  disabled: boolean
  isLoadingCampaigns: boolean
  isOpen?: boolean
  onClearCampaignContext: () => void
  onOpenChange?: (open: boolean) => void
  onToggleCampaignContext: (campaign: AgentChatCampaignContextSelection) => void
  selectedCampaigns: AgentChatCampaignContextSelection[]
  variant?: "compact" | "panel"
}

export function AgentChatCampaignContextBar({
  availableCampaigns,
  disabled,
  isLoadingCampaigns,
  isOpen,
  onClearCampaignContext,
  onOpenChange,
  onToggleCampaignContext,
  selectedCampaigns,
  variant = "compact",
}: AgentChatCampaignContextBarProps) {
  const { t } = useTranslation()
  const [internalOpen, setInternalOpen] = useState(false)
  const open = isOpen ?? internalOpen
  const selectedCampaignNames = selectedCampaigns.map((campaign) => campaign.name)
  const selectedCampaignSummary =
    selectedCampaigns.length > 0
      ? t("agentChat.composer.campaignContext.selectedCount", {
          count: selectedCampaigns.length,
        })
      : t("agentChat.composer.campaignContext.noneSelected")
  const title = selectedCampaigns.length
    ? `${t("agentChat.composer.campaignContext.compactTitle")} - ${selectedCampaignNames.join(" / ")}`
    : `${t("agentChat.composer.campaignContext.compactTitle")} - ${selectedCampaignSummary}`

  const toggleOpen = () => {
    const nextOpen = !open
    if (isOpen === undefined) setInternalOpen(nextOpen)
    onOpenChange?.(nextOpen)
  }

  if (variant === "panel") {
    return (
      <CampaignContextPicker
        availableCampaigns={availableCampaigns}
        disabled={disabled}
        isLoadingCampaigns={isLoadingCampaigns}
        onClearCampaignContext={onClearCampaignContext}
        onToggleCampaignContext={onToggleCampaignContext}
        selectedCampaigns={selectedCampaigns}
      />
    )
  }

  return (
    <div className="relative shrink-0" data-composer-interactive="true">
      <button
        type="button"
        onClick={toggleOpen}
        className={`flex h-7 max-w-28 shrink-0 items-center gap-1.5 rounded-full border px-2.5 text-xs font-medium transition-colors sm:max-w-36 ${
          open
            ? "border-sky-400/80 bg-sky-50 text-sky-700 dark:bg-sky-500/15 dark:text-sky-100"
            : "border-border bg-card text-muted-foreground hover:bg-muted hover:text-foreground"
        }`}
        aria-expanded={open}
        aria-label={title}
        title={title}
      >
        <Megaphone className="size-3.5 shrink-0" />
        <span className="truncate">
          {selectedCampaigns.length
            ? t("agentChat.composer.campaignContext.compactLabel")
            : t("agentChat.composer.campaignContext.noneCompactLabel")}
        </span>
        {selectedCampaigns.length ? (
          <span className="flex size-4 shrink-0 items-center justify-center rounded-full bg-current/10 text-[10px]">
            {selectedCampaigns.length}
          </span>
        ) : null}
        <ChevronDown
          className={`size-3.5 shrink-0 transition-transform ${
            open ? "rotate-180" : ""
          }`}
        />
      </button>
    </div>
  )
}
