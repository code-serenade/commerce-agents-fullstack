import type {
  AgentOutputMetric,
  AgentOutputSection,
  AgentOutputStep,
} from "../types"
import {
  asRecord,
  collectionEntries,
  firstText,
  humanizeKey,
  readField,
  readStringList,
  readText,
  safeId,
  textValue,
  type UnknownRecord,
} from "./report-parser-utils"

export function findStructuredRecord(
  ...records: Array<UnknownRecord | null>
): UnknownRecord {
  for (const record of records) {
    if (!record) continue
    for (const key of [
      "structuredReport",
      "structured_report",
      "report",
      "presentation",
    ]) {
      const nested = asRecord(record[key])
      if (nested) return nested
    }
    if (
      ["metrics", "steps", "sections", "actions", "summary"].some(
        (key) => key in record
      )
    ) {
      return record
    }
  }
  return {}
}

export function readStructuredReport(record: UnknownRecord) {
  return {
    actions: readStringList(
      readField(record, "actions", "nextActions", "todos")
    ),
    badges: [
      ...(readStringList(readField(record, "badges", "tags")) ?? []),
      readText(record, "category", "businessCategory"),
      readText(record, "platform", "channel"),
    ],
    collaboration: readText(
      record,
      "collaboration",
      "dispatch",
      "agents",
      "agentRouting"
    ),
    description: readText(
      record,
      "description",
      "summary",
      "overview",
      "abstract"
    ),
    metrics: readMetrics(record),
    prompt: readText(record, "prompt", "userPrompt", "question", "input"),
    sections: readSections(record),
    steps: readSteps(record),
    title: readText(record, "title", "name", "subject"),
  }
}

function readMetrics(record: UnknownRecord): AgentOutputMetric[] | null {
  const entries = collectionEntries(
    readField(record, "metrics", "indicators", "kpis")
  )
  if (!entries.length) return null

  return entries.flatMap(([key, item]) => {
    const itemRecord = asRecord(item)
    const value = itemRecord
      ? readText(itemRecord, "value", "target", "amount")
      : textValue(item)
    if (!value) return []
    return [
      {
        label: firstText(
          readText(itemRecord, "label", "title", "name"),
          humanizeKey(key)
        ),
        value,
        description: readText(itemRecord, "description", "detail", "note"),
      },
    ]
  })
}

function readSteps(record: UnknownRecord): AgentOutputStep[] | null {
  const entries = collectionEntries(
    readField(record, "steps", "plan", "workflow")
  )
  if (!entries.length) return null

  return entries.flatMap(([key, item]) => {
    const itemRecord = asRecord(item)
    if (!itemRecord) {
      const detail = textValue(item)
      return detail
        ? [{ title: humanizeKey(key), owner: "", timing: "", detail }]
        : []
    }
    const title = firstText(
      readText(itemRecord, "title", "name"),
      humanizeKey(key)
    )
    const detail = readText(itemRecord, "detail", "description", "content")
    if (!title && !detail) return []
    return [
      {
        title,
        owner: readText(itemRecord, "owner", "role", "agent"),
        timing: readText(itemRecord, "timing", "time", "deadline", "period"),
        detail,
      },
    ]
  })
}

function readSections(record: UnknownRecord): AgentOutputSection[] | null {
  const entries = collectionEntries(
    readField(record, "sections", "chapters", "details")
  )
  if (!entries.length) return null

  const sections = entries.flatMap(([key, item], index) => {
    const itemRecord = asRecord(item)
    if (!itemRecord) {
      const content = textValue(item)
      return content
        ? [makeSection(key, humanizeKey(key), "", content, [], index)]
        : []
    }
    const title = firstText(
      readText(itemRecord, "title", "name"),
      humanizeKey(key)
    )
    const content = readText(
      itemRecord,
      "content",
      "description",
      "summary",
      "text"
    )
    const bullets =
      readStringList(readField(itemRecord, "bullets", "items", "points")) ?? []
    if (!title && !content && !bullets.length) return []
    return [
      makeSection(
        key,
        title,
        readText(itemRecord, "eyebrow", "subtitle", "label"),
        content,
        bullets,
        index
      ),
    ]
  })
  return sections.length ? sections : null
}

function makeSection(
  key: string,
  title: string,
  eyebrow: string,
  content: string,
  bullets: string[],
  index: number
): AgentOutputSection {
  return {
    id: `${safeId(key) || "section"}-${index + 1}`,
    title,
    eyebrow,
    content,
    bullets,
  }
}
