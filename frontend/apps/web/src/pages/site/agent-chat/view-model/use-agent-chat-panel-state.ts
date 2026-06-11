import { useCallback, useState } from "react"

export type AgentChatComposerPanelId =
  | "campaign"
  | "contextPreview"
  | "product"
  | "settings"
  | "source"

export function useAgentChatPanelState({
  requestedPanel = null,
  requestedPanelVersion = 0,
}: {
  requestedPanel?: AgentChatComposerPanelId | null
  requestedPanelVersion?: number
} = {}) {
  const [activePanel, setActivePanel] =
    useState<AgentChatComposerPanelId | null>(() =>
      requestedPanelVersion ? requestedPanel : null
    )

  const closePanels = useCallback(() => {
    setActivePanel(null)
  }, [])

  const setProductContextOpen = useCallback((open: boolean) => {
    setActivePanel(open ? "product" : null)
  }, [])

  const setCampaignContextOpen = useCallback((open: boolean) => {
    setActivePanel(open ? "campaign" : null)
  }, [])

  const setSourceContextOpen = useCallback((open: boolean) => {
    setActivePanel(open ? "source" : null)
  }, [])

  const setContextPreviewOpen = useCallback((open: boolean) => {
    setActivePanel(open ? "contextPreview" : null)
  }, [])

  const toggleSettingsPanel = useCallback(() => {
    setActivePanel((current) => (current === "settings" ? null : "settings"))
  }, [])

  return {
    campaignContextOpen: activePanel === "campaign",
    closePanels,
    contextPreviewOpen: activePanel === "contextPreview",
    productContextOpen: activePanel === "product",
    setCampaignContextOpen,
    setContextPreviewOpen,
    setProductContextOpen,
    setSourceContextOpen,
    settingsOpen: activePanel === "settings",
    sourceContextOpen: activePanel === "source",
    toggleSettingsPanel,
  }
}
