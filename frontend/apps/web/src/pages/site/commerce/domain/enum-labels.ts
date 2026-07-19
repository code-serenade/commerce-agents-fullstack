import type { CommerceTranslate } from "../components/commerce-types"

const enumLabelKeys = {
  actionType: {
    approve: "commerce.enumLabels.actionType.approve",
    archive: "commerce.enumLabels.actionType.archive",
    execute: "commerce.enumLabels.actionType.execute",
    publish: "commerce.enumLabels.actionType.publish",
    reject: "commerce.enumLabels.actionType.reject",
    update: "commerce.enumLabels.actionType.update",
  },
  adapterStatus: {
    adapter_not_implemented: "commerce.enumLabels.adapterStatus.notImplemented",
    adapter_required: "commerce.enumLabels.adapterStatus.required",
    degraded: "commerce.enumLabels.adapterStatus.degraded",
    disabled: "commerce.enumLabels.adapterStatus.disabled",
    live: "commerce.enumLabels.adapterStatus.live",
  },
  alertType: {
    agent_approval_required: "commerce.enumLabels.alertType.approvalRequired",
    budget_risk: "commerce.enumLabels.alertType.budgetRisk",
    data_anomaly: "commerce.enumLabels.alertType.dataAnomaly",
    policy_risk: "commerce.enumLabels.alertType.policyRisk",
    system: "commerce.enumLabels.alertType.system",
    task_failed: "commerce.enumLabels.alertType.taskFailed",
  },
  assetType: {
    docs: "commerce.enumLabels.assetType.docs",
    images: "commerce.enumLabels.assetType.images",
    other: "commerce.enumLabels.other",
    videos: "commerce.enumLabels.assetType.videos",
  },
  campaignType: {
    content: "commerce.enumLabels.campaignType.content",
    flash_sale: "commerce.enumLabels.campaignType.flashSale",
    launch: "commerce.enumLabels.campaignType.launch",
    live: "commerce.enumLabels.campaignType.live",
    promotion: "commerce.enumLabels.campaignType.promotion",
    seasonal: "commerce.enumLabels.campaignType.seasonal",
  },
  changeType: {
    description_change: "commerce.enumLabels.changeType.description",
    price: "commerce.enumLabels.changeType.price",
    rating: "commerce.enumLabels.changeType.rating",
    sales: "commerce.enumLabels.changeType.sales",
    stock: "commerce.enumLabels.changeType.stock",
    title_change: "commerce.enumLabels.changeType.title",
  },
  connectionStatus: {
    connected: "commerce.enumLabels.connectionStatus.connected",
    connection_disabled: "commerce.enumLabels.connectionStatus.disabled",
    connection_missing: "commerce.enumLabels.connectionStatus.missing",
    disabled: "commerce.enumLabels.connectionStatus.disabled",
    disconnected: "commerce.enumLabels.connectionStatus.disconnected",
  },
  eventType: {
    "campaign.updated": "commerce.enumLabels.eventType.campaignUpdated",
    "report.created": "commerce.enumLabels.eventType.reportCreated",
    system: "commerce.enumLabels.eventType.system",
    "task.completed": "commerce.enumLabels.eventType.taskCompleted",
    "task.created": "commerce.enumLabels.eventType.taskCreated",
    "task.failed": "commerce.enumLabels.eventType.taskFailed",
  },
  executionMode: {
    adapter: "commerce.enumLabels.executionMode.adapter",
    deferred: "commerce.enumLabels.executionMode.deferred",
    manual: "commerce.enumLabels.executionMode.manual",
    native: "commerce.enumLabels.executionMode.native",
  },
  materialType: {
    context: "commerce.enumLabels.materialType.context",
    copy: "commerce.enumLabels.materialType.copy",
    document: "commerce.enumLabels.materialType.document",
    excel: "commerce.enumLabels.materialType.sheet",
    image: "commerce.enumLabels.materialType.image",
    text: "commerce.enumLabels.materialType.text",
    video: "commerce.enumLabels.materialType.video",
  },
  ownerRole: {
    creative: "commerce.enumLabels.ownerRole.creative",
    data: "commerce.enumLabels.ownerRole.data",
    delivery: "commerce.enumLabels.ownerRole.delivery",
    design: "commerce.enumLabels.ownerRole.design",
    engineering: "commerce.enumLabels.ownerRole.engineering",
    finance: "commerce.enumLabels.ownerRole.finance",
    ops: "commerce.enumLabels.ownerRole.ops",
    planning: "commerce.enumLabels.ownerRole.planning",
    search: "commerce.enumLabels.ownerRole.search",
    seo: "commerce.enumLabels.ownerRole.seo",
    service: "commerce.enumLabels.ownerRole.service",
  },
  readiness: {
    blocked: "commerce.enumLabels.readiness.blocked",
    empty: "commerce.enumLabels.readiness.empty",
    missing: "commerce.enumLabels.readiness.missing",
    partial: "commerce.enumLabels.readiness.partial",
    ready: "commerce.enumLabels.readiness.ready",
    unsupported: "commerce.enumLabels.readiness.unsupported",
  },
  riskLevel: {
    critical: "commerce.enumLabels.riskLevel.high",
    high: "commerce.enumLabels.riskLevel.high",
    info: "commerce.enumLabels.riskLevel.low",
    l1: "commerce.enumLabels.riskLevel.low",
    l2: "commerce.enumLabels.riskLevel.medium",
    l3: "commerce.enumLabels.riskLevel.high",
    low: "commerce.enumLabels.riskLevel.low",
    medium: "commerce.enumLabels.riskLevel.medium",
    warning: "commerce.enumLabels.riskLevel.medium",
  },
  severity: {
    blocking: "commerce.enumLabels.severity.blocking",
    critical: "commerce.enumLabels.severity.critical",
    high: "commerce.enumLabels.severity.high",
    info: "commerce.enumLabels.severity.info",
    low: "commerce.enumLabels.severity.low",
    medium: "commerce.enumLabels.severity.medium",
    warning: "commerce.enumLabels.severity.warning",
  },
  source: {
    agent: "commerce.enumLabels.source.agent",
    api: "commerce.enumLabels.source.api",
    imported: "commerce.enumLabels.source.imported",
    manual: "commerce.enumLabels.source.manual",
    platform_sync: "commerce.enumLabels.source.platformSync",
    system: "commerce.enumLabels.source.system",
  },
  targetType: {
    campaign: "commerce.enumLabels.targetType.campaign",
    execution: "commerce.enumLabels.targetType.execution",
    product: "commerce.enumLabels.targetType.product",
    run: "commerce.enumLabels.targetType.run",
    task: "commerce.enumLabels.targetType.task",
    tool: "commerce.enumLabels.targetType.tool",
    workspace: "commerce.enumLabels.targetType.workspace",
  },
  taskType: {
    analysis: "commerce.enumLabels.taskType.analysis",
    campaign: "commerce.enumLabels.taskType.campaign",
    export: "commerce.enumLabels.taskType.export",
    manual: "commerce.enumLabels.taskType.manual",
    report: "commerce.enumLabels.taskType.report",
    sync: "commerce.enumLabels.taskType.sync",
  },
} as const

export type CommerceEnumGroup = keyof typeof enumLabelKeys

export function getCommerceEnumLabel(
  t: CommerceTranslate,
  group: CommerceEnumGroup,
  value: string | null | undefined
) {
  if (!value?.trim()) return t("commerce.common.none")

  const normalizedValue = normalizeLookupValue(group, value)
  const keys = enumLabelKeys[group] as Record<string, string>
  const labelKey = keys[normalizedValue]
  return labelKey ? t(labelKey) : t("commerce.enumLabels.other")
}

function normalizeLookupValue(group: CommerceEnumGroup, value: string) {
  const normalizedValue = value.trim().toLowerCase()
  if (group !== "changeType") return normalizedValue
  return normalizedValue === "description" || normalizedValue === "title"
    ? `${normalizedValue}_change`
    : normalizedValue
}

export function commerceEnumOptions(
  t: CommerceTranslate,
  group: CommerceEnumGroup,
  values: readonly string[]
) {
  return values.map((value) => ({
    label: getCommerceEnumLabel(t, group, value),
    value,
  }))
}

export function commerceBooleanOptions(t: CommerceTranslate) {
  return [
    { label: t("commerce.common.no"), value: "false" },
    { label: t("commerce.common.yes"), value: "true" },
  ]
}
