import { useEffect, useState, type CSSProperties, type RefObject } from "react"
import type { AgentChatComposerPanelPlacement } from "./use-agent-chat-composer-panel-layout"

const BOTTOM_PANEL_RESERVE_GAP = 20

export function useAgentChatBottomPanelReserve({
  composerRootRef,
  hasOpenPanel,
  placement,
}: {
  composerRootRef: RefObject<HTMLElement | null>
  hasOpenPanel: boolean
  placement: AgentChatComposerPanelPlacement
}) {
  const [bottomPanelReserve, setBottomPanelReserve] = useState(0)

  useEffect(() => {
    const shouldReservePanelSpace = placement === "bottom" && hasOpenPanel
    if (!shouldReservePanelSpace) {
      const frameId = window.requestAnimationFrame(() => {
        setBottomPanelReserve(0)
      })
      return () => window.cancelAnimationFrame(frameId)
    }

    const composerRoot = composerRootRef.current
    if (!composerRoot) return

    let frameId = 0
    let resizeObserver: ResizeObserver | null = null
    const readOpenPanel = () =>
      composerRoot.querySelector<HTMLElement>(
        "[data-agent-chat-composer-panel-shell='true'][aria-hidden='false']"
      )
    const updateReserve = () => {
      window.cancelAnimationFrame(frameId)
      frameId = window.requestAnimationFrame(() => {
        const panelHeight = readOpenPanel()?.getBoundingClientRect().height ?? 0
        setBottomPanelReserve(
          panelHeight > 0
            ? Math.ceil(panelHeight) + BOTTOM_PANEL_RESERVE_GAP
            : 0
        )
      })
    }
    const observeOpenPanel = () => {
      resizeObserver?.disconnect()
      resizeObserver =
        typeof ResizeObserver === "undefined"
          ? null
          : new ResizeObserver(updateReserve)
      const openPanel = readOpenPanel()
      if (openPanel) resizeObserver?.observe(openPanel)
    }
    const refreshPanelReserve = () => {
      observeOpenPanel()
      updateReserve()
    }
    const mutationObserver =
      typeof MutationObserver === "undefined"
        ? null
        : new MutationObserver(refreshPanelReserve)

    refreshPanelReserve()
    mutationObserver?.observe(composerRoot, {
      attributeFilter: ["aria-hidden", "class", "style"],
      attributes: true,
      childList: true,
      subtree: true,
    })
    window.addEventListener("resize", refreshPanelReserve)

    return () => {
      window.cancelAnimationFrame(frameId)
      mutationObserver?.disconnect()
      resizeObserver?.disconnect()
      window.removeEventListener("resize", refreshPanelReserve)
    }
  }, [composerRootRef, hasOpenPanel, placement])

  return bottomPanelReserve > 0
    ? ({ marginBottom: bottomPanelReserve } satisfies CSSProperties)
    : undefined
}
