import type { ReactNode } from "react"

export function AgentChatComposerPanelShell({
  children,
  maxHeight,
  size = "default",
}: {
  children: ReactNode
  maxHeight: number
  size?: "default" | "wide"
}) {
  return (
    <div
      className="absolute inset-x-0 bottom-[calc(100%+1.25rem)] z-50 overflow-hidden rounded-lg border border-border bg-background shadow-2xl shadow-black/15"
      data-composer-interactive="true"
      data-agent-chat-composer-panel-shell="true"
      data-agent-chat-composer-panel-size={size}
      style={{ maxHeight }}
    >
      <div
        className="overflow-y-auto overscroll-contain p-2.5 [scrollbar-gutter:stable] sm:p-3"
        data-agent-chat-composer-panel-scroll="true"
        style={{ maxHeight }}
      >
        {children}
      </div>
    </div>
  )
}
