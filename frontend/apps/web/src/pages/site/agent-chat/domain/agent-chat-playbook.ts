import type { CommerceAgentPlaybookResponse } from "@/api"
import {
  DEFAULT_AGENT_CHAT_STRATEGY_ID,
  normalizeAgentChatStrategyId,
  type AgentChatStrategyId,
} from "./agent-chat-strategy"

export interface AgentChatPlaybookSelection {
  id: string
  title: string
  description: string
  category: string
  recommendedStrategy: AgentChatStrategyId
  prompt: string
  requiredContext: string[]
  expectedOutputs: string[]
  roles: string[]
}

export function mapCommercePlaybookToSelection(
  playbook: CommerceAgentPlaybookResponse
): AgentChatPlaybookSelection {
  return {
    id: playbook.id,
    title: playbook.title,
    description: playbook.description,
    category: playbook.category,
    recommendedStrategy:
      normalizeAgentChatStrategyId(playbook.recommendedStrategy) ??
      DEFAULT_AGENT_CHAT_STRATEGY_ID,
    prompt: playbook.prompt,
    requiredContext: Array.isArray(playbook.requiredContext)
      ? playbook.requiredContext
      : [],
    expectedOutputs: Array.isArray(playbook.expectedOutputs)
      ? playbook.expectedOutputs
      : [],
    roles: Array.isArray(playbook.roles) ? playbook.roles : [],
  }
}
