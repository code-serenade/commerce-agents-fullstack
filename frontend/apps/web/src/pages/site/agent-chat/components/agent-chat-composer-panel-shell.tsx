import { useEffect, useState, type CSSProperties, type ReactNode } from "react"
import { useTranslation } from "react-i18next"
import { ComposerPanelCloseProvider } from "./composer-panel-layout"

const PANEL_EXIT_DURATION_MS = 180

export function AgentChatComposerPanelShell({
  children,
  maxHeight,
  onClose,
  open,
  size = "default",
}: {
  children: ReactNode
  maxHeight: number
  onClose?: () => void
  open: boolean
  size?: "default" | "wide"
}) {
  const { t } = useTranslation()
  const closeLabel = t("actions.close", { ns: "common" })
  const [shouldRender, setShouldRender] = useState(open)

  useEffect(() => {
    if (open) {
      setShouldRender(true)
      return
    }

    const timeoutId = window.setTimeout(() => {
      setShouldRender(false)
    }, PANEL_EXIT_DURATION_MS)

    return () => window.clearTimeout(timeoutId)
  }, [open])

  if (!shouldRender) return null
  const panelStyle = {
    "--agent-chat-composer-panel-max-height": `${maxHeight}px`,
    maxHeight,
  } as CSSProperties

  return (
    <div
      aria-hidden={!open}
      className={`absolute inset-x-0 bottom-[calc(100%+1.25rem)] z-50 origin-bottom overflow-hidden rounded-2xl border border-(--agent-chat-soft-border) bg-(--agent-chat-elevated-strong) bg-[image:var(--agent-chat-card-background)] shadow-[var(--agent-chat-card-shadow)] backdrop-blur-xl transition-[opacity,transform] duration-200 ease-out ${
        open
          ? "translate-y-0 scale-100 opacity-100"
          : "pointer-events-none translate-y-3 scale-[0.97] opacity-0"
      }`}
      data-composer-interactive="true"
      data-agent-chat-composer-panel-shell="true"
      data-agent-chat-composer-panel-size={size}
      style={panelStyle}
    >
      <div className="relative flex max-h-[var(--agent-chat-composer-panel-max-height)] min-h-0 flex-col">
        <div
          className="max-h-[calc(var(--agent-chat-composer-panel-max-height)-1.5rem)] min-h-0 flex-1 px-3 pt-3 pb-3"
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
