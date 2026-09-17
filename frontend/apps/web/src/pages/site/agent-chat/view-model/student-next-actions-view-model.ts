import type { TFunction } from "i18next"
import type { AgentChatMessage } from "../types"
import {
  getContextualProductFollowupPrompts,
  hasConcreteProductFollowupSignal,
  matchesProductFollowupIntent,
} from "./student-product-followup-view-model"

export type StudentNextActionGroup =
  | "campaign"
  | "compliance"
  | "data"
  | "default"
  | "product"
  | "productFollowup"
  | "productNoMaterial"
  | "quick"
  | "report"
  | "service"
  | "task"

export interface StudentNextActionMeta {
  detail: string
  group: StudentNextActionGroup
  label: string
}

export function shouldShowStudentNextActions(message: AgentChatMessage) {
  return (
    message.role === "assistant" &&
    message.id !== "welcome" &&
    Boolean(message.content.trim()) &&
    message.status !== "streaming" &&
    message.status !== "error" &&
    message.status !== "stopped"
  )
}

export function getCurrentLearningProbeMessageId(messages: AgentChatMessage[]) {
  const latestMessage = messages.at(-1)
  return latestMessage && shouldShowStudentNextActions(latestMessage)
    ? latestMessage.id
    : null
}

export function getPersistedLearningProbeMessageId(
  message?: AgentChatMessage
): string | null {
  const candidate = message?.persistedId ?? message?.id
  return candidate &&
    /^[0-9a-f]{8}-[0-9a-f]{4}-[0-9a-f]{4}-[0-9a-f]{4}-[0-9a-f]{12}$/i.test(
      candidate
    )
    ? candidate
    : null
}

export function getStudentNextActionPrompts(
  t: TFunction,
  message?: AgentChatMessage,
  previousUserContent?: string
) {
  const group = message
    ? getStudentNextActionGroup(message, previousUserContent)
    : "default"
  if (message && group === "productFollowup") {
    const contextualPrompts = getContextualProductFollowupPrompts(
      t,
      getMessageIntentText(message, previousUserContent)
    )
    if (contextualPrompts.length > 0) return contextualPrompts
  }

  const prompts = t(`agentChat.thread.nextActions.groups.${group}`, {
    returnObjects: true,
  }) as string[]

  if (Array.isArray(prompts)) return prompts

  return t("agentChat.thread.nextActions.prompts", {
    returnObjects: true,
  }) as string[]
}

export function getStudentNextActionMeta(
  t: TFunction,
  message?: AgentChatMessage,
  previousUserContent?: string
): StudentNextActionMeta {
  const group = message
    ? getStudentNextActionGroup(message, previousUserContent)
    : "default"

  return {
    detail: t(`agentChat.thread.nextActions.meta.${group}.detail`),
    group,
    label: t(`agentChat.thread.nextActions.meta.${group}.label`),
  }
}

export function getStudentNextActionGroup(
  message: AgentChatMessage,
  previousUserContent?: string
): StudentNextActionGroup {
  const hasSelectedProduct = Boolean(message.contextSummary?.productIds.length)
  const directUserGroup = previousUserContent?.trim()
    ? matchStudentNextActionGroup(previousUserContent, hasSelectedProduct)
    : undefined
  if (
    directUserGroup &&
    directUserGroup !== "data" &&
    directUserGroup !== "productNoMaterial"
  ) {
    return directUserGroup
  }

  const text = getMessageIntentText(message, previousUserContent)
  const fallbackGroup = matchStudentNextActionGroup(text, hasSelectedProduct)
  if (directUserGroup === "data" || directUserGroup === "productNoMaterial") {
    return fallbackGroup === "productFollowup" ? fallbackGroup : directUserGroup
  }
  if (fallbackGroup) return fallbackGroup

  if (message.contextSummary?.strategyId === "quick") return "quick"
  return "default"
}

function matchStudentNextActionGroup(
  text: string,
  hasSelectedProduct: boolean
): StudentNextActionGroup | undefined {
  if (matchesSourceLookupIntent(text)) return "default"
  if (matchesServiceIntent(text)) return "service"
  if (matchesConversionAnalyticsIntent(text)) return "data"
  if (matchesComplianceIntent(text)) return "compliance"
  if (
    matchesProductFollowupIntent(text) &&
    hasConcreteProductFollowupSignal(text)
  ) {
    return "productFollowup"
  }
  if (matchesCampaignIntent(text)) return "campaign"
  if (matchesDataIntent(text)) return "data"
  if (matchesProductIntent(text)) {
    return hasSelectedProduct ? "product" : "productNoMaterial"
  }
  if (matchesReportIntent(text)) return "report"
  if (matchesTaskIntent(text)) return "task"
  return undefined
}

function getMessageIntentText(
  message: AgentChatMessage,
  previousUserContent?: string
) {
  return [
    previousUserContent,
    message.content,
    message.contextSummary?.strategyId,
    message.contextSummary?.primaryRole,
    ...(message.contextSummary?.selectedRoles ?? []),
    ...(message.runtimeEvents ?? []).map((event) => event.type),
  ]
    .filter(Boolean)
    .join(" ")
}

function matchesProductIntent(text: string) {
  return /product|sku|item|inventory|stocktake|restock|\u5546\u54c1|\u9009\u54c1|\u5356\u70b9|\u4eba\u7fa4|\u98ce\u9669|\u6807\u9898|\u4e3b\u56fe|\u5b9a\u4ef7|\u4ef7\u683c|\u52a0\u8d2d|\u70b9\u51fb|\u5e93\u5b58|\u76d8\u70b9|\u8865\u8d27|\u6ede\u9500|\u9632\u6652\u8863/i.test(
    text
  )
}

function matchesCampaignIntent(text: string) {
  return /campaign|activity|budget|\u6d3b\u52a8|\u6295\u653e|\u9884\u7b97|\u6267\u884c/i.test(
    text
  )
}

function matchesDataIntent(text: string) {
  return /data|metric|diagnos|roi|gmv|\u6570\u636e|\u6307\u6807|\u8bca\u65ad|\u5f02\u5e38|\u8f6c\u5316/i.test(
    text
  )
}

function matchesConversionAnalyticsIntent(text: string) {
  const hasAnalyticsSignal =
    /\bga4\b|google analytics|analytics|\b(?:view_item|add_to_cart|begin_checkout|add_shipping_info|add_payment_info|purchase)\b|\u7535\u5546\u4e8b\u4ef6|\u4e8b\u4ef6\u53c2\u6570|\u57cb\u70b9|\u4e8b\u4ef6\u8ffd\u8e2a/i.test(
      text
    )
  const hasConversionSignal =
    /checkout|payment|purchase|conversion|\u7ed3\u8d26|\u652f\u4ed8|\u8d2d\u4e70|\u8f6c\u5316/i.test(
      text
    )

  return hasAnalyticsSignal && hasConversionSignal
}

function matchesComplianceIntent(text: string) {
  return /personal information|privacy|personalized recommendation|algorithmic recommendation|consumer rights|\u4e2a\u4eba\u4fe1\u606f|\u9690\u79c1|\u4e2a\u6027\u5316\u63a8\u8350|\u7b97\u6cd5\u63a8\u8350|\u9009\u62e9\u6743|\u4e0d\u9488\u5bf9(?:\u5176)?\u4e2a\u4eba\u7279\u5f81|\u7535\u5b50\u5546\u52a1\u6cd5|\u6d88\u8d39\u8005\u6743\u76ca/i.test(
    text
  )
}

function matchesServiceIntent(text: string) {
  return /after[ -]?sales|refund|return request|deposit refund|complaint|consumer dispute|service recovery|support ticket|\u552e\u540e|\u9000\u6b3e|\u9000\u8fd8|\u62bc\u91d1|\u9000\u8d27|\u6362\u8d27|\u6295\u8bc9|\u5ba2\u8bc9|\u6d88\u8d39\u4e89\u8bae|\u4ea4\u6613\u7ea0\u7eb7|\u7ea0\u7eb7\u5904\u7406|\u5de5\u5355/i.test(
    text
  )
}

function matchesReportIntent(text: string) {
  return /report|review|summary|\u590d\u76d8|\u62a5\u544a|\u603b\u7ed3|\u63d0\u4ea4/i.test(
    text
  )
}

function matchesTaskIntent(text: string) {
  return /task|deliver|checklist|\u4efb\u52a1|\u6e05\u5355|\u4ea4\u4ed8|\u6b65\u9aa4/i.test(
    text
  )
}

function matchesSourceLookupIntent(text: string) {
  return /source link|official notice|official rule|published date|\u8bf7\u8054\u7f51\u641c\u7d22|\u8054\u7f51\u641c\u7d22|\u6765\u6e90\u94fe\u63a5|\u5b98\u65b9\u516c\u5f00\u89c4\u5219|\u5b98\u65b9\u516c\u544a|\u89c4\u5219\u4e2d\u5fc3|\u53ea\u8f93\u51fa\u6765\u6e90|\u53d1\u5e03\u65e5\u671f|\u53d1\u5e03\u65f6\u95f4|\u9002\u7528\u98ce\u9669\u70b9/i.test(
    text
  )
}
