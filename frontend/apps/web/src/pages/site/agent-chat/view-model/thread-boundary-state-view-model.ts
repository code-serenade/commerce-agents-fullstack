import type { AgentRuntimeEvent, AgentWorkerReport } from "@/api"
import { readRuntimeNodeId, resolveWorkerNodeId } from "./agent-runtime-event-mapping"

const CATEGORY_LEVEL_BOUNDARY_MARKER = "category_level_boundary"
const PRODUCT_DATA_GAP_TERMS = [
  "product context",
  "product metric",
  "product metrics",
  "product_ref",
  "product_name",
  textFromCodes([21830, 21697, 19978, 19979, 25991]),
  textFromCodes([21830, 21697, 25351, 26631]),
  textFromCodes([20135, 21697, 32534, 21495]),
  textFromCodes([21830, 21697, 32534, 21495]),
  textFromCodes([21830, 21697, 21517, 31216]),
  textFromCodes([20135, 21697, 21517, 31216]),
  textFromCodes([20855, 20307, 21830, 21697]),
  textFromCodes([25351, 23450, 21830, 21697]),
  textFromCodes([38656, 35201, 34917, 20805]),
  textFromCodes([38656, 35201, 25552, 20379]),
  textFromCodes([32570, 23569]),
  textFromCodes([32570, 20047]),
  textFromCodes([26410, 25552, 20379]),
  "missing_user_input",
  "explicit_product_context_required",
]
const CATEGORY_LEVEL_TERMS = [
  textFromCodes([21654, 21857]),
  textFromCodes([39278, 21697]),
  textFromCodes([39135, 21697]),
  textFromCodes([31867, 21830, 21697]),
  textFromCodes([21830, 21697, 31867]),
  textFromCodes([21697, 31867, 20998, 26512]),
  textFromCodes([31867, 30446, 32423]),
  textFromCodes([31867, 30446]),
  textFromCodes([21697, 31867]),
  textFromCodes([21654, 21857, 31867]),
  textFromCodes([31867, 21830, 21697]),
  textFromCodes([31867, 20135, 21697]),
  textFromCodes([26410, 25351, 23450, 21830, 21697]),
  textFromCodes([26410, 36873, 21830, 21697]),
  textFromCodes([19981, 24341, 29992, 20219, 20309, 39044, 35774, 21830, 21697]),
  textFromCodes([19981, 24341, 29992, 20219, 20309, 21830, 21697]),
  textFromCodes([19981, 36873, 25321, 21830, 21697]),
  "category",
  "without selected product",
  "no selected product",
]
const QUALITY_BOUNDARY_GAP_TERMS = [
  "data role should provide at least two findings",
  "data role should mark data or interpretation risks",
  "service role output lacks service-specific content",
  "service role should provide scripts",
  "engineering role output lacks technical content",
  "engineering role should provide technical risks",
  "design role output lacks design-specific content",
  "design role should provide visual or page recommendations",
  "creative role output lacks creative-specific content",
  "creative role should provide usable creative directions",
  "accounting role output lacks financial content",
  "accounting role should provide budget or financial risks",
  "web role output lacks web or SEO-specific content",
  "web role should provide SEO, keyword, or conversion recommendations",
  "ops role should provide at least three recommendations",
  "structured role output is missing",
  "role output has open questions",
  "open questions remain",
  "worker reported failed status",
]
const NO_SELECTED_PRODUCT_TERMS = [
  ...CATEGORY_LEVEL_TERMS,
  textFromCodes([19981, 24341, 29992, 20219, 20309, 39044, 35774, 21830, 21697]),
  textFromCodes([19981, 24341, 29992, 39044, 35774, 21830, 21697]),
  textFromCodes([19981, 20351, 29992, 39044, 35774, 21830, 21697]),
  textFromCodes([19981, 20351, 29992, 40664, 35748, 21830, 21697]),
  textFromCodes([19981, 36873, 25321, 21830, 21697]),
  textFromCodes([19981, 32465, 23450, 21830, 21697]),
]
const CATEGORY_LEVEL_BOUNDARY_SKILLS = [
  "query_product_metrics",
  "query_store_metrics",
  "query_campaign_metrics",
  "data_dashboard",
  "data_funnel_analysis",
  "data_comprehensive_diagnosis",
  "service_query_product",
  "search_market_info",
  "search_industry_benchmarks",
  "search_product_reviews",
  "search_platform_policy",
  "search_trends",
  "coordination_create_task",
  "coordination_create_campaign",
  "coordination_external_agent_call",
  "platform_update_price",
  "platform_update_inventory",
  "platform_sync_products",
  "engineering_system_check",
]

export function hasBlockingTaskFailure(runtimeEvents: AgentRuntimeEvent[]) {
  return runtimeEvents.some((event) => {
    if (event.type !== "task_node_failed") return false
    return !isNonBlockingNodeFailure(event, runtimeEvents)
  })
}

export function hasBlockingWorkspaceFailure(runtimeEvents: AgentRuntimeEvent[]) {
  return runtimeEvents.some((event) => {
    if (event.type !== "workspace_run_failed") return false
    return !isNonBlockingWorkspaceFailure(event, runtimeEvents)
  })
}

export function isNonBlockingWorkspaceFailure(
  event: Extract<AgentRuntimeEvent, { type: "workspace_run_failed" }>,
  runtimeEvents: AgentRuntimeEvent[]
) {
  if (containsBoundaryMarker(`${event.summary ?? ""} ${stringifyUnknown(event)}`)) {
    return true
  }
  return runtimeEvents.some((item) => {
    if (item.type !== "task_node_failed") return false
    return isNonBlockingNodeFailure(item, runtimeEvents)
  })
}

export function isNonBlockingNodeFailure(
  event: Extract<AgentRuntimeEvent, { type: "task_node_failed" }>,
  runtimeEvents: AgentRuntimeEvent[]
) {
  const nodeId = readRuntimeNodeId(event)
  if (containsBoundaryMarker(event.reason ?? "")) return true
  if (isLegacyCategoryLevelProductGap(event.reason ?? "", runtimeEvents)) return true
  return runtimeEvents.some(
    (item) =>
      item.type === "task_node_reported" &&
      readRuntimeNodeId(item) === nodeId &&
      isBoundaryReport(item.report)
  )
}

export function isBoundaryReport(report: AgentWorkerReport) {
  const roleOutput = report.role_output ?? report.roleOutput
  return containsBoundaryMarker(
    [
      report.content,
      ...report.evidence,
      ...report.risks,
      ...readOpenQuestions(report),
      roleOutput?.summary,
      ...(roleOutput?.evidence ?? []),
      ...(roleOutput?.risks ?? []),
      ...readRoleOpenQuestions(roleOutput),
    ]
      .filter(Boolean)
      .join(" ")
  ) || isReportWithCategoryProductGap(report)
}

export function isBoundaryEvaluation(
  evaluation: Extract<AgentRuntimeEvent, { type: "task_node_evaluated" }>["evaluation"],
  runtimeEvents: AgentRuntimeEvent[]
) {
  if (evaluation.passed) return true
  return evaluation.reasons.every((reason) =>
    isLegacyCategoryLevelProductGap(reason, runtimeEvents)
  )
}

export function isNonBlockingSkillGap(
  event: Extract<AgentRuntimeEvent, { type: "skill_result" }>,
  runtimeEvents: AgentRuntimeEvent[] = []
) {
  if (!["failed", "not_found", "validation_error", "deferred"].includes(event.status)) {
    return false
  }
  const text = `${event.skill_id} ${event.summary ?? ""} ${event.error ?? ""} ${stringifyUnknown(event.output)}`
  if (containsBoundaryMarker(text)) return true
  return isLegacyCategoryLevelProductGap(text, runtimeEvents)
}

export function resolveSkillResultNodeId(
  event: Extract<AgentRuntimeEvent, { type: "skill_result" }>
) {
  return resolveWorkerNodeId(event)
}

export function containsBoundaryMarker(value: string) {
  return value.includes(CATEGORY_LEVEL_BOUNDARY_MARKER)
}

function isLegacyCategoryLevelProductGap(
  value: string,
  runtimeEvents: AgentRuntimeEvent[]
) {
  if (!isCategoryLevelWithoutSelectedProduct(runtimeEvents)) return false
  const text = value.toLowerCase()
  if (containsAny(text, QUALITY_BOUNDARY_GAP_TERMS)) return true
  const productSkillGap =
    CATEGORY_LEVEL_BOUNDARY_SKILLS.some((skill) => text.includes(skill)) ||
    text.includes("product_metrics_required") ||
    text.includes("context_required")
  const productDataGap = containsAny(text, PRODUCT_DATA_GAP_TERMS)
  const roleOrAdapterGap =
    text.includes("not allowed for role") ||
    text.includes("invalid parameter") ||
    text.includes("expected integer") ||
    text.includes("adapter required") ||
    text.includes("not_configured") ||
    text.includes("pending approval")
  return productSkillGap && (productDataGap || roleOrAdapterGap)
}

function isReportWithCategoryProductGap(report: AgentWorkerReport) {
  const roleOutput = report.role_output ?? report.roleOutput
  const text = [
    report.content,
    ...report.evidence,
    ...report.risks,
    ...readOpenQuestions(report),
    roleOutput?.summary,
    ...(roleOutput?.findings ?? []),
    ...(roleOutput?.recommendations ?? []),
    ...(roleOutput?.evidence ?? []),
    ...(roleOutput?.risks ?? []),
    ...readRoleOpenQuestions(roleOutput),
  ]
    .filter(Boolean)
    .join(" ")
    .toLowerCase()
  const hasCategoryScope = containsAny(text, CATEGORY_LEVEL_TERMS)
  const hasGap = containsAny(text, PRODUCT_DATA_GAP_TERMS) || text.includes("context.selected_summary")
  return hasCategoryScope && hasGap
}

function isCategoryLevelWithoutSelectedProduct(runtimeEvents: AgentRuntimeEvent[]) {
  const selectedContextLoaded = runtimeEvents.some((event) => {
    if (event.type !== "context_loaded") return false
    const productCount = event.product_count ?? event.productIds?.length ?? event.product_ids?.length ?? 0
    const productRefs = [
      ...(event.product_ids ?? []),
      ...(event.productIds ?? []),
      ...(event.refs ?? []),
      ...(event.contextRefs ?? []),
    ]
    return productCount > 0 || productRefs.some((ref) => ref.startsWith("product:"))
  })
  if (selectedContextLoaded) return false

  return runtimeEvents.some((event) => {
    if (event.type !== "task_graph_planned") return false
    return isNoSelectedProductTaskText(event.graph.root_task)
  })
}

function isNoSelectedProductTaskText(value: string) {
  return containsAny(value.toLowerCase(), NO_SELECTED_PRODUCT_TERMS)
}

function containsAny(value: string, needles: string[]) {
  return needles.some((needle) => value.includes(needle))
}

function textFromCodes(codes: number[]) {
  return String.fromCodePoint(...codes)
}

function readOpenQuestions(report: AgentWorkerReport) {
  return report.open_questions ?? report.openQuestions ?? []
}

function readRoleOpenQuestions(roleOutput: AgentWorkerReport["role_output"] | AgentWorkerReport["roleOutput"]) {
  if (!roleOutput) return []
  const camelCaseQuestions =
    "openQuestions" in roleOutput ? roleOutput.openQuestions : undefined
  return roleOutput.open_questions ?? camelCaseQuestions ?? []
}

function stringifyUnknown(value: unknown) {
  if (value === null || value === undefined) return ""
  if (typeof value === "string") return value
  try {
    return JSON.stringify(value)
  } catch {
    return ""
  }
}
