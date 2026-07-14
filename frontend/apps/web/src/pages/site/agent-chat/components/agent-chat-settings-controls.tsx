import type {
  AgentChatRuntimeSelection,
  AgentRuntimeOptions,
} from "../domain/agent-runtime-options"
import type { AgentChatStrategyId } from "../domain/agent-chat-strategy"
import { AgentChatRuntimeOptionsBar } from "./agent-chat-runtime-options-bar"
import { AgentChatStrategyBar } from "./agent-chat-strategy-bar"

export function AgentChatSettingsControls({
  isLoadingRuntimeOptions,
  isStreaming,
  onCollaborationModeChange,
  onLearningLevelChange,
  onPrimaryRoleChange,
  onRoutingModeChange,
  onStrategyChange,
  onToggleRole,
  runtimeOptions,
  runtimeSelection,
  size = "default",
  strategyId,
}: {
  isLoadingRuntimeOptions: boolean
  isStreaming: boolean
  onCollaborationModeChange: (value: string) => void
  onLearningLevelChange: (value: string) => void
  onPrimaryRoleChange: (value: string) => void
  onRoutingModeChange: (value: string) => void
  onStrategyChange: (strategyId: AgentChatStrategyId) => void
  onToggleRole: (value: string) => void
  runtimeOptions: AgentRuntimeOptions | null
  runtimeSelection: AgentChatRuntimeSelection
  size?: "default" | "compact"
  strategyId: AgentChatStrategyId
}) {
  return (
    <div className="grid gap-2 rounded-xl border border-(--agent-chat-soft-border) bg-(--agent-chat-subtle-surface) p-2.5">
      <AgentChatStrategyBar
        disabled={isStreaming}
        onStrategyChange={onStrategyChange}
        size={size}
        strategyId={strategyId}
      />
      <AgentChatRuntimeOptionsBar
        disabled={isStreaming}
        isLoadingRuntimeOptions={isLoadingRuntimeOptions}
        onCollaborationModeChange={onCollaborationModeChange}
        onLearningLevelChange={onLearningLevelChange}
        onPrimaryRoleChange={onPrimaryRoleChange}
        onRoutingModeChange={onRoutingModeChange}
        onToggleRole={onToggleRole}
        runtimeOptions={runtimeOptions}
        runtimeSelection={runtimeSelection}
        size={size}
        strategyId={strategyId}
      />
    </div>
  )
}
