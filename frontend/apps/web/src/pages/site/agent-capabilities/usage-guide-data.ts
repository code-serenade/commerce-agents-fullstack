import type { TFunction } from "i18next"
import commerceScreenshotEn from "./assets/usage-guide/commerce.en.png"
import commerceScreenshotZhCn from "./assets/usage-guide/commerce.zh-CN.png"
import preferencesScreenshotEn from "./assets/usage-guide/preferences.en.png"
import preferencesScreenshotZhCn from "./assets/usage-guide/preferences.zh-CN.png"
import productsScreenshotEn from "./assets/usage-guide/products.en.png"
import productsScreenshotZhCn from "./assets/usage-guide/products.zh-CN.png"
import workspacesScreenshotEn from "./assets/usage-guide/workspaces.en.png"
import workspacesScreenshotZhCn from "./assets/usage-guide/workspaces.zh-CN.png"

interface JourneyStepCopy {
  navLabel: string
  title: string
  description: string
  screenshotAlt?: string
  note?: string
}

export interface UsageGuideJourneyStep extends JourneyStepCopy {
  id: string
  screenshot?: string
  pins?: Array<{ marker: string; left: number; top: number }>
}

export type UsageGuideScreenshotLanguage = "en" | "zh-CN"

const journeyAssetsByLanguage: Record<
  UsageGuideScreenshotLanguage,
  Array<Omit<UsageGuideJourneyStep, keyof JourneyStepCopy>>
> = {
  "zh-CN": [
    {
      id: "prepare",
      screenshot: productsScreenshotZhCn,
      pins: [
        { marker: "1", left: 91.6, top: 15.8 },
        { marker: "2", left: 83.4, top: 37.6 },
      ],
    },
    {
      id: "ask",
      screenshot: commerceScreenshotZhCn,
      pins: [
        { marker: "1", left: 10, top: 11.5 },
        { marker: "2", left: 58.3, top: 48 },
        { marker: "3", left: 54.8, top: 54.8 },
      ],
    },
    {
      id: "preferences",
      screenshot: preferencesScreenshotZhCn,
      pins: [
        { marker: "1", left: 46.4, top: 73 },
        { marker: "2", left: 64.5, top: 88 },
      ],
    },
    { id: "prompt" },
    {
      id: "save",
      screenshot: workspacesScreenshotZhCn,
      pins: [
        { marker: "1", left: 91.6, top: 15.8 },
        { marker: "2", left: 81.1, top: 37.6 },
      ],
    },
  ],
  en: [
    {
      id: "prepare",
      screenshot: productsScreenshotEn,
      pins: [
        { marker: "1", left: 89.6, top: 15.8 },
        { marker: "2", left: 84.1, top: 37.6 },
      ],
    },
    {
      id: "ask",
      screenshot: commerceScreenshotEn,
      pins: [
        { marker: "1", left: 10, top: 11.5 },
        { marker: "2", left: 58.3, top: 48 },
        { marker: "3", left: 54.8, top: 54.8 },
      ],
    },
    {
      id: "preferences",
      screenshot: preferencesScreenshotEn,
      pins: [
        { marker: "1", left: 46.4, top: 73 },
        { marker: "2", left: 64.5, top: 88 },
      ],
    },
    { id: "prompt" },
    {
      id: "save",
      screenshot: workspacesScreenshotEn,
      pins: [
        { marker: "1", left: 89.8, top: 17.3 },
        { marker: "2", left: 81.6, top: 40.6 },
      ],
    },
  ],
}

export function getUsageGuideContent(
  t: TFunction,
  screenshotLanguage: UsageGuideScreenshotLanguage = "zh-CN"
) {
  const journeyCopy = t("agentCapabilities.final.journey.steps", {
    returnObjects: true,
  }) as JourneyStepCopy[]
  const journeyAssets = journeyAssetsByLanguage[screenshotLanguage]

  return {
    intro: {
      title: t("agentCapabilities.final.intro.title"),
      description: t("agentCapabilities.final.intro.description"),
      columns: t("agentCapabilities.final.intro.columns", {
        returnObjects: true,
      }) as {
        problem: string
        action: string
        result: string
      },
      items: t("agentCapabilities.final.intro.items", {
        returnObjects: true,
      }) as Array<{
        problem: string
        action: string
        result: string
      }>,
    },
    flow: {
      title: t("agentCapabilities.final.flow.title"),
      description: t("agentCapabilities.final.flow.description"),
      items: t("agentCapabilities.final.flow.items", {
        returnObjects: true,
      }) as Array<{
        stage: string
        title: string
        detail: string
      }>,
    },
    journey: {
      title: t("agentCapabilities.final.journey.title"),
      description: t("agentCapabilities.final.journey.description"),
      steps: journeyAssets.map((asset, index) => ({
        ...asset,
        ...journeyCopy[index],
      })) as UsageGuideJourneyStep[],
      recipe: t("agentCapabilities.final.journey.recipe", {
        returnObjects: true,
      }) as Array<{ title: string; detail: string }>,
      recipeTitle: t("agentCapabilities.final.journey.recipeTitle"),
      recipeDescription: t(
        "agentCapabilities.final.journey.recipeDescription"
      ),
      promptLabel: t("agentCapabilities.final.journey.promptLabel"),
      prompt: t("agentCapabilities.final.journey.prompt"),
      destinations: t("agentCapabilities.final.journey.destinations", {
        returnObjects: true,
      }) as Array<{ purpose: string; title: string; detail: string }>,
      destinationsTitle: t(
        "agentCapabilities.final.journey.destinationsTitle"
      ),
    },
    reference: {
      title: t("agentCapabilities.final.reference.title"),
      description: t("agentCapabilities.final.reference.description"),
      items: t("agentCapabilities.final.reference.items", {
        returnObjects: true,
      }) as Array<{ title: string; detail: string }>,
    },
    boundary: {
      title: t("agentCapabilities.final.boundary.title"),
      description: t("agentCapabilities.final.boundary.description"),
      supportedTitle: t("agentCapabilities.final.boundary.supportedTitle"),
      guardedTitle: t("agentCapabilities.final.boundary.guardedTitle"),
      supported: t("agentCapabilities.final.boundary.supported", {
        returnObjects: true,
      }) as string[],
      guarded: t("agentCapabilities.final.boundary.guarded", {
        returnObjects: true,
      }) as string[],
    },
  }
}
