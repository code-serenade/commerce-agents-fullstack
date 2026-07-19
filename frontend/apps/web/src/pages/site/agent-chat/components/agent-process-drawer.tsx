import { Button } from "@workspace/ui-components/stable/button"
import { ChevronDown, ChevronUp } from "lucide-react"
import { useState } from "react"
import { useTranslation } from "react-i18next"
import type { AgentRuntimeEvent } from "../agent-chat-data"
import { useAgentChatContext } from "../context/use-agent-chat-context"
import { getWorkbenchAgents } from "../view-model/agent-dispatch-workbench-agents-view-model"
import { AgentChatSwarmPanel } from "./agent-chat-swarm-panel"
import { AgentRoleClusterPanel } from "./agent-role-cluster-panel"

export function AgentProcessDrawer({
  isStreaming,
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
  const [isRoleClusterCollapsed, setIsRoleClusterCollapsed] = useState(false)
  const agents = getWorkbenchAgents(runtimeEvents, isStreaming, t)
  const roleClusterToggleLabel = isRoleClusterCollapsed
    ? t("agentChat.thread.dispatchPanel.expandRoleClusterCards")
    : t("agentChat.thread.dispatchPanel.collapseRoleClusterCards")
  const RoleClusterToggleIcon = isRoleClusterCollapsed ? ChevronDown : ChevronUp

  return (
    <div
      className="absolute inset-0 z-50 flex justify-end bg-background/28 backdrop-blur-[2px]"
      data-agent-process-drawer="true"
      onPointerDown={onClose}
      role="presentation"
    >
      <div
        aria-label={t("agentChat.workbench.title")}
        aria-modal="true"
        className="flex h-full w-[min(42rem,100%)] max-w-full min-w-0 flex-col items-end md:w-[min(42rem,88%)] xl:w-[min(44rem,48%)] 2xl:w-[min(48rem,46%)]"
        onPointerDown={(event) => event.stopPropagation()}
        role="dialog"
      >
        <div className="agent-process-drawer__panel pointer-events-auto size-full min-h-0 max-w-full min-w-0 overflow-hidden border-y-0 border-r-0 border-l border-(--agent-chat-soft-border) bg-(--agent-chat-elevated-surface) bg-(image:--agent-chat-card-background) text-[13px] leading-relaxed shadow-(--agent-chat-card-shadow) backdrop-blur-xl [&_.text-\[10px\]]:text-[11px] [&_.text-\[11px\]]:text-xs [&_.text-xs]:text-sm">
          <AgentChatSwarmPanel
            feedback={chat.feedback}
            headerAddon={
              <AgentRoleClusterPanel
                agents={agents}
                animatePortraits={isStreaming}
                headerAction={
                  <Button
                    aria-expanded={!isRoleClusterCollapsed}
                    aria-label={roleClusterToggleLabel}
                    className="h-7 gap-1.5 rounded-md border-(--agent-chat-soft-border) bg-(--agent-chat-elevated-strong)/70 px-2 text-xs text-muted-foreground hover:bg-muted hover:text-foreground"
                    onClick={() =>
                      setIsRoleClusterCollapsed((collapsed) => !collapsed)
                    }
                    size="xs"
                    title={roleClusterToggleLabel}
                    type="button"
                    variant="outline"
                  >
                    <RoleClusterToggleIcon className="size-3.5" />
                    {isRoleClusterCollapsed
                      ? t("agentChat.thread.dispatchPanel.expandRoleCluster")
                      : t("agentChat.thread.dispatchPanel.collapseRoleCluster")}
                  </Button>
                }
                itemDetail="summary"
                isCollapsed={isRoleClusterCollapsed}
                title={t("agentChat.thread.dispatchPanel.roleClusterTitle")}
                variant="compact"
              />
            }
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
