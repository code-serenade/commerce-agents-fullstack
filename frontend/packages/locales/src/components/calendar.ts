import type { SupportedLanguage } from "../index"
import { localeResources } from "../resources"

export interface CalendarCopy {
  previousYearLabel: string
  previousMonthLabel: string
  nextMonthLabel: string
  nextYearLabel: string
}

export function getCalendarCopy(language: SupportedLanguage) {
  return localeResources[language].components.calendar
}
