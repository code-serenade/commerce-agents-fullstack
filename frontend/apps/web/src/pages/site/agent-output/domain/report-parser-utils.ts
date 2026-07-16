export type UnknownRecord = Record<string, unknown>

export function collectionEntries(value: unknown): Array<[string, unknown]> {
  if (Array.isArray(value)) {
    return value.map((item, index) => [String(index + 1), item])
  }
  const record = asRecord(value)
  return record ? Object.entries(record) : []
}

export function readStringList(value: unknown): string[] | null {
  if (Array.isArray(value)) {
    const values = value.map(textValue).filter(Boolean)
    return values.length ? values : null
  }
  const record = asRecord(value)
  if (record) {
    const values = Object.values(record).map(textValue).filter(Boolean)
    return values.length ? values : null
  }
  const text = textValue(value)
  return text ? [text] : null
}

export function readField(record: UnknownRecord | null, ...keys: string[]) {
  if (!record) return undefined
  for (const key of keys) {
    if (record[key] != null) return record[key]
  }
  return undefined
}

export function readText(record: UnknownRecord | null, ...keys: string[]) {
  return textValue(readField(record, ...keys))
}

export function textValue(value: unknown): string {
  if (typeof value === "string" || typeof value === "number") {
    return String(value).trim()
  }
  return ""
}

export function firstText(...values: Array<string | null | undefined>) {
  return values.find((value) => value?.trim())?.trim() ?? ""
}

export function firstTextOrNull(...values: Array<string | null | undefined>) {
  return firstText(...values) || null
}

export function asRecord(value: unknown): UnknownRecord | null {
  return value && typeof value === "object" && !Array.isArray(value)
    ? (value as UnknownRecord)
    : null
}

export function mergeRecords(...values: unknown[]): UnknownRecord {
  return Object.assign({}, ...values.map(asRecord).filter(Boolean))
}

export function parseJsonRecord(content: string): UnknownRecord | null {
  const candidate =
    content.match(/```(?:json)?\s*([\s\S]*?)```/i)?.[1] ?? content
  try {
    return asRecord(JSON.parse(candidate))
  } catch {
    return null
  }
}

export function cleanInline(value: string) {
  return value
    .replace(/^\|+|\|+$/g, "")
    .replace(/[*_`~]/g, "")
    .trim()
}

export function safeId(value: string) {
  return value
    .toLowerCase()
    .replace(/[^a-z0-9\u4e00-\u9fff]+/g, "-")
    .replace(/^-|-$/g, "")
}

export function safeFileName(value: string) {
  return (
    value.replace(/[\\/:*?"<>|]/g, "-").slice(0, 80) || "conversation-report"
  )
}

export function humanizeKey(value: string) {
  return value
    .replace(/[_-]+/g, " ")
    .replace(/([a-z])([A-Z])/g, "$1 $2")
    .trim()
}

export function uniqueTexts(values: Array<string | null | undefined>) {
  return [
    ...new Set(
      values
        .map((value) => value?.trim())
        .filter((value): value is string => Boolean(value))
    ),
  ]
}
