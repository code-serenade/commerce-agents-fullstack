import { FileUp, LockKeyhole, Send, Square } from "lucide-react"
import { useTranslation } from "react-i18next"
import type { AgentChatProductContextSelection } from "../domain/agent-chat-context-selection"
import type { AgentChatPlaybookSelection } from "../domain/agent-chat-playbook"
import type {
  AgentChatRuntimeSelection,
  AgentRuntimeOptions,
} from "../domain/agent-runtime-options"
import type { AgentChatStrategyId } from "../domain/agent-chat-strategy"
import { AgentChatProductContextBar } from "./agent-chat-product-context-bar"
import { AgentChatSettingsPanel } from "./agent-chat-settings-panel"

export function AgentChatComposerToolbar({
  availableProducts,
  canSend,
  isLoggedIn,
  isLoadingPlaybooks,
  isLoadingProducts,
  isLoadingRuntimeOptions,
  isStreaming,
  onAttach,
  onRuntimeCollaborationModeChange,
  onRuntimeLearningLevelChange,
  onRuntimeModelChange,
  onRuntimePrimaryRoleChange,
  onSelectPlaybook,
  onSend,
  onStop,
  onStrategyChange,
  onClearProductContext,
  onToggleProductContext,
  onToggleProductPanel,
  onToggleRuntimeRole,
  onToggleSettingsPanel,
  playbooks,
  productContextOpen,
  runtimeOptions,
  runtimeSelection,
  selectedProducts,
  settingsOpen,
  strategyId,
}: {
  availableProducts: AgentChatProductContextSelection[]
  canSend: boolean
  isLoggedIn: boolean
  isLoadingPlaybooks: boolean
  isLoadingProducts: boolean
  isLoadingRuntimeOptions: boolean
  isStreaming: boolean
  onAttach: () => void
  onRuntimeCollaborationModeChange: (value: string) => void
  onRuntimeLearningLevelChange: (value: string) => void
  onRuntimeModelChange: (value: string) => void
  onRuntimePrimaryRoleChange: (value: string) => void
  onSelectPlaybook: (playbook: AgentChatPlaybookSelection) => void
  onSend: () => void
  onStop: () => void
  onStrategyChange: (strategyId: AgentChatStrategyId) => void
  onClearProductContext: () => void
  onToggleProductContext: (product: AgentChatProductContextSelection) => void
  onToggleProductPanel: (open: boolean) => void
  onToggleRuntimeRole: (value: string) => void
  onToggleSettingsPanel: () => void
  playbooks: AgentChatPlaybookSelection[]
  productContextOpen: boolean
  runtimeOptions: AgentRuntimeOptions | null
  runtimeSelection: AgentChatRuntimeSelection
  selectedProducts: AgentChatProductContextSelection[]
  settingsOpen: boolean
  strategyId: AgentChatStrategyId
}) {
  const { t } = useTranslation()

  return (
    <div className="flex min-w-0 items-center justify-between gap-2">
      <div className="flex min-w-0 flex-wrap items-center gap-1.5">
        <button
          type="button"
          className="flex h-7 shrink-0 items-center gap-1.5 rounded-full border border-border bg-card px-2.5 text-xs font-medium text-muted-foreground transition-colors hover:bg-muted hover:text-foreground disabled:cursor-not-allowed disabled:opacity-50"
          disabled={isStreaming || !isLoggedIn}
          onClick={onAttach}
          title={t("agentChat.composer.attachTitle")}
        >
          <FileUp className="size-3.5" />
          <span>{t("agentChat.composer.attachShort")}</span>
        </button>
        <AgentChatProductContextBar
          availableProducts={availableProducts}
          disabled={isStreaming}
          isLoadingProducts={isLoadingProducts}
          isOpen={productContextOpen}
          onClearProductContext={onClearProductContext}
          onOpenChange={onToggleProductPanel}
          onToggleProductContext={onToggleProductContext}
          selectedProducts={selectedProducts}
          variant="compact"
        />
        <AgentChatSettingsPanel
          isLoadingPlaybooks={isLoadingPlaybooks}
          isLoadingRuntimeOptions={isLoadingRuntimeOptions}
          isOpen={settingsOpen}
          isStreaming={isStreaming}
          onCollaborationModeChange={onRuntimeCollaborationModeChange}
          onLearningLevelChange={onRuntimeLearningLevelChange}
          onModelChange={onRuntimeModelChange}
          onPrimaryRoleChange={onRuntimePrimaryRoleChange}
          onSelectPlaybook={onSelectPlaybook}
          onStrategyChange={onStrategyChange}
          onToggleOpen={onToggleSettingsPanel}
          onToggleRole={onToggleRuntimeRole}
          playbooks={playbooks}
          runtimeOptions={runtimeOptions}
          runtimeSelection={runtimeSelection}
          strategyId={strategyId}
          variant="compact"
        />
      </div>
      <div className="sm:hidden">
        <AgentChatComposerSendButton
          canSend={canSend}
          isLoggedIn={isLoggedIn}
          isStreaming={isStreaming}
          onSend={onSend}
          onStop={onStop}
        />
      </div>
    </div>
  )
}

export function AgentChatComposerSendButton({
  canSend,
  isLoggedIn,
  isStreaming,
  onSend,
  onStop,
}: {
  canSend: boolean
  isLoggedIn: boolean
  isStreaming: boolean
  onSend: () => void
  onStop: () => void
}) {
  const { t } = useTranslation()

  if (isStreaming) {
    return (
      <button
        type="button"
        onClick={onStop}
        className="flex size-8 shrink-0 items-center justify-center rounded-full border border-border bg-card text-muted-foreground transition-colors hover:bg-muted hover:text-foreground"
        title={t("agentChat.composer.stop")}
      >
        <Square className="size-4" />
      </button>
    )
  }

  return (
    <button
      type="button"
      disabled={!canSend}
      onClick={onSend}
      className="flex size-8 shrink-0 items-center justify-center rounded-full bg-foreground text-background transition-colors hover:bg-foreground/90 disabled:cursor-not-allowed disabled:opacity-45"
      title={t("agentChat.composer.send")}
    >
      {canSend ? (
        isLoggedIn ? (
          <Send className="size-4" />
        ) : (
          <LockKeyhole className="size-4" />
        )
      ) : (
        <Send className="size-4" />
      )}
    </button>
  )
}
