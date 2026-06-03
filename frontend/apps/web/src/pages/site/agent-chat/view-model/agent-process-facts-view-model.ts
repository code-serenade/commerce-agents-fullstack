import type { TFunction } from "i18next"
import type { AgentRuntimeEvent } from "@/api"
import {
  getDispatchPlanContextBudget,
  getDispatchPlanReasoningSummary,
  getDispatchPlanRouteMode,
  getDispatchPlanSkills,
  getLatestRouteEvent,
  getLatestDispatchPlan,
  getRouteMode,
  getRoutePrimaryRole,
} from "./agent-process-runtime-view-model"
import { getAdaptiveDispatchFact } from "./agent-adaptive-dispatch-fact-view-model"
import { formatRuntimeRole, type AgentDispatchFact } from "./agent-process-kind-view-model"
import { appendAuditFacts } from "./agent-process-audit-facts-view-model"
import { selectDispatchFacts } from "./agent-dispatch-fact-selection"
import { appendMaterialContextFacts } from "./agent-material-context-facts-view-model"
import { appendSkillAuditFact } from "./agent-process-skill-audit-facts-view-model"

export function getDispatchFacts(
  runtimeEvents: AgentRuntimeEvent[],
  t: TFunction
): AgentDispatchFact[] {
  const facts: AgentDispatchFact[] = []
  appendDispatchPlanFact(facts, runtimeEvents, t)
  appendRouteFact(facts, runtimeEvents, t)
  appendAuditFacts(facts, runtimeEvents, t)
  appendSkillAuditFact(facts, runtimeEvents, t)
  appendMaterialContextFacts(facts, runtimeEvents, t)
  appendMemoryFact(facts, runtimeEvents, t)
  appendConversationContextFact(facts, runtimeEvents, t)
  appendContextFact(facts, runtimeEvents, t)
  appendRagFact(facts, runtimeEvents, t)
  appendOutputFact(facts, runtimeEvents, t)

  return facts.length
    ? selectDispatchFacts(facts, t)
    : [
        {
          detail: t("agentChat.thread.dispatchFacts.empty.detail"),
          label: t("agentChat.thread.dispatchFacts.empty.label"),
          tone: "default",
        },
      ]
}

function appendDispatchPlanFact(
  facts: AgentDispatchFact[],
  runtimeEvents: AgentRuntimeEvent[],
  t: TFunction
) {
  const plan = getLatestDispatchPlan(runtimeEvents)
  if (!plan) return

  facts.push({
    detail: t("agentChat.thread.dispatchFacts.plan.detail", {
      difficulty: plan.difficulty,
      mode: getDispatchPlanRouteMode(plan),
    }),
    label: t("agentChat.thread.dispatchFacts.plan.label"),
    tone: "success",
  })

  const adaptiveFact = getAdaptiveDispatchFact(plan, t)
  if (adaptiveFact) facts.push(adaptiveFact)

  const reasoning = getDispatchPlanReasoningSummary(plan)
  if (reasoning.trim()) {
    facts.push({
      detail: reasoning,
      label: t("agentChat.thread.dispatchFacts.reasoning.label"),
      tone: "default",
    })
  }

  const skills = getDispatchPlanSkills(plan)
  if (skills.length > 0) {
    facts.push({
      detail: skills.join(", "),
      label: t("agentChat.thread.dispatchFacts.plannedSkills.label"),
      tone: "default",
    })
  }

  const budget = getDispatchPlanContextBudget(plan)
  if (budget) {
    const tone = budget.historySummaryAvailable ?? budget.history_summary_available
      ? "success"
      : "default"
    const history = budget.recentHistoryMessageCount ?? budget.recent_history_message_count ?? 0
    const totalHistory =
      budget.historyTotalMessageCount ?? budget.history_total_message_count ?? history
    const truncated = budget.historyTruncated ?? budget.history_truncated ?? false
    facts.push({
      detail: t("agentChat.thread.dispatchFacts.contextBudget.detail", {
        history,
        materials:
          (budget.explicitProductCount ?? budget.explicit_product_count ?? 0) +
          (budget.explicitContextRefCount ?? budget.explicit_context_ref_count ?? 0) +
          (budget.workspaceContextRefCount ?? budget.workspace_context_ref_count ?? 0) +
          (budget.workspacePackItemCount ?? budget.workspace_pack_item_count ?? 0),
      }),
      label: t("agentChat.thread.dispatchFacts.contextBudget.label"),
      tone,
    })
    if (truncated) {
      facts.push({
        detail: t("agentChat.thread.dispatchFacts.historyWindow.truncated", {
          recent: history,
          total: totalHistory,
        }),
        label: t("agentChat.thread.dispatchFacts.historyWindow.label"),
        tone: "success",
      })
    } else if (history > 0) {
      facts.push({
        detail: t("agentChat.thread.dispatchFacts.historyWindow.full", {
          recent: history,
          total: totalHistory,
        }),
        label: t("agentChat.thread.dispatchFacts.historyWindow.label"),
        tone: "success",
      })
    }
  }
}

function appendConversationContextFact(
  facts: AgentDispatchFact[],
  runtimeEvents: AgentRuntimeEvent[],
  t: TFunction
) {
  const context = [...runtimeEvents]
    .reverse()
    .find((event) => event.type === "conversation_context_loaded")
  if (!context) return

  const recent = context.messageCount ?? context.message_count ?? 0
  const total = context.totalMessageCount ?? context.total_message_count ?? recent
  const relevantOlder = context.relevantOlderCount ?? context.relevant_older_count ?? 0
  const truncated = context.truncated ?? false
  facts.push({
    detail: t("agentChat.thread.dispatchFacts.conversationContext.detail", {
      recent,
      relevantOlder,
      total,
    }),
    label: t("agentChat.thread.dispatchFacts.conversationContext.label"),
    tone: truncated || total > 0 ? "success" : "default",
  })
}

function appendMemoryFact(
  facts: AgentDispatchFact[],
  runtimeEvents: AgentRuntimeEvent[],
  t: TFunction
) {
  const memory = [...runtimeEvents]
    .reverse()
    .find((event) => event.type === "memory_loaded")
  if (!memory) return

  facts.push({
    detail: t("agentChat.thread.dispatchFacts.memory.detail", {
      count: memory.count ?? 0,
      kinds: (memory.kinds ?? []).join(", ") || "-",
    }),
    label: t("agentChat.thread.dispatchFacts.memory.label"),
    tone: "success",
  })
}

function appendContextFact(
  facts: AgentDispatchFact[],
  runtimeEvents: AgentRuntimeEvent[],
  t: TFunction
) {
  const context = [...runtimeEvents]
    .reverse()
    .find((event) => event.type === "context_loaded")
  if (!context) return

  facts.push({
    detail: t("agentChat.thread.dispatchFacts.context.detail", {
      knowledge: context.knowledge_count ?? 0,
      products: context.product_count ?? context.productIds?.length ?? 0,
      sources: context.source_count ?? context.refs?.length ?? 0,
    }),
    label: t("agentChat.thread.dispatchFacts.context.label"),
    tone: "success",
  })
}

function appendRagFact(
  facts: AgentDispatchFact[],
  runtimeEvents: AgentRuntimeEvent[],
  t: TFunction
) {
  const rag = [...runtimeEvents]
    .reverse()
    .find((event) => event.type === "rag_context_loaded")
  if (!rag) return

  facts.push({
    detail: rag.degraded
      ? t("agentChat.thread.dispatchFacts.rag.degraded", {
          reason: rag.degradedReason ?? rag.degraded_reason ?? "-",
        })
      : t("agentChat.thread.dispatchFacts.rag.detail", {
          sources: rag.sourceCount ?? rag.source_count ?? 0,
        }),
    label: t("agentChat.thread.dispatchFacts.rag.label"),
    tone: rag.degraded ? "warning" : "success",
  })
}

function appendRouteFact(
  facts: AgentDispatchFact[],
  runtimeEvents: AgentRuntimeEvent[],
  t: TFunction
) {
  const route = getLatestRouteEvent(runtimeEvents)
  if (!route) return

  facts.push({
    detail: t("agentChat.thread.dispatchFacts.route.detail", {
      mode: getRouteMode(route),
      role: formatRuntimeRole(getRoutePrimaryRole(route), t),
      strategy: route.strategyId ?? route.strategy_id ?? "-",
    }),
    label: t("agentChat.thread.dispatchFacts.route.label"),
    tone: getRouteMode(route) === "chat_bypass" ? "default" : "success",
  })
}

function appendOutputFact(
  facts: AgentDispatchFact[],
  runtimeEvents: AgentRuntimeEvent[],
  t: TFunction
) {
  if (
    !runtimeEvents.some(
      (event) =>
        event.type === "finished" ||
        event.type === "task_graph_finished" ||
        event.type === "workspace_run_completed"
    )
  ) {
    return
  }

  facts.push({
    detail: t("agentChat.thread.dispatchFacts.output.detail"),
    label: t("agentChat.thread.dispatchFacts.output.label"),
    tone: "success",
  })
}
