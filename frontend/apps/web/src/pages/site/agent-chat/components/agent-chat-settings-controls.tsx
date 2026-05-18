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
  onModelChange,
  onPrimaryRoleChange,
  onStrategyChange,
  onToggleRole,
  runtimeOptions,
  runtimeSelection,
  strategyId,
}: {
  isLoadingRuntimeOptions: boolean
  isStreaming: boolean
  onCollaborationModeChange: (value: string) => void
  onLearningLevelChange: (value: string) => void
  onModelChange: (value: string) => void
  onPrimaryRoleChange: (value: string) => void
  onStrategyChange: (strategyId: AgentChatStrategyId) => void
  onToggleRole: (value: string) => void
  runtimeOptions: AgentRuntimeOptions | null
  runtimeSelection: AgentChatRuntimeSelection
  strategyId: AgentChatStrategyId
}) {
  return (
    <div className="grid gap-3 rounded-xl border border-border bg-background p-3">
      <AgentChatStrategyBar
        disabled={isStreaming}
        onStrategyChange={onStrategyChange}
        strategyId={strategyId}
      />
      <AgentChatRuntimeOptionsBar
        disabled={isStreaming}
        isLoadingRuntimeOptions={isLoadingRuntimeOptions}
        onCollaborationModeChange={onCollaborationModeChange}
        onLearningLevelChange={onLearningLevelChange}
        onModelChange={onModelChange}
        onPrimaryRoleChange={onPrimaryRoleChange}
        onToggleRole={onToggleRole}
        runtimeOptions={runtimeOptions}
        runtimeSelection={runtimeSelection}
        strategyId={strategyId}
      />
    </div>
  )
}
