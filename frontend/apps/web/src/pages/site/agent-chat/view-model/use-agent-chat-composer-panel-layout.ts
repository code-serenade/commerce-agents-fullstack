import { useLayoutEffect, useState, type RefObject } from "react"

const DEFAULT_PANEL_TOP_SAFE_SPACE = 120
const STATUS_CARD_GAP = 12
const PANEL_BOTTOM_GAP = 20
const MIN_PANEL_HEIGHT = 160
const FALLBACK_PANEL_HEIGHT = 560

export function useAgentChatComposerPanelMaxHeight(
  composerRootRef: RefObject<HTMLElement | null>
) {
  const [panelMaxHeight, setPanelMaxHeight] = useState(FALLBACK_PANEL_HEIGHT)

  useLayoutEffect(() => {
    const updatePanelMaxHeight = () => {
      const composerRoot = composerRootRef.current
      if (!composerRoot) return

      const viewportHeight =
        window.visualViewport?.height ?? window.innerHeight
      const rootTop = composerRoot.getBoundingClientRect().top
      const topSafeSpace = readPanelTopSafeSpace()
      const visibleRootTop = Math.min(rootTop, viewportHeight)
      const availableHeight =
        visibleRootTop - topSafeSpace - PANEL_BOTTOM_GAP
      setPanelMaxHeight(
        Math.max(MIN_PANEL_HEIGHT, Math.floor(availableHeight))
      )
    }

    updatePanelMaxHeight()
    window.addEventListener("resize", updatePanelMaxHeight)
    window.visualViewport?.addEventListener("resize", updatePanelMaxHeight)

    const observer =
      typeof ResizeObserver === "undefined"
        ? null
        : new ResizeObserver(updatePanelMaxHeight)
    if (composerRootRef.current) observer?.observe(composerRootRef.current)

    return () => {
      window.removeEventListener("resize", updatePanelMaxHeight)
      window.visualViewport?.removeEventListener("resize", updatePanelMaxHeight)
      observer?.disconnect()
    }
  }, [composerRootRef])

  return panelMaxHeight
}

function readPanelTopSafeSpace() {
  const statusCards = Array.from(
    document.querySelectorAll("[data-agent-chat-status-card='true']")
  )
  const visibleBottoms = statusCards
    .map((card) => card.getBoundingClientRect())
    .filter((rect) => rect.width > 0 && rect.height > 0)
    .map((rect) => rect.bottom)

  if (!visibleBottoms.length) return DEFAULT_PANEL_TOP_SAFE_SPACE

  return Math.max(DEFAULT_PANEL_TOP_SAFE_SPACE, ...visibleBottoms) + STATUS_CARD_GAP
}
