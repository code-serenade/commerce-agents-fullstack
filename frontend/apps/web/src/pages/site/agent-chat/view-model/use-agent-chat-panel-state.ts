import { useCallback, useState } from "react"

export type AgentChatComposerPanelId = "product" | "settings" | "source"

export function useAgentChatPanelState() {
  const [activePanel, setActivePanel] = useState<AgentChatComposerPanelId | null>(
    null
  )

  const closePanels = useCallback(() => {
    setActivePanel(null)
  }, [])

  const setProductContextOpen = useCallback((open: boolean) => {
    setActivePanel(open ? "product" : null)
  }, [])

  const setSourceContextOpen = useCallback((open: boolean) => {
    setActivePanel(open ? "source" : null)
  }, [])

  const toggleSettingsPanel = useCallback(() => {
    setActivePanel((current) => (current === "settings" ? null : "settings"))
  }, [])

  return {
    closePanels,
    productContextOpen: activePanel === "product",
    setProductContextOpen,
    setSourceContextOpen,
    settingsOpen: activePanel === "settings",
    sourceContextOpen: activePanel === "source",
    toggleSettingsPanel,
  }
}
