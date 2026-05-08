import { BrainCircuit, GraduationCap, Workflow } from "lucide-react"
import { useTranslation } from "react-i18next"
import {
  AGENT_CHAT_STRATEGIES,
  type AgentChatStrategyId,
} from "../domain/agent-chat-strategy"

const STRATEGY_ICONS = {
  collab: Workflow,
  execution: BrainCircuit,
  learning: GraduationCap,
} as const

export interface AgentChatStrategyBarProps {
  disabled: boolean
  onStrategyChange: (strategyId: AgentChatStrategyId) => void
  strategyId: AgentChatStrategyId
}

export function AgentChatStrategyBar({
  disabled,
  onStrategyChange,
  strategyId,
}: AgentChatStrategyBarProps) {
  const { t } = useTranslation()

  return (
    <div className="grid gap-1.5 rounded-2xl bg-background p-1 sm:grid-cols-3">
      {AGENT_CHAT_STRATEGIES.map((strategy) => {
        const Icon = STRATEGY_ICONS[strategy.id]
        const active = strategy.id === strategyId

        return (
          <button
            key={strategy.id}
            type="button"
            disabled={disabled}
            onClick={() => onStrategyChange(strategy.id)}
            className={`flex min-w-0 items-center gap-2 rounded-xl border px-3 py-2 text-left transition-colors disabled:cursor-not-allowed disabled:opacity-50 ${
              active
                ? "border-blue-300 bg-blue-50 text-blue-700 shadow-sm dark:border-blue-700/70 dark:bg-blue-950/40 dark:text-blue-100"
                : "border-transparent bg-transparent text-muted-foreground hover:bg-muted hover:text-foreground"
            }`}
            title={t(strategy.hintKey)}
          >
            <Icon className="size-4 shrink-0" />
            <span className="min-w-0">
              <span className="block truncate text-xs font-medium">
                {t(strategy.labelKey)}
              </span>
              <span className="block truncate text-[11px]/4 opacity-80">
                {t(strategy.hintKey)}
              </span>
            </span>
          </button>
        )
      })}
    </div>
  )
}
