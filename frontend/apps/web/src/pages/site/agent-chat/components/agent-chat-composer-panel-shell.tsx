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
      className="absolute inset-x-0 bottom-[calc(100%+1.25rem)] z-50 overflow-x-hidden overflow-y-auto overscroll-contain rounded-2xl border border-(--agent-chat-soft-border) bg-(--agent-chat-elevated-strong) shadow-[var(--agent-chat-card-shadow)] backdrop-blur-xl [scrollbar-gutter:stable]"
      data-composer-interactive="true"
      data-agent-chat-composer-panel-scroll="true"
      data-agent-chat-composer-panel-shell="true"
      data-agent-chat-composer-panel-size={size}
      style={{ maxHeight }}
    >
      <div className="relative flex min-h-0 flex-col" style={{ maxHeight }}>
        {onClose ? (
          <button
            type="button"
            aria-label={closeLabel}
            title={closeLabel}
            onClick={onClose}
            className="absolute top-3 right-3 z-10 inline-flex size-8 items-center justify-center rounded-xl border border-border bg-(--agent-chat-elevated-strong) text-muted-foreground shadow-sm transition-colors hover:bg-muted hover:text-foreground focus-visible:ring-2 focus-visible:ring-ring focus-visible:outline-hidden"
          >
            <X className="size-4" aria-hidden="true" />
          </button>
        ) : null}
        <div
          className="min-h-0 flex-1 p-2.5 pt-3 pr-12 pb-16 sm:p-3 sm:pr-12 sm:pb-14"
          data-agent-chat-composer-panel-content="true"
        >
          {children}
        </div>
      </div>
    </div>
  )
}
