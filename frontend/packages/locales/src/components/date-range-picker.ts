import type { SupportedLanguage } from "../index"
import { localeResources } from "../resources"

export interface DateRangePickerCopy {
  placeholder: string
  endDate: string
  clearLabel: string
}

export function getDateRangePickerCopy(language: SupportedLanguage) {
  return localeResources[language].components.dateRangePicker
}
