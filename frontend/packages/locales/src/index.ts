export type SupportedLanguage = "en" | "zhCN"

export const supportedLanguages = ["en", "zhCN"] as const
export const localeNamespaces = [
  "common",
  "messages",
  "pages",
  "components",
  "errors",
] as const
export const defaultLocaleNamespace = "messages" as const

export function normalizeLanguage(
  language: string | undefined
): SupportedLanguage {
  const normalized = language?.toLowerCase()

  if (normalized === "zhcn" || normalized === "zh-cn" || normalized === "zh") {
    return "zhCN"
  }

  return "en"
}

export { localeResources } from "./resources"
export type { LocaleNamespace, LocaleResources } from "./resources"
export * from "./components"
