import type { TFunction } from "i18next"
import campaignCover from "./assets/usage-guide/campaign.jpg"
import commerceScreenshot from "./assets/usage-guide/commerce.png"
import dataCover from "./assets/usage-guide/data.jpg"
import preferencesScreenshot from "./assets/usage-guide/preferences.png"
import productCover from "./assets/usage-guide/product.jpg"
import productsScreenshot from "./assets/usage-guide/products.png"
import workspacesScreenshot from "./assets/usage-guide/workspaces.png"

interface TaskCopy {
  title: string
  shortDescription: string
  description: string
  result: string
}

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

const taskAssets = [
  { id: "campaign", cover: campaignCover },
  { id: "product", cover: productCover },
  { id: "data", cover: dataCover },
]

const journeyAssets = [
  {
    id: "prepare",
    screenshot: productsScreenshot,
    pins: [
      { marker: "1", left: 42, top: 36 },
      { marker: "2", left: 79, top: 44 },
    ],
  },
  {
    id: "ask",
    screenshot: commerceScreenshot,
    pins: [
      { marker: "1", left: 3, top: 9 },
      { marker: "2", left: 42, top: 59 },
      { marker: "3", left: 52, top: 59 },
    ],
  },
  {
    id: "preferences",
    screenshot: preferencesScreenshot,
    pins: [
      { marker: "1", left: 77, top: 58 },
      { marker: "2", left: 54, top: 73 },
    ],
  },
  { id: "prompt" },
  {
    id: "save",
    screenshot: workspacesScreenshot,
    pins: [
      { marker: "1", left: 85, top: 17 },
      { marker: "2", left: 78, top: 44 },
    ],
  },
]

export function getUsageGuideContent(t: TFunction) {
  const taskCopy = t("agentCapabilities.final.tasks.items", {
    returnObjects: true,
  }) as TaskCopy[]
  const journeyCopy = t("agentCapabilities.final.journey.steps", {
    returnObjects: true,
  }) as JourneyStepCopy[]

  return {
    intro: {
      title: t("agentCapabilities.final.intro.title"),
      description: t("agentCapabilities.final.intro.description"),
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
    tasks: {
      title: t("agentCapabilities.final.tasks.title"),
      description: t("agentCapabilities.final.tasks.description"),
      resultLabel: t("agentCapabilities.final.tasks.resultLabel"),
      items: taskAssets.map((asset, index) => ({
        ...asset,
        ...taskCopy[index],
      })),
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
