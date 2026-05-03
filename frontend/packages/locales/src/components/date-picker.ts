import type { SupportedLanguage } from "../index"
import { localeResources } from "../resources"

export interface DatePickerCopy {
  placeholder: string
  clearLabel: string
}

export function getDatePickerCopy(language: SupportedLanguage) {
  return localeResources[language].components.datePicker
}
