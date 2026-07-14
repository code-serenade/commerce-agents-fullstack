import { useLayoutEffect } from "react"
import type { RefObject } from "react"

export function useAgentChatThreadViewportHeight(
  scrollRootRef: RefObject<HTMLElement | null>
) {
  useLayoutEffect(() => {
    const root = scrollRootRef.current
    if (!root) return

    const syncThreadViewportHeight = () => {
      root.style.setProperty(
        "--agent-chat-thread-viewport-height",
        `${root.clientHeight}px`
      )
    }

    syncThreadViewportHeight()

    if (typeof ResizeObserver === "undefined") return

    const observer = new ResizeObserver(syncThreadViewportHeight)
    observer.observe(root)
    return () => observer.disconnect()
  }, [scrollRootRef])
}
