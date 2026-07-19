type FileEnumTranslate = (key: string) => string

export const fileKindFilterValues = [
  "all",
  "image",
  "pdf",
  "word",
  "sheet",
  "text",
  "other",
] as const

export const fileReadStatusFilterValues = [
  "all",
  "readable",
  "pending",
  "failed",
  "unsupported",
] as const

export const fileUsageFilterValues = [
  "all",
  "available",
  "attached",
  "selectedForTurn",
] as const

export function fileKindLabel(t: FileEnumTranslate, value: string) {
  return t(`commerce.files.filters.kind.${value}`)
}

export function fileReadStatusLabel(t: FileEnumTranslate, value: string) {
  return t(`commerce.files.status.${value}`)
}

export function fileUsageLabel(t: FileEnumTranslate, value: string) {
  return t(`commerce.files.detailDialog.usageState.${value}`)
}

export function fileKindFilterOptions(t: FileEnumTranslate) {
  return fileKindFilterValues.map((value) => ({
    label: fileKindLabel(t, value),
    value,
  }))
}

export function fileReadStatusFilterOptions(t: FileEnumTranslate) {
  return fileReadStatusFilterValues.map((value) => ({
    label: t(`commerce.files.filters.readStatus.${value}`),
    value,
  }))
}

export function fileUsageFilterOptions(t: FileEnumTranslate) {
  return fileUsageFilterValues.map((value) => ({
    label: t(`commerce.files.filters.usage.${value}`),
    value,
  }))
}
