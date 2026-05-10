import { FolderSearch, SearchCheck, TriangleAlert } from "lucide-react"
import type { TFunction } from "i18next"
import type { AgentRuntimeEvent } from "@/api"
import type { ComputerCard } from "./swarm-panel-view-model"

export function getContextCard(
  runtimeEvents: AgentRuntimeEvent[],
  t: TFunction
): ComputerCard | null {
  const workspaceEvent = [...runtimeEvents]
    .reverse()
    .find(
      (
        item
      ): item is Extract<AgentRuntimeEvent, { type: "workspace_context_loaded" }> =>
        item.type === "workspace_context_loaded"
    )
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
        ...(workspaceEvent
          ? [
              t("agentChat.swarm.cards.workspaceContext.compactLine", {
                total: getWorkspaceContextTotal(workspaceEvent),
              }),
            ]
          : []),
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

  if (workspaceEvent) {
    const sourceLines = getWorkspaceContextSources(workspaceEvent)
      .slice(0, 3)
      .map((source) =>
        t("agentChat.swarm.cards.workspaceContext.sourceLine", {
          title: source.title ?? source.preview ?? source.id,
          type: source.sourceType ?? source.source_type ?? "source",
        })
      )
    const warnings = workspaceEvent.warnings ?? []
    const readiness = workspaceEvent.readiness ?? "empty"
    return {
      id: "workspace-context-loaded",
      title: t("agentChat.swarm.cards.workspaceContext.title"),
      meta: t("agentChat.swarm.cards.workspaceContext.meta", {
        total: getWorkspaceContextTotal(workspaceEvent),
        readiness,
      }),
      description:
        readiness === "ready"
          ? t("agentChat.swarm.cards.workspaceContext.readyDescription")
          : readiness === "partial"
            ? t("agentChat.swarm.cards.workspaceContext.partialDescription")
            : t("agentChat.swarm.cards.workspaceContext.emptyDescription"),
      icon: readiness === "empty" ? TriangleAlert : FolderSearch,
      lines: [
        ...sourceLines,
        ...warnings.slice(0, 1).map((warning) =>
          t("agentChat.swarm.cards.workspaceContext.warningLine", {
            warning,
          })
        ),
        ...(workspaceEvent.summary ? [workspaceEvent.summary] : []),
      ].slice(0, 4),
      state: readiness === "empty" ? "reworking" : "done",
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

function getWorkspaceContextSources(
  event: Extract<AgentRuntimeEvent, { type: "workspace_context_loaded" }>
) {
  return [
    ...(event.attachmentSources ?? event.attachment_sources ?? []),
    ...(event.productAssetSources ?? event.product_asset_sources ?? []),
    ...(event.productMaterialSources ?? event.product_material_sources ?? []),
    ...(event.artifactSources ?? event.artifact_sources ?? []),
  ]
}

function getWorkspaceContextTotal(
  event: Extract<AgentRuntimeEvent, { type: "workspace_context_loaded" }>
) {
  const counts = event.counts
  if (!counts) return getWorkspaceContextSources(event).length
  return (
    (counts.conversationFiles ?? counts.conversation_files ?? 0) +
    (counts.productAssets ?? counts.product_assets ?? 0) +
    (counts.productMaterials ?? counts.product_materials ?? 0) +
    (counts.workspaceArtifacts ?? counts.workspace_artifacts ?? 0)
  )
}
