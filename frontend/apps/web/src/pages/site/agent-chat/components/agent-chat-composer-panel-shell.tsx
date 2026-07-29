import { useEffect, useState, type CSSProperties, type ReactNode } from "react"
import { useTranslation } from "react-i18next"
import type { AgentChatComposerPanelPlacement } from "../view-model/use-agent-chat-composer-panel-layout"
import { ComposerPanelCloseProvider } from "./composer-panel-layout"

const PANEL_EXIT_DURATION_MS = 180
const PREFERRED_PANEL_HEIGHT = 500

export function AgentChatComposerPanelShell({
  children,
  maxHeight,
  onClose,
  open,
  placement = "top",
  size = "default",
}: {
  children: ReactNode
  maxHeight: number
  onClose?: () => void
  open: boolean
  placement?: AgentChatComposerPanelPlacement
  size?: "default" | "wide"
}) {
  const { t } = useTranslation()
  const closeLabel = t("actions.close", { ns: "common" })
  const [shouldRender, setShouldRender] = useState(open)

  useEffect(() => {
    if (open) {
      if (shouldRender) return

      const frameId = window.requestAnimationFrame(() => {
        setShouldRender(true)
      })
      return () => window.cancelAnimationFrame(frameId)
    }

    if (!shouldRender) {
      return
    }

    const timeoutId = window.setTimeout(() => {
      setShouldRender(false)
    }, PANEL_EXIT_DURATION_MS)

    return () => window.clearTimeout(timeoutId)
  }, [open, shouldRender])

  if (!shouldRender) return null
  const panelHeight = Math.min(maxHeight, PREFERRED_PANEL_HEIGHT)
  const panelStyle = {
    "--agent-chat-composer-panel-max-height": `${maxHeight}px`,
    height: panelHeight,
    maxHeight,
  } as CSSProperties
  const positionClassName =
    placement === "bottom"
      ? "top-[calc(100%+1.25rem)] origin-top"
      : "bottom-[calc(100%+1.25rem)] origin-bottom"
  // Keep floating controls inside the composer that opened them. The sidebar can
  // change width independently of the browser viewport, so viewport math here
  // can otherwise place the panel beyond the readable content region.
  const widthClassName = "inset-x-0"
  const hiddenTransformClassName =
    placement === "bottom" ? "-translate-y-3" : "translate-y-3"

  return (
    <div
      aria-hidden={!open}
      className={`absolute z-50 overflow-hidden rounded-2xl border border-(--agent-chat-soft-border) bg-(--agent-chat-elevated-strong) bg-(image:--agent-chat-card-background) shadow-(--agent-chat-card-shadow) backdrop-blur-xl transition-[opacity,transform] duration-200 ease-out ${widthClassName} ${positionClassName} ${
        open
          ? "translate-y-0 scale-100 opacity-100"
          : `pointer-events-none ${hiddenTransformClassName} scale-[0.97] opacity-0`
      }`}
      data-composer-interactive="true"
      data-agent-chat-composer-panel-shell="true"
      data-agent-chat-composer-panel-placement={placement}
      data-agent-chat-composer-panel-size={size}
      style={panelStyle}
    >
      <div className="relative flex max-h-(--agent-chat-composer-panel-max-height) min-h-0 flex-col overflow-hidden">
        <div
          className="flex max-h-[calc(var(--agent-chat-composer-panel-max-height)-1.5rem)] min-h-0 flex-1 flex-col overflow-hidden p-4"
          data-agent-chat-composer-panel-content="true"
        >
          {onClose ? (
            <ComposerPanelCloseProvider label={closeLabel} onClose={onClose}>
              {children}
            </ComposerPanelCloseProvider>
          ) : (
            children
          )}
        </div>
      </div>
    </div>
  )
}
