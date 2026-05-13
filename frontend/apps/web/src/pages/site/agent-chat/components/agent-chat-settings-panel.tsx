import {
  BadgeDollarSign,
  BarChart3,
  ChevronsUpDown,
  ClipboardList,
  Loader2,
  PenLine,
  Rocket,
  ShieldAlert,
  type LucideIcon,
} from "lucide-react"
import { useTranslation } from "react-i18next"
import type { AgentChatProductContextSelection } from "../domain/agent-chat-context-selection"
import type { AgentChatPlaybookSelection } from "../domain/agent-chat-playbook"
import type {
  AgentChatRuntimeSelection,
  AgentRuntimeOptions,
} from "../domain/agent-runtime-options"
import type { AgentChatStrategyId } from "../domain/agent-chat-strategy"
import { AgentChatProductContextBar } from "./agent-chat-product-context-bar"
import { AgentChatRuntimeOptionsBar } from "./agent-chat-runtime-options-bar"
import { AgentChatStrategyBar } from "./agent-chat-strategy-bar"

const PLAYBOOK_ICONS: Record<string, LucideIcon> = {
  briefing: ClipboardList,
  creative: PenLine,
  launch: Rocket,
  pricing: BadgeDollarSign,
  review: BarChart3,
  service: ShieldAlert,
}

export interface AgentChatSettingsPanelProps {
  availableProducts: AgentChatProductContextSelection[]
  isLoadingPlaybooks: boolean
  isLoadingProducts: boolean
  isLoadingRuntimeOptions: boolean
  isOpen: boolean
  isStreaming: boolean
  onCollaborationModeChange: (value: string) => void
  onLearningLevelChange: (value: string) => void
  onModelChange: (value: string) => void
  onPrimaryRoleChange: (value: string) => void
  onSelectPlaybook: (playbook: AgentChatPlaybookSelection) => void
  onStrategyChange: (strategyId: AgentChatStrategyId) => void
  onToggleOpen: () => void
  onToggleProductContext: (product: AgentChatProductContextSelection) => void
  onToggleRole: (value: string) => void
  playbooks: AgentChatPlaybookSelection[]
  runtimeOptions: AgentRuntimeOptions | null
  runtimeSelection: AgentChatRuntimeSelection
  selectedProducts: AgentChatProductContextSelection[]
  strategyId: AgentChatStrategyId
}

export function AgentChatSettingsPanel({
  availableProducts,
  isLoadingPlaybooks,
  isLoadingProducts,
  isLoadingRuntimeOptions,
  isOpen,
  isStreaming,
  onCollaborationModeChange,
  onLearningLevelChange,
  onModelChange,
  onPrimaryRoleChange,
  onSelectPlaybook,
  onStrategyChange,
  onToggleOpen,
  onToggleProductContext,
  onToggleRole,
  playbooks,
  runtimeOptions,
  runtimeSelection,
  selectedProducts,
  strategyId,
}: AgentChatSettingsPanelProps) {
  const { t } = useTranslation()
  const activeStrategyLabel = t(`agentChat.strategy.options.${strategyId}.label`)
  const selectedProductLabel =
    selectedProducts.length > 0
      ? selectedProducts.map((product) => product.name).join(" / ")
      : t("agentChat.composer.productContext.noneSelected")
  const runtimeLabel = [
    runtimeSelection.modelName,
    runtimeSelection.primaryRole,
    runtimeSelection.selectedRoles.length
      ? t("agentChat.composer.settings.rolesCount", {
          count: runtimeSelection.selectedRoles.length,
        })
      : null,
  ]
    .filter(Boolean)
    .join(" · ")

  return (
    <>
      <div className="flex min-w-0 items-center justify-between gap-2 rounded-xl bg-background px-3 py-2">
        <div className="flex min-w-0 flex-wrap items-center gap-1.5 text-xs text-muted-foreground">
          <span className="font-medium text-foreground">
            {activeStrategyLabel}
          </span>
          <span>·</span>
          <button
            type="button"
            disabled={isStreaming}
            onClick={() => {
              if (!isOpen) onToggleOpen()
            }}
            className="inline-flex max-w-56 min-w-0 items-center gap-1.5 rounded-full border border-border bg-card px-2.5 py-1 text-xs text-muted-foreground transition-colors hover:bg-muted hover:text-foreground disabled:cursor-not-allowed disabled:opacity-50"
            title={t("agentChat.composer.productContext.choose")}
          >
            <span
              className={`size-1.5 shrink-0 rounded-full ${
                selectedProducts.length ? "bg-emerald-500" : "bg-muted-foreground/50"
              }`}
            />
            <span className="truncate">{selectedProductLabel}</span>
          </button>
          {runtimeLabel ? (
            <>
              <span>·</span>
              <span className="hidden sm:inline">{runtimeLabel}</span>
            </>
          ) : null}
        </div>
        <button
          type="button"
          disabled={isStreaming}
          onClick={onToggleOpen}
          className="flex shrink-0 items-center gap-1.5 rounded-full border border-border bg-card px-3 py-1.5 text-xs text-muted-foreground transition-colors hover:bg-muted hover:text-foreground disabled:cursor-not-allowed disabled:opacity-50"
          aria-expanded={isOpen}
        >
          <ChevronsUpDown className="size-3.5" />
          {t("agentChat.composer.settings.toggle")}
        </button>
      </div>
      {isOpen ? (
        <div className="grid gap-2 rounded-xl border border-border bg-background/80 p-2">
          <AgentChatProductContextBar
            availableProducts={availableProducts}
            disabled={isStreaming}
            isLoadingProducts={isLoadingProducts}
            onToggleProductContext={onToggleProductContext}
            selectedProducts={selectedProducts}
          />
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
          />
          <PlaybookButtons
            isLoading={isLoadingPlaybooks}
            isStreaming={isStreaming}
            onSelectPlaybook={onSelectPlaybook}
            playbooks={playbooks}
          />
        </div>
      ) : null}
    </>
  )
}

function PlaybookButtons({
  isLoading,
  isStreaming,
  onSelectPlaybook,
  playbooks,
}: {
  isLoading: boolean
  isStreaming: boolean
  onSelectPlaybook: (playbook: AgentChatPlaybookSelection) => void
  playbooks: AgentChatPlaybookSelection[]
}) {
  const { t } = useTranslation()

  return (
    <div className="flex flex-wrap gap-2">
      {isLoading ? (
        <span className="flex h-7 items-center gap-2 rounded-full border border-border bg-background/70 px-3 text-xs text-muted-foreground">
          <Loader2 className="size-3.5 animate-spin" />
          {t("agentChat.composer.playbooks.loading")}
        </span>
      ) : null}
      {!isLoading && playbooks.length === 0 ? (
        <span className="flex h-7 items-center rounded-full border border-border bg-background/70 px-3 text-xs text-muted-foreground">
          {t("agentChat.composer.playbooks.empty")}
        </span>
      ) : null}
      {playbooks.map((playbook) => {
        const Icon = PLAYBOOK_ICONS[playbook.category] ?? ClipboardList
        return (
          <button
            key={playbook.id}
            type="button"
            disabled={isStreaming}
            onClick={() => onSelectPlaybook(playbook)}
            className="flex max-w-44 items-center gap-2 rounded-full border border-border bg-background px-3 py-1.5 text-xs text-muted-foreground transition-colors hover:bg-muted hover:text-foreground disabled:cursor-not-allowed disabled:opacity-50"
            title={`${playbook.description} ${playbook.expectedOutputs.join(" / ")}`}
          >
            <Icon className="size-3.5" />
            <span className="truncate">{playbook.title}</span>
          </button>
        )
      })}
    </div>
  )
}
