import { BrainCircuit, GraduationCap, Zap } from "lucide-react"
import { useTranslation } from "react-i18next"
import {
  AGENT_CHAT_STRATEGIES,
  type AgentChatStrategyId,
} from "../domain/agent-chat-strategy"

const STRATEGY_ICONS = {
  execution: BrainCircuit,
  learning: GraduationCap,
  quick: Zap,
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
    <div className="grid gap-3 rounded-lg border border-border bg-background p-3">
      <div className="text-xs font-semibold text-foreground">
        {t("agentChat.strategy.current")}
      </div>
      <div className="grid gap-2 sm:grid-cols-3">
        {AGENT_CHAT_STRATEGIES.map((strategy) => {
          const Icon = STRATEGY_ICONS[strategy.id]
          const active = strategy.id === strategyId

          return (
            <button
              key={strategy.id}
              type="button"
              disabled={disabled}
              onClick={() => onStrategyChange(strategy.id)}
              className={`flex min-w-0 items-start gap-2 rounded-lg border px-3 py-2.5 text-left transition-colors disabled:cursor-not-allowed disabled:opacity-50 ${
                active
                  ? "border-primary/50 bg-primary/10 text-primary shadow-sm"
                  : "border-border bg-card text-muted-foreground hover:bg-muted hover:text-foreground"
              }`}
              title={t(strategy.hintKey)}
            >
              <Icon className="mt-0.5 size-4 shrink-0" />
              <span className="min-w-0">
                <span className="block text-sm font-semibold">
                  {t(strategy.labelKey)}
                </span>
                <span className="mt-1 block text-xs/5 opacity-80">
                  {t(strategy.hintKey)}
                </span>
              </span>
            </button>
          )
        })}
      </div>
    </div>
  )
}
