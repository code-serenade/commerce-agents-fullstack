import type { AgentRuntimeEvent } from "@/api"

const textFromCodes = (codes: number[]) =>
  String.fromCharCode(...codes)

const thermosEntity = textFromCodes([20415, 25658, 20445, 28201, 26479])

export const dispatchFactTestLabels = {
  contextBudgetDetail: "12 history 3 materials",
  contextBudgetLabel: "Context budget",
  historyWindowDetail: "recent 12 total 18",
  historyWindowLabel: "History window",
  planDetail: "complex task_graph",
  planLabel: "Plan",
  roleAuditDetail: "planned Data, Ops executed Data, Ops",
  roleAuditLabel: "Role audit",
  roleAuditWarning: "missing Ops extra - planned Data, Ops executed Data",
  searchAuditDetail:
    "planned search_competitor executed search_competitor sources 1 claims 1 gaps 0 confidence 72% strength standard",
  searchAuditLabel: "Search audit",
  searchAuditPending:
    "planned search_platform_policy requested search_platform_policy",
  skillAuditDetail:
    "planned ops_pricing_strategy requested ops_pricing_strategy succeeded ops_pricing_strategy failed - pending -",
  skillAuditLabel: "Skill audit",
  skillAuditPending:
    "planned ops_inventory_plan requested ops_inventory_plan succeeded - failed - pending ops_inventory_plan(no_result)",
  materialAuditLabel: "Material audit",
  memoryDetailWithRecords:
    `loaded 2 kinds conversation_fact records memory_card:facts [memory_card, entity:${thermosEntity}] 86 chars; memory_card:decisions [memory_card, decision] 42 chars truncated`,
  memoryLabel: "History memory",
  workspaceDetail: "partial 1/3",
  workspaceLabel: "Workspace materials",
  workspaceOmittedDetail: "2 omitted file-b, file-c",
  workspaceOmittedLabel: "Not in context",
  workspaceWarningDetail: "unsupported pdf",
  workspaceWarningLabel: "Material note",
}

const materialAuditSummaries = {
  empty: "no material",
  missingPack: "missing pack",
  notReady: "not ready",
  omitted: "omitted",
  pendingPack: "pending pack",
  ready: "ready",
  unreadablePacked: "unreadable packed",
  warnings: "warnings",
} as const

export const t = ((key: string, params?: Record<string, unknown>) => {
  if (key === "agentChat.thread.dispatchFacts.plan.label") {
    return dispatchFactTestLabels.planLabel
  }
  if (key === "agentChat.thread.dispatchFacts.plan.detail") {
    return `${params?.difficulty} ${params?.mode}`
  }
  if (key === "agentChat.thread.dispatchFacts.contextBudget.label") {
    return dispatchFactTestLabels.contextBudgetLabel
  }
  if (key === "agentChat.thread.dispatchFacts.contextBudget.detail") {
    return `${params?.history} history ${params?.materials} materials`
  }
  if (key === "agentChat.thread.dispatchFacts.historyWindow.label") {
    return dispatchFactTestLabels.historyWindowLabel
  }
  if (key === "agentChat.thread.dispatchFacts.historyWindow.truncated") {
    return `recent ${params?.recent} total ${params?.total}`
  }
  if (key === "agentChat.thread.dispatchFacts.workspace.label") {
    return dispatchFactTestLabels.workspaceLabel
  }
  if (key === "agentChat.thread.dispatchFacts.workspace.detail") {
    return `${params?.readiness} ${params?.readable}/${params?.total}`
  }
  if (key === "agentChat.thread.dispatchFacts.workspaceWarning.label") {
    return dispatchFactTestLabels.workspaceWarningLabel
  }
  if (key === "agentChat.thread.dispatchFacts.workspaceWarning.detail") {
    return String(params?.warning)
  }
  if (key === "agentChat.thread.dispatchFacts.workspaceOmitted.label") {
    return dispatchFactTestLabels.workspaceOmittedLabel
  }
  if (key === "agentChat.thread.dispatchFacts.workspaceOmitted.detail") {
    return `${params?.count} omitted ${params?.refs}`
  }
  if (key === "agentChat.thread.roleFallbackTitles.data") return "Data"
  if (key === "agentChat.thread.roleFallbackTitles.ops") return "Ops"
  if (key === "agentChat.thread.roleFallbackTitles.search") return "Search"
  if (key === "agentChat.thread.dispatchFacts.roleAudit.label") {
    return dispatchFactTestLabels.roleAuditLabel
  }
  if (key === "agentChat.thread.dispatchFacts.roleAudit.detail") {
    return `planned ${params?.planned} executed ${params?.executed}`
  }
  if (key === "agentChat.thread.dispatchFacts.roleAudit.warning") {
    return `missing ${params?.missing} extra ${params?.extra} planned ${params?.planned} executed ${params?.executed}`
  }
  if (key === "agentChat.thread.dispatchFacts.searchAudit.label") {
    return dispatchFactTestLabels.searchAuditLabel
  }
  if (key === "agentChat.thread.dispatchFacts.searchAudit.detail") {
    return `planned ${params?.planned} executed ${params?.executed} sources ${params?.sources} claims ${params?.claims} gaps ${params?.gaps} confidence ${params?.confidence} strength ${params?.strength}`
  }
  if (key === "agentChat.thread.dispatchFacts.searchAudit.pending") {
    return `planned ${params?.planned} requested ${params?.requested}`
  }
  if (key === "agentChat.thread.dispatchFacts.skillAudit.label") {
    return dispatchFactTestLabels.skillAuditLabel
  }
  if (key === "agentChat.thread.dispatchFacts.skillAudit.detail") {
    return `planned ${params?.planned} requested ${params?.requested} succeeded ${params?.succeeded} failed ${params?.failed} pending ${params?.pending}`
  }
  if (key === "agentChat.thread.dispatchFacts.materialAudit.label") {
    return dispatchFactTestLabels.materialAuditLabel
  }
  if (key === "agentChat.thread.dispatchFacts.materialAudit.detail") {
    return `${params?.summary} selected ${params?.selected} packed ${params?.packed} readable ${params?.readable} missing ${params?.missing} omitted ${params?.omitted} unreadable ${params?.unreadable} warnings ${params?.warnings} readiness ${params?.readiness}`
  }
  if (key === "agentChat.thread.dispatchFacts.materialAudit.summary.empty") {
    return materialAuditSummaries.empty
  }
  if (key === "agentChat.thread.dispatchFacts.materialAudit.summary.missingPack") {
    return `${materialAuditSummaries.missingPack} ${params?.missing}`
  }
  if (key === "agentChat.thread.dispatchFacts.materialAudit.summary.notReady") {
    return `${materialAuditSummaries.notReady} ${params?.readiness}`
  }
  if (key === "agentChat.thread.dispatchFacts.materialAudit.summary.omitted") {
    return `${materialAuditSummaries.omitted} ${params?.omitted}`
  }
  if (key === "agentChat.thread.dispatchFacts.materialAudit.summary.pendingPack") {
    return materialAuditSummaries.pendingPack
  }
  if (key === "agentChat.thread.dispatchFacts.materialAudit.summary.ready") {
    return materialAuditSummaries.ready
  }
  if (key === "agentChat.thread.dispatchFacts.materialAudit.summary.unreadablePacked") {
    return `${materialAuditSummaries.unreadablePacked} ${params?.unreadable}`
  }
  if (key === "agentChat.thread.dispatchFacts.materialAudit.summary.warnings") {
    return `${materialAuditSummaries.warnings} ${params?.warnings}`
  }
  if (key === "agentChat.thread.dispatchFacts.memory.label") {
    return dispatchFactTestLabels.memoryLabel
  }
  if (key === "agentChat.thread.dispatchFacts.memory.detailWithRecords") {
    return `loaded ${params?.count} kinds ${params?.kinds} records ${params?.records}`
  }
  if (key === "agentChat.thread.dispatchFacts.memory.recordChars") {
    return `${params?.count} chars`
  }
  if (key === "agentChat.thread.dispatchFacts.memory.recordTruncated") {
    return "truncated"
  }
  if (key === "agentChat.thread.dispatchPanel.preview.fact") {
    return `${params?.label}: ${params?.detail}`
  }
  if (key === "agentChat.thread.dispatchPanel.preview.heading") {
    return `# ${params?.mode}`
  }
  if (key === "agentChat.thread.dispatchPanel.preview.progress") {
    return `${params?.completed}/${params?.total} ${params?.stage}`
  }
  if (key === "agentChat.thread.dispatchPanel.preview.request") {
    return `request ${params?.request}`
  }
  if (key === "agentChat.thread.dispatchPanel.preview.step") {
    return `${params?.index} ${params?.title}: ${params?.detail}`
  }
  if (key === "agentChat.thread.dispatchPanel.preview.summary") {
    return `summary ${params?.summary}`
  }
  if (key === "agentChat.thread.dispatchPanel.emptyTask") return "empty task"
  if (key === "agentChat.thread.dispatchPanel.previewFiles.multi_agent") {
    return "multi.agent.process.md"
  }
  if (key === "agentChat.thread.dispatchPanel.routeModes.task_graph") {
    return "task graph"
  }
  return key
}) as never

export function node(id: string, role: string) {
  return {
    acceptance: [],
    id,
    input_refs: [],
    max_attempts: 1,
    objective: `${role} objective`,
    role,
    title: role,
    worker_id: `worker.${role}`,
  }
}

export function reported(nodeId: string, role: string): AgentRuntimeEvent {
  return {
    graph_id: "graph-1",
    node_id: nodeId,
    report: {
      content: `${role} output`,
      evidence: [],
      graph_id: "graph-1",
      node_id: nodeId,
      open_questions: [],
      risks: [],
      role,
      role_output: {
        evidence: [],
        findings: [`${role} finding`],
        open_questions: [],
        recommendations: [`${role} recommendation`],
        risks: [],
        summary: `${role} summary`,
      },
      status: "completed",
      task_id: `task.${nodeId}`,
      worker_id: `worker.${role}`,
    },
    type: "task_node_reported",
    worker_id: `worker.${role}`,
  }
}
