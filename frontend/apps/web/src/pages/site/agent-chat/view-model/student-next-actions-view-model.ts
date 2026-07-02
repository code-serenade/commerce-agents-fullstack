import type { TFunction } from "i18next"
import type { AgentChatMessage } from "../types"
import {
  getContextualProductFollowupPrompts,
  hasConcreteProductFollowupSignal,
  matchesProductFollowupIntent,
} from "./student-product-followup-view-model"

export interface StudentStartFlowStep {
  detail: string
  title: string
}

export interface StudentTaskEntry {
  detail: string
  prompt: string
  title: string
}

export interface StudentPromptRecipeItem {
  label: string
  value: string
}

export type StudentNextActionGroup =
  | "campaign"
  | "data"
  | "default"
  | "product"
  | "productFollowup"
  | "productNoMaterial"
  | "quick"
  | "report"
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
  const text = getMessageIntentText(message, previousUserContent)

  const hasSelectedProduct = Boolean(message.contextSummary?.productIds.length)

  if (matchesSourceLookupIntent(text)) return "default"
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
  if (message.contextSummary?.strategyId === "quick") return "quick"
  return "default"
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
  return /product|sku|item|\u5546\u54c1|\u9009\u54c1|\u5356\u70b9|\u4eba\u7fa4|\u98ce\u9669|\u6807\u9898|\u4e3b\u56fe|\u5b9a\u4ef7|\u4ef7\u683c|\u52a0\u8d2d|\u70b9\u51fb|\u9632\u6652\u8863/i.test(text)
}

function matchesCampaignIntent(text: string) {
  return /campaign|activity|budget|\u6d3b\u52a8|\u6295\u653e|\u9884\u7b97|\u6267\u884c/i.test(text)
}

function matchesDataIntent(text: string) {
  return /data|metric|diagnos|roi|gmv|\u6570\u636e|\u6307\u6807|\u8bca\u65ad|\u5f02\u5e38|\u8f6c\u5316/i.test(text)
}

function matchesReportIntent(text: string) {
  return /report|review|summary|\u590d\u76d8|\u62a5\u544a|\u603b\u7ed3|\u63d0\u4ea4/i.test(text)
}

function matchesTaskIntent(text: string) {
  return /task|deliver|checklist|\u4efb\u52a1|\u6e05\u5355|\u4ea4\u4ed8|\u6b65\u9aa4/i.test(text)
}

function matchesSourceLookupIntent(text: string) {
  return /source link|official notice|official rule|published date|\u8bf7\u8054\u7f51\u641c\u7d22|\u8054\u7f51\u641c\u7d22|\u6765\u6e90\u94fe\u63a5|\u5b98\u65b9\u516c\u5f00\u89c4\u5219|\u5b98\u65b9\u516c\u544a|\u89c4\u5219\u4e2d\u5fc3|\u53ea\u8f93\u51fa\u6765\u6e90|\u53d1\u5e03\u65e5\u671f|\u53d1\u5e03\u65f6\u95f4|\u9002\u7528\u98ce\u9669\u70b9/i.test(text)
}

export function getStudentStartFlowSteps(t: TFunction): StudentStartFlowStep[] {
  return [
    {
      detail: t("agentChat.thread.studentFlow.ask.detail"),
      title: t("agentChat.thread.studentFlow.ask.title"),
    },
    {
      detail: t("agentChat.thread.studentFlow.watch.detail"),
      title: t("agentChat.thread.studentFlow.watch.title"),
    },
    {
      detail: t("agentChat.thread.studentFlow.deliver.detail"),
      title: t("agentChat.thread.studentFlow.deliver.title"),
    },
  ]
}

export function getStudentTaskEntries(t: TFunction): StudentTaskEntry[] {
  return [
    {
      detail: t("agentChat.thread.studentTaskEntries.start.detail"),
      prompt: t("agentChat.thread.studentTaskEntries.start.prompt"),
      title: t("agentChat.thread.studentTaskEntries.start.title"),
    },
    {
      detail: t("agentChat.thread.studentTaskEntries.analyze.detail"),
      prompt: t("agentChat.thread.studentTaskEntries.analyze.prompt"),
      title: t("agentChat.thread.studentTaskEntries.analyze.title"),
    },
    {
      detail: t("agentChat.thread.studentTaskEntries.plan.detail"),
      prompt: t("agentChat.thread.studentTaskEntries.plan.prompt"),
      title: t("agentChat.thread.studentTaskEntries.plan.title"),
    },
    {
      detail: t("agentChat.thread.studentTaskEntries.data.detail"),
      prompt: t("agentChat.thread.studentTaskEntries.data.prompt"),
      title: t("agentChat.thread.studentTaskEntries.data.title"),
    },
    {
      detail: t("agentChat.thread.studentTaskEntries.review.detail"),
      prompt: t("agentChat.thread.studentTaskEntries.review.prompt"),
      title: t("agentChat.thread.studentTaskEntries.review.title"),
    },
  ]
}

export function getStudentPromptRecipeItems(
  t: TFunction
): StudentPromptRecipeItem[] {
  return [
    {
      label: t("agentChat.thread.studentPromptRecipe.goal.label"),
      value: t("agentChat.thread.studentPromptRecipe.goal.value"),
    },
    {
      label: t("agentChat.thread.studentPromptRecipe.material.label"),
      value: t("agentChat.thread.studentPromptRecipe.material.value"),
    },
    {
      label: t("agentChat.thread.studentPromptRecipe.limit.label"),
      value: t("agentChat.thread.studentPromptRecipe.limit.value"),
    },
    {
      label: t("agentChat.thread.studentPromptRecipe.output.label"),
      value: t("agentChat.thread.studentPromptRecipe.output.value"),
    },
  ]
}
