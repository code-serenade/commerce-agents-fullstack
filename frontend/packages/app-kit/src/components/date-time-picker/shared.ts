import {
  normalizeLanguage,
  type SupportedLanguage as PickerLanguage,
} from "@workspace/locales"

export interface YearRange {
  from: number
  to: number
}

export function getDefaultYearRange(): YearRange {
  const currentYear = new Date().getFullYear()

  return {
    from: currentYear - 10,
    to: currentYear + 10,
  }
}

export function toIntlLocale(language: PickerLanguage) {
  return language === "zhCN" ? "zh-CN" : "en-US"
}

export function pad(value: number) {
  return String(value).padStart(2, "0")
}

export { normalizeLanguage }
export type { PickerLanguage }
