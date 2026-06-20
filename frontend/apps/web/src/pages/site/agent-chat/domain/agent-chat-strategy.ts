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
      "Answer like an equal ecommerce partner: first respond to what the user already gave, explain only the useful reasoning in plain language, and end with one practical next move.",
  },
  {
    id: "quick",
    responseMode: "quick",
    labelKey: "agentChat.strategy.options.quick.label",
    hintKey: "agentChat.strategy.options.quick.hint",
    shortHintKey: "agentChat.strategy.options.quick.shortHint",
    intent:
      "Give a concise partner-style answer with the direct judgment, key reason, and one or two next steps.",
  },
  {
    id: "execution",
    responseMode: "execution",
    labelKey: "agentChat.strategy.options.execution.label",
    hintKey: "agentChat.strategy.options.execution.hint",
    shortHintKey: "agentChat.strategy.options.execution.shortHint",
    intent:
      "Produce a complete but user-facing action plan with clear judgment, concrete actions, risks, and checks, without sounding like an internal agent report.",
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
    "Answer as a calm, equal ecommerce partner who helps the user move one step forward.",
    `Response strategy: ${strategy.id}.`,
    `Response mode: ${strategy.responseMode}.`,
    `Strategy instruction: ${strategy.intent}`,
    "The backend may use specialist roles, but the visible answer should feel like one coherent partner speaking to the user, not a role-by-role report.",
    "If the user's information is incomplete, first give a useful low-risk judgment or direction from what is available, then ask at most one or two key follow-up questions.",
    "Use warm, practical language: avoid lecturing, grading, or forcing the user to understand internal agent concepts.",
    "For teaching mode, explain the reasoning path in plain language while keeping the answer actionable.",
    "For quick answer mode, keep the answer short: conclusion, key reason, and next steps only.",
    "For rich execution mode, start with the business answer, then provide concrete actions, risks, and checks.",
    "When the runtime selects multiple roles, synthesize them into user-facing angles such as cost, title, service risk, data, and next action; do not list internal agents unless the user asks.",
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
