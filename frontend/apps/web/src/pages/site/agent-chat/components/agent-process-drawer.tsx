import { ListTree, X } from "lucide-react"
import { useTranslation } from "react-i18next"
import type { AgentRuntimeEvent } from "../agent-chat-data"
import { useAgentChatContext } from "../context/use-agent-chat-context"
import { AgentChatSwarmPanel } from "./agent-chat-swarm-panel"

export function AgentProcessDrawer({
  isStreaming,
  onClose,
  runtimeEvents,
  task,
}: {
  isStreaming: boolean
  onClose: () => void
  runtimeEvents: AgentRuntimeEvent[]
  task?: string
}) {
  const chat = useAgentChatContext()
  const { t } = useTranslation()
  const closeLabel = t("agentChat.workbench.close")

  return (
    <div className="pointer-events-none fixed inset-y-3 right-3 z-40 flex w-[min(48rem,calc(100%-1.5rem))] max-w-full flex-col items-end">
      <div className="pointer-events-auto mb-2 flex w-full items-center justify-between rounded-md border border-border bg-background/95 px-3 py-2 shadow-sm backdrop-blur">
        <div className="flex min-w-0 items-center gap-2 text-sm font-medium text-foreground">
          <ListTree className="size-4 shrink-0 text-muted-foreground" />
          <div className="min-w-0">
            <div className="truncate">
              {t("agentChat.workbench.title")}
              <span className="ml-1 font-normal text-muted-foreground">
                - {t("agentChat.thread.currentAnswerProcess")}
              </span>
            </div>
            {task ? (
              <div className="truncate text-xs font-normal text-muted-foreground">
                {task}
              </div>
            ) : null}
          </div>
        </div>
        <button
          type="button"
          onClick={onClose}
          className="flex h-8 shrink-0 items-center gap-1.5 rounded-md border border-border bg-background px-2.5 text-sm text-muted-foreground transition-colors hover:bg-muted hover:text-foreground"
          aria-label={closeLabel}
          title={closeLabel}
        >
          <X className="size-4" />
          {closeLabel}
        </button>
      </div>
      <div className="pointer-events-auto min-h-0 flex-1 overflow-hidden rounded-lg shadow-2xl ring-1 ring-border/80">
        <AgentChatSwarmPanel
          feedback={chat.feedback}
          isStreaming={isStreaming}
          messages={chat.messages}
          onCollapse={onClose}
          runtimeEvents={runtimeEvents}
          task={task}
        />
      </div>
    </div>
  )
}
