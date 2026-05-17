export type AgentChatResponseMode = "execution" | "learning"
export type AgentChatCollaborationMode = "single" | "manual"

export interface AgentChatStrategy {
  id: "execution" | "learning" | "collab"
  responseMode: AgentChatResponseMode
  collaborationMode: AgentChatCollaborationMode
  labelKey: string
  hintKey: string
  intent: string
}

export const AGENT_CHAT_STRATEGIES = [
  {
    id: "execution",
    responseMode: "execution",
    collaborationMode: "single",
    labelKey: "agentChat.strategy.options.execution.label",
    hintKey: "agentChat.strategy.options.execution.hint",
    intent: "Give a direct, usable answer first, then list the next actions.",
  },
  {
    id: "learning",
    responseMode: "learning",
    collaborationMode: "single",
    labelKey: "agentChat.strategy.options.learning.label",
    hintKey: "agentChat.strategy.options.learning.hint",
    intent:
      "Explain the reasoning in simple steps while still producing an actionable commerce answer.",
  },
  {
    id: "collab",
    responseMode: "execution",
    collaborationMode: "manual",
    labelKey: "agentChat.strategy.options.collab.label",
    hintKey: "agentChat.strategy.options.collab.hint",
    intent:
      "Use a commerce agent cluster style: plan first, split the task by data, budget, ads, creative, operations, and delivery, then synthesize the final answer.",
  },
] as const satisfies readonly AgentChatStrategy[]

export type AgentChatStrategyId = (typeof AGENT_CHAT_STRATEGIES)[number]["id"]

export const DEFAULT_AGENT_CHAT_STRATEGY_ID: AgentChatStrategyId = "collab"

export function isAgentChatStrategyId(value: string): value is AgentChatStrategyId {
  return AGENT_CHAT_STRATEGIES.some((strategy) => strategy.id === value)
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
    `Response mode: ${strategy.responseMode}. Collaboration mode: ${strategy.collaborationMode}.`,
    `Strategy instruction: ${strategy.intent}`,
    "Relevant virtual employee roles include product, data, support, content, ads, operations, finance, supply chain, and delivery.",
    "For execution mode, start with the business answer and concrete next actions.",
    "For learning mode, explain the reasoning path while keeping the answer actionable.",
    "For agent cluster mode, present role-based findings and then synthesize one final recommendation.",
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
