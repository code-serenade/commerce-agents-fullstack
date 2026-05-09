import { SearchCheck, TriangleAlert } from "lucide-react"
import type { TFunction } from "i18next"
import type { AgentRuntimeEvent } from "@/api"
import type { ComputerCard } from "./swarm-panel-view-model"

export function getContextCard(
  runtimeEvents: AgentRuntimeEvent[],
  t: TFunction
): ComputerCard | null {
  const ragEvent = [...runtimeEvents]
    .reverse()
    .find((item): item is Extract<AgentRuntimeEvent, { type: "rag_context_loaded" }> =>
      item.type === "rag_context_loaded"
    )
  if (ragEvent) {
    const sourceCount = ragEvent.sourceCount ?? ragEvent.source_count ?? 0
    const knowledgeCount = ragEvent.knowledgeCount ?? ragEvent.knowledge_count ?? 0
    const reason = ragEvent.degradedReason ?? ragEvent.degraded_reason
    const sourceLines = (ragEvent.sources ?? [])
      .slice(0, 2)
      .map((source) =>
        t("agentChat.swarm.cards.ragContext.sourceLine", {
          title: source.title,
          type: source.sourceType ?? source.source_type ?? "source",
        })
      )
    const queryTerms = ragEvent.queryTerms ?? ragEvent.query_terms ?? []
    return {
      id: "rag-context-loaded",
      title: t("agentChat.swarm.cards.ragContext.title"),
      meta: t("agentChat.swarm.cards.ragContext.meta", {
        knowledge: knowledgeCount,
        sources: sourceCount,
      }),
      description: ragEvent.degraded
        ? t("agentChat.swarm.cards.ragContext.degradedDescription")
        : t("agentChat.swarm.cards.ragContext.description"),
      icon: ragEvent.degraded ? TriangleAlert : SearchCheck,
      lines: [
        ...(queryTerms.length
          ? [
              t("agentChat.swarm.cards.ragContext.queryLine", {
                terms: queryTerms.slice(0, 5).join(", "),
              }),
            ]
          : []),
        ...sourceLines,
        ...(reason
          ? [
              t("agentChat.swarm.cards.ragContext.fallbackLine", {
                reason,
              }),
            ]
          : []),
        ...(ragEvent.summary ? [ragEvent.summary] : []),
      ].slice(0, 4),
      state: ragEvent.degraded ? "reworking" : "done",
    }
  }

  const event = [...runtimeEvents]
    .reverse()
    .find((item): item is Extract<AgentRuntimeEvent, { type: "context_loaded" }> =>
      item.type === "context_loaded"
    )
  if (!event) return null

  return {
    id: "context-loaded",
    title: t("agentChat.swarm.cards.contextLoaded.title"),
    meta: t("agentChat.swarm.cards.contextLoaded.meta", {
      knowledge: event.knowledge_count ?? 0,
      products: event.product_count ?? event.productIds?.length ?? 0,
    }),
    description: t("agentChat.swarm.cards.contextLoaded.description"),
    icon: SearchCheck,
    lines: [
      ...(event.productIds ?? event.product_ids ?? []).slice(0, 3).map((id) =>
        t("agentChat.swarm.cards.contextLoaded.productLine", { id })
      ),
      ...(event.summary ? [event.summary] : []),
    ].slice(0, 4),
    state: "done",
  }
}
