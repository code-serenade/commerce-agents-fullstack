import { useEffect } from "react"
import type { RefObject } from "react"

export function useAgentChatPanelOutsideClose({
  closePanels,
  composerRootRef,
  hasOpenPanel,
}: {
  closePanels: () => void
  composerRootRef: RefObject<HTMLDivElement | null>
  hasOpenPanel: boolean
}) {
  useEffect(() => {
    if (!hasOpenPanel) return

    const closePanelsOnOutsideEvent = (event: Event) => {
      const target = event.target
      if (!(target instanceof Node)) return
      if (composerRootRef.current?.contains(target)) return
      closePanels()
    }

    document.addEventListener("pointerdown", closePanelsOnOutsideEvent, true)
    document.addEventListener("click", closePanelsOnOutsideEvent, true)
    return () => {
      document.removeEventListener(
        "pointerdown",
        closePanelsOnOutsideEvent,
        true
      )
      document.removeEventListener("click", closePanelsOnOutsideEvent, true)
    }
  }, [closePanels, composerRootRef, hasOpenPanel])
}
