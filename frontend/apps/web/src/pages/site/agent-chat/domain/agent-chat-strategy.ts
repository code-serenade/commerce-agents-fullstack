export type AgentChatResponseMode = "execution" | "learning" | "quick"

export interface AgentChatStrategy {
  id: AgentChatStrategyId
  responseMode: AgentChatResponseMode
  labelKey: string
  hintKey: string
  shortHintKey: string
  intent: string
}

export type AgentChatStrategyId = "execution" | "learning" | "quick"

export const AGENT_CHAT_STRATEGIES = [
  {
    id: "learning",
    responseMode: "learning",
    labelKey: "agentChat.strategy.options.learning.label",
    hintKey: "agentChat.strategy.options.learning.hint",
    shortHintKey: "agentChat.strategy.options.learning.shortHint",
    intent:
      "Explain the reasoning in simple student-friendly steps and produce an actionable ecommerce training answer.",
  },
  {
    id: "quick",
    responseMode: "quick",
    labelKey: "agentChat.strategy.options.quick.label",
    hintKey: "agentChat.strategy.options.quick.hint",
    shortHintKey: "agentChat.strategy.options.quick.shortHint",
    intent:
      "Give a concise answer with the direct conclusion, key reason, and a short next-step list.",
  },
  {
    id: "execution",
    responseMode: "execution",
    labelKey: "agentChat.strategy.options.execution.label",
    hintKey: "agentChat.strategy.options.execution.hint",
    shortHintKey: "agentChat.strategy.options.execution.shortHint",
    intent:
      "Produce a complete training task plan with specialist agent findings, tables, next actions, risks, and submission checks.",
  },
] as const satisfies readonly AgentChatStrategy[]

export const DEFAULT_AGENT_CHAT_STRATEGY_ID: AgentChatStrategyId = "learning"

export function isAgentChatStrategyId(
  value: string
): value is AgentChatStrategyId {
  return AGENT_CHAT_STRATEGIES.some((strategy) => strategy.id === value)
}

export function normalizeAgentChatStrategyId(
  value: string | null | undefined
): AgentChatStrategyId | null {
  if (!value) return null
  if (value === "collab") return "execution"
  return isAgentChatStrategyId(value) ? value : null
}

export function getAgentChatStrategy(
  id: AgentChatStrategyId
): AgentChatStrategy {
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
    "Answer as an ecommerce AI training assistant for students.",
    `Response strategy: ${strategy.id}.`,
    `Response mode: ${strategy.responseMode}.`,
    `Strategy instruction: ${strategy.intent}`,
    "Relevant specialist agents include product selection, data diagnosis, customer understanding, content, ads, operations, budget, supply chain, deliverables, and final review report.",
    "For teaching mode, explain the reasoning path in plain student-friendly language while keeping the answer actionable.",
    "For quick answer mode, keep the answer short: conclusion, key reason, and next steps only.",
    "For rich execution mode, start with the business answer, then provide multi-agent role findings, tables, concrete actions, risks, and submission checks.",
    "When the runtime selects multiple roles, explicitly show which specialist agents contributed before synthesizing one final recommendation.",
    "Use markdown tables for comparisons, funnels, agent role findings, or action plans when they make the answer easier to scan.",
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
