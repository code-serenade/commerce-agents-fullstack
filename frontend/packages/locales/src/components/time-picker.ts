import type { SupportedLanguage } from "../index"
import { localeResources } from "../resources"

export interface TimePickerCopy {
  placeholder: string
  clearLabel: string
  ariaLabel: string
}

export function getTimePickerCopy(language: SupportedLanguage) {
  return localeResources[language].components.timePicker
}
