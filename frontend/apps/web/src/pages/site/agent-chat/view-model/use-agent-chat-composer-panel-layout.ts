import { useLayoutEffect, useState, type RefObject } from "react"

const DEFAULT_PANEL_TOP_SAFE_SPACE = 96
const STATUS_CARD_GAP = 10
const PANEL_VIEWPORT_GAP = 20
const MIN_PANEL_HEIGHT = 220
const FALLBACK_PANEL_HEIGHT = 640

export type AgentChatComposerPanelPlacement = "top" | "bottom"

export interface AgentChatComposerPanelLayout {
  maxHeight: number
  placement: AgentChatComposerPanelPlacement
}

export function useAgentChatComposerPanelLayout(
  composerRootRef: RefObject<HTMLElement | null>,
  placementPreference: AgentChatComposerPanelPlacement | "auto" = "auto"
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
        placementPreference === "auto"
          ? availableBelow > availableAbove
            ? "bottom"
            : "top"
          : placementPreference
      const availableHeight =
        placement === "bottom" ? availableBelow : availableAbove
      const viewportLimit =
        viewportHeight - topSafeSpace - PANEL_VIEWPORT_GAP * 2

      setPanelLayout({
        maxHeight: Math.max(
          MIN_PANEL_HEIGHT,
          Math.min(
            FALLBACK_PANEL_HEIGHT,
            Math.floor(Math.max(MIN_PANEL_HEIGHT, availableHeight)),
            Math.floor(Math.max(MIN_PANEL_HEIGHT, viewportLimit))
          )
        ),
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
  }, [composerRootRef, placementPreference])

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
