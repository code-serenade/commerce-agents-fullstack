import type { AgentChatSourceContextSelection } from "./agent-chat-context-selection"
import type { AgentChatSourceReadiness } from "../types"

export type SourceReadinessKey = AgentChatSourceReadiness

export type SourceReadinessSummary = Record<SourceReadinessKey, number>

export const sourceReadinessKeys: SourceReadinessKey[] = [
  "readable",
  "pending",
  "failed",
  "unsupported",
  "metadataOnly",
]

export function summarizeSelectedSourceReadiness(
  sources: AgentChatSourceContextSelection[]
): SourceReadinessSummary {
  return sources.reduce<SourceReadinessSummary>(
    (summary, source) => {
      const key = sourceReadinessKey(source)
      summary[key] += 1
      return summary
    },
    {
      failed: 0,
      metadataOnly: 0,
      pending: 0,
      readable: 0,
      unsupported: 0,
    }
  )
}

export function sourceReadinessKey(
  source: AgentChatSourceContextSelection
): SourceReadinessKey {
  if (source.visionReady || source.readable) return "readable"
  if (source.status === "pending") return "pending"
  if (source.status === "failed") return "failed"
  if (source.status === "unsupported") return "unsupported"
  return "metadataOnly"
}
