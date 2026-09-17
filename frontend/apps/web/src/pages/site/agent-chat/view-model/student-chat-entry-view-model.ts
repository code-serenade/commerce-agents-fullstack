import type { TFunction } from "i18next"

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
      detail: t("agentChat.thread.studentTaskEntries.campaign.detail"),
      prompt: t("agentChat.thread.studentTaskEntries.campaign.prompt"),
      title: t("agentChat.thread.studentTaskEntries.campaign.title"),
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
    {
      detail: t("agentChat.thread.studentTaskEntries.competitor.detail"),
      prompt: t("agentChat.thread.studentTaskEntries.competitor.prompt"),
      title: t("agentChat.thread.studentTaskEntries.competitor.title"),
    },
    {
      detail: t("agentChat.thread.studentTaskEntries.visual.detail"),
      prompt: t("agentChat.thread.studentTaskEntries.visual.prompt"),
      title: t("agentChat.thread.studentTaskEntries.visual.title"),
    },
    {
      detail: t("agentChat.thread.studentTaskEntries.live.detail"),
      prompt: t("agentChat.thread.studentTaskEntries.live.prompt"),
      title: t("agentChat.thread.studentTaskEntries.live.title"),
    },
    {
      detail: t("agentChat.thread.studentTaskEntries.budget.detail"),
      prompt: t("agentChat.thread.studentTaskEntries.budget.prompt"),
      title: t("agentChat.thread.studentTaskEntries.budget.title"),
    },
    {
      detail: t("agentChat.thread.studentTaskEntries.refund.detail"),
      prompt: t("agentChat.thread.studentTaskEntries.refund.prompt"),
      title: t("agentChat.thread.studentTaskEntries.refund.title"),
    },
    {
      detail: t("agentChat.thread.studentTaskEntries.execution.detail"),
      prompt: t("agentChat.thread.studentTaskEntries.execution.prompt"),
      title: t("agentChat.thread.studentTaskEntries.execution.title"),
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
