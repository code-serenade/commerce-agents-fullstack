import type { ReactNode } from "react"
import { X } from "lucide-react"
import { useTranslation } from "react-i18next"

export function AgentChatComposerPanelShell({
  children,
  maxHeight,
  onClose,
  size = "default",
}: {
  children: ReactNode
  maxHeight: number
  onClose?: () => void
  size?: "default" | "wide"
}) {
  const { t } = useTranslation()
  const closeLabel = t("actions.close", { ns: "common" })

  return (
    <div
      className="absolute inset-x-0 bottom-[calc(100%+1.25rem)] z-50 overflow-hidden rounded-lg border border-border bg-background shadow-2xl shadow-black/15"
      data-composer-interactive="true"
      data-agent-chat-composer-panel-shell="true"
      data-agent-chat-composer-panel-size={size}
      style={{ maxHeight }}
    >
      <div className="relative min-h-0" style={{ maxHeight }}>
        {onClose ? (
          <button
            type="button"
            aria-label={closeLabel}
            title={closeLabel}
            onClick={onClose}
            className="absolute right-3 top-3 z-10 inline-flex size-8 items-center justify-center rounded-md border border-border bg-background/95 text-muted-foreground shadow-sm transition-colors hover:bg-muted hover:text-foreground focus-visible:ring-2 focus-visible:ring-ring focus-visible:outline-hidden"
          >
            <X className="size-4" aria-hidden="true" />
          </button>
        ) : null}
        <div
          className="min-h-0 overflow-y-auto overscroll-contain p-2.5 pt-3 pr-12 [scrollbar-gutter:stable] sm:p-3 sm:pr-12"
          data-agent-chat-composer-panel-scroll="true"
        >
          {children}
        </div>
      </div>
    </div>
  )
}
