import type { SupportedLanguage } from "../index"
import { localeResources } from "../resources"

export interface CopyableTextCopy {
  copyLabel: string
  copiedLabel: string
}

export function getCopyableTextCopy(language: SupportedLanguage) {
  return localeResources[language].components.copyableText
}
