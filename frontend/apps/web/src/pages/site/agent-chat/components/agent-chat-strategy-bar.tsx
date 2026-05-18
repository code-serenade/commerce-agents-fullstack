import { useTranslation } from "react-i18next"
import {
  AGENT_CHAT_STRATEGIES,
  type AgentChatStrategyId,
} from "../domain/agent-chat-strategy"
import { OptionSegment } from "./agent-chat-runtime-shared"

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
    <OptionSegment
      disabled={disabled}
      label={t("agentChat.strategy.current")}
      onValueChange={(value) => onStrategyChange(value as AgentChatStrategyId)}
      options={AGENT_CHAT_STRATEGIES.map((strategy) => ({
        description: t(strategy.hintKey),
        label: t(strategy.labelKey),
        value: strategy.id,
      }))}
      value={strategyId}
    />
  )
}
