import type { SupportedLanguage } from "../index"
import { localeResources } from "../resources"

export interface DateTimePickerCopy {
  placeholder: string
  clearLabel: string
  timeTitle: string
  timeAria: string
  now: string
  today: string
  yesterday: string
  weekStart: string
  monthStart: string
  yearStart: string
}

export function getDateTimePickerCopy(language: SupportedLanguage) {
  return localeResources[language].components.dateTimePicker
}
