import { SearchCheck } from "lucide-react"
import type { TFunction } from "i18next"
import type { AgentRuntimeEvent } from "@/api"
import type { ComputerCard } from "./swarm-panel-view-model"

export function getContextCard(
  runtimeEvents: AgentRuntimeEvent[],
  t: TFunction
): ComputerCard | null {
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
