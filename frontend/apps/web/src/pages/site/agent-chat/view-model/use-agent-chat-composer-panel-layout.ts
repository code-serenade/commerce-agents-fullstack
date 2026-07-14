import { useLayoutEffect, useState, type RefObject } from "react"

const DEFAULT_PANEL_TOP_SAFE_SPACE = 120
const STATUS_CARD_GAP = 12
const PANEL_VIEWPORT_GAP = 20
const MIN_PANEL_HEIGHT = 160
const FALLBACK_PANEL_HEIGHT = 560

export type AgentChatComposerPanelPlacement = "top" | "bottom"

export interface AgentChatComposerPanelLayout {
  maxHeight: number
  placement: AgentChatComposerPanelPlacement
}

export function useAgentChatComposerPanelLayout(
  composerRootRef: RefObject<HTMLElement | null>
) {
  const [panelLayout, setPanelLayout] =
    useState<AgentChatComposerPanelLayout>({
      maxHeight: FALLBACK_PANEL_HEIGHT,
      placement: "top",
    })

  useLayoutEffect(() => {
    const updatePanelLayout = () => {
      const composerRoot = composerRootRef.current
      if (!composerRoot) return

      const viewportHeight =
        window.visualViewport?.height ?? window.innerHeight
      const rootRect = composerRoot.getBoundingClientRect()
      const topSafeSpace = readPanelTopSafeSpace()
      const availableAbove =
        Math.min(rootRect.top, viewportHeight) -
        topSafeSpace -
        PANEL_VIEWPORT_GAP
      const availableBelow =
        viewportHeight -
        Math.max(rootRect.bottom, 0) -
        PANEL_VIEWPORT_GAP
      const placement: AgentChatComposerPanelPlacement =
        availableBelow > availableAbove ? "bottom" : "top"
      const availableHeight =
        placement === "bottom" ? availableBelow : availableAbove

      setPanelLayout({
        maxHeight: Math.max(MIN_PANEL_HEIGHT, Math.floor(availableHeight)),
        placement,
      })
    }

    updatePanelLayout()
    window.addEventListener("resize", updatePanelLayout)
    window.visualViewport?.addEventListener("resize", updatePanelLayout)

    const observer =
      typeof ResizeObserver === "undefined"
        ? null
        : new ResizeObserver(updatePanelLayout)
    if (composerRootRef.current) observer?.observe(composerRootRef.current)

    return () => {
      window.removeEventListener("resize", updatePanelLayout)
      window.visualViewport?.removeEventListener("resize", updatePanelLayout)
      observer?.disconnect()
    }
  }, [composerRootRef])

  return panelLayout
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
