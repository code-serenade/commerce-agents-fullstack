import { ListTree, PanelRightClose, X } from "lucide-react"
import { useTranslation } from "react-i18next"
import type { AgentRuntimeEvent } from "../agent-chat-data"
import { useAgentChatContext } from "../context/use-agent-chat-context"
import { AgentChatSwarmPanel } from "./agent-chat-swarm-panel"

export function AgentProcessDrawer({
  anchorLabel,
  isStreaming,
  messageId,
  onClose,
  runtimeEvents,
  task,
}: {
  anchorLabel?: string
  isStreaming: boolean
  messageId?: string
  onClose: () => void
  runtimeEvents: AgentRuntimeEvent[]
  task?: string
}) {
  const chat = useAgentChatContext()
  const { t } = useTranslation()
  const closeLabel = t("agentChat.workbench.close")
  const answerLabel = messageId
    ? t("agentChat.thread.processLinkedToAnswer")
    : t("agentChat.thread.currentAnswerProcess")

  return (
    <div
      className="fixed inset-0 z-40 flex justify-end bg-background/35 p-3 backdrop-blur-[1px]"
      onPointerDown={onClose}
    >
      <div
        className="flex h-full w-[min(46rem,calc(100%-1rem))] max-w-full flex-col items-end xl:w-[min(48rem,42vw)] 2xl:w-[min(52rem,38vw)]"
        onPointerDown={(event) => event.stopPropagation()}
      >
        <div className="pointer-events-auto mb-3 flex w-full items-center justify-between rounded-lg border border-border bg-background/95 px-4 py-3 shadow-lg backdrop-blur">
          <div className="flex min-w-0 items-center gap-2 text-sm font-medium text-foreground">
            <ListTree className="size-4 shrink-0 text-primary" />
            <div className="min-w-0">
              <div className="truncate">
                {t("agentChat.workbench.title")}
                {anchorLabel ? (
                  <span className="ml-2 rounded-full bg-primary px-2 py-0.5 text-xs font-medium text-primary-foreground">
                    {anchorLabel}
                  </span>
                ) : null}
                <span className="ml-1 rounded-full bg-secondary px-2 py-0.5 text-xs font-medium text-secondary-foreground">
                  {answerLabel}
                </span>
              </div>
              {task ? (
                <div className="truncate text-xs font-normal text-muted-foreground">
                  {task}
                </div>
              ) : null}
            </div>
          </div>
          <div className="flex shrink-0 items-center gap-2">
            <button
              type="button"
              onClick={onClose}
              className="inline-flex h-9 shrink-0 items-center gap-1.5 rounded-md border border-primary/40 bg-primary/10 px-3 text-sm font-medium text-primary shadow-sm transition-colors hover:border-primary/70 hover:bg-primary/15"
              aria-label={t("agentChat.workbench.closePanel")}
              title={t("agentChat.workbench.closePanel")}
            >
              <PanelRightClose className="size-4" />
              {t("agentChat.workbench.closePanel")}
            </button>
            <button
              type="button"
              onClick={onClose}
              className="flex size-9 shrink-0 items-center justify-center rounded-md border border-border bg-background text-muted-foreground transition-colors hover:bg-muted hover:text-foreground"
              aria-label={closeLabel}
              title={closeLabel}
            >
              <X className="size-4" />
            </button>
          </div>
        </div>
        <div className="pointer-events-auto min-h-0 flex-1 overflow-hidden rounded-lg border border-border bg-background shadow-xl">
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
    </div>
  )
}
