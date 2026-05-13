import type { AgentChatProductContextSelection } from "../domain/agent-chat-context-selection"
import type { AgentChatPlaybookSelection } from "../domain/agent-chat-playbook"
import type {
  AgentChatRuntimeSelection,
  AgentRuntimeOptions,
} from "../domain/agent-runtime-options"
import type { AgentChatStrategyId } from "../domain/agent-chat-strategy"
import { AgentChatProductContextBar } from "./agent-chat-product-context-bar"
import { AgentChatSettingsPanel } from "./agent-chat-settings-panel"

export function AgentChatComposerPanels({
  availableProducts,
  isLoadingPlaybooks,
  isLoadingProducts,
  isLoadingRuntimeOptions,
  isStreaming,
  onFocusComposer,
  onRuntimeCollaborationModeChange,
  onRuntimeLearningLevelChange,
  onRuntimeModelChange,
  onRuntimePrimaryRoleChange,
  onSelectPlaybook,
  onStrategyChange,
  onClearProductContext,
  onToggleProductContext,
  onToggleRuntimeRole,
  playbooks,
  productContextOpen,
  runtimeOptions,
  runtimeSelection,
  selectedProducts,
  settingsOpen,
  strategyId,
}: {
  availableProducts: AgentChatProductContextSelection[]
  isLoadingPlaybooks: boolean
  isLoadingProducts: boolean
  isLoadingRuntimeOptions: boolean
  isStreaming: boolean
  onFocusComposer: () => void
  onRuntimeCollaborationModeChange: (value: string) => void
  onRuntimeLearningLevelChange: (value: string) => void
  onRuntimeModelChange: (value: string) => void
  onRuntimePrimaryRoleChange: (value: string) => void
  onSelectPlaybook: (playbook: AgentChatPlaybookSelection) => void
  onStrategyChange: (strategyId: AgentChatStrategyId) => void
  onClearProductContext: () => void
  onToggleProductContext: (product: AgentChatProductContextSelection) => void
  onToggleRuntimeRole: (value: string) => void
  playbooks: AgentChatPlaybookSelection[]
  productContextOpen: boolean
  runtimeOptions: AgentRuntimeOptions | null
  runtimeSelection: AgentChatRuntimeSelection
  selectedProducts: AgentChatProductContextSelection[]
  settingsOpen: boolean
  strategyId: AgentChatStrategyId
}) {
  return (
    <>
      {productContextOpen ? (
        <div
          className="absolute inset-x-0 bottom-full z-50 mb-2 rounded-xl border border-border bg-background p-3 shadow-2xl shadow-black/15"
          data-composer-interactive="true"
        >
          <AgentChatProductContextBar
            availableProducts={availableProducts}
            disabled={isStreaming}
            isLoadingProducts={isLoadingProducts}
            onClearProductContext={onClearProductContext}
            onToggleProductContext={onToggleProductContext}
            selectedProducts={selectedProducts}
            variant="panel"
          />
        </div>
      ) : null}
      {settingsOpen ? (
        <div
          className="absolute inset-x-0 bottom-full z-50 mb-2 grid max-h-[min(32rem,70vh)] gap-2 overflow-y-auto rounded-xl border border-border bg-background p-3 shadow-2xl shadow-black/15"
          data-composer-interactive="true"
        >
          <AgentChatSettingsPanel
            isLoadingPlaybooks={isLoadingPlaybooks}
            isLoadingRuntimeOptions={isLoadingRuntimeOptions}
            isOpen
            isStreaming={isStreaming}
            onCollaborationModeChange={onRuntimeCollaborationModeChange}
            onLearningLevelChange={onRuntimeLearningLevelChange}
            onModelChange={onRuntimeModelChange}
            onPrimaryRoleChange={onRuntimePrimaryRoleChange}
            onSelectPlaybook={(playbook) => {
              onSelectPlaybook(playbook)
              queueMicrotask(onFocusComposer)
            }}
            onStrategyChange={onStrategyChange}
            onToggleOpen={() => undefined}
            onToggleRole={onToggleRuntimeRole}
            playbooks={playbooks}
            runtimeOptions={runtimeOptions}
            runtimeSelection={runtimeSelection}
            strategyId={strategyId}
            variant="panel"
          />
        </div>
      ) : null}
    </>
  )
}
