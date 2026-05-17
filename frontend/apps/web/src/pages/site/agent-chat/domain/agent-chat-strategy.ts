export type AgentChatResponseMode = "execution" | "learning" | "quick"

export interface AgentChatStrategy {
  id: AgentChatStrategyId
  responseMode: AgentChatResponseMode
  labelKey: string
  hintKey: string
  intent: string
}

export type AgentChatStrategyId = "execution" | "learning" | "quick"

export const AGENT_CHAT_STRATEGIES = [
  {
    id: "learning",
    responseMode: "learning",
    labelKey: "agentChat.strategy.options.learning.label",
    hintKey: "agentChat.strategy.options.learning.hint",
    intent:
      "Explain the reasoning in simple steps while still producing an actionable commerce answer.",
  },
  {
    id: "quick",
    responseMode: "quick",
    labelKey: "agentChat.strategy.options.quick.label",
    hintKey: "agentChat.strategy.options.quick.hint",
    intent:
      "Give a concise answer with the direct conclusion, key reason, and a short next-step list.",
  },
  {
    id: "execution",
    responseMode: "execution",
    labelKey: "agentChat.strategy.options.execution.label",
    hintKey: "agentChat.strategy.options.execution.hint",
    intent:
      "Produce a complete execution plan with role division, tables, action owners, risks, and acceptance checks.",
  },
] as const satisfies readonly AgentChatStrategy[]

export const DEFAULT_AGENT_CHAT_STRATEGY_ID: AgentChatStrategyId = "learning"

export function isAgentChatStrategyId(value: string): value is AgentChatStrategyId {
  return AGENT_CHAT_STRATEGIES.some((strategy) => strategy.id === value)
}

export function normalizeAgentChatStrategyId(
  value: string | null | undefined
): AgentChatStrategyId | null {
  if (!value) return null
  if (value === "collab") return "execution"
  return isAgentChatStrategyId(value) ? value : null
}

export function getAgentChatStrategy(id: AgentChatStrategyId): AgentChatStrategy {
  return (
    AGENT_CHAT_STRATEGIES.find((strategy) => strategy.id === id) ??
    AGENT_CHAT_STRATEGIES[0]
  )
}

export function buildAgentStrategyInput(
  content: string,
  strategy: AgentChatStrategy
) {
  if (isQuickChatInput(content)) return content

  return [
    "Answer as an ecommerce multi-agent virtual employee workspace.",
    `Response strategy: ${strategy.id}.`,
    `Response mode: ${strategy.responseMode}.`,
    `Strategy instruction: ${strategy.intent}`,
    "Relevant virtual employee roles include product, data, support, content, ads, operations, finance, supply chain, and delivery.",
    "For teaching mode, explain the reasoning path in student-friendly language while keeping the answer actionable.",
    "For quick answer mode, keep the answer short: conclusion, key reason, and next steps only.",
    "For rich execution mode, start with the business answer, then provide role division, tables, concrete actions, risks, and acceptance checks.",
    "When the runtime selects multiple roles, present role-based findings and then synthesize one final recommendation.",
    "Use markdown tables for comparisons, funnels, role division, or action plans when they make the answer easier to scan.",
    "Keep the answer centered on the user's question. Do not claim external platform execution or dynamic tool calls unless runtime events prove them.",
    "User question:",
    content,
  ].join("\n")
}

export function isQuickChatInput(content: string) {
  return /^(hi|hello|hey|你好|您好|嗨|哈喽|在吗|早上好|中午好|下午好|晚上好)[.!。！?？\s]*$/i.test(
    content.trim()
  )
}
