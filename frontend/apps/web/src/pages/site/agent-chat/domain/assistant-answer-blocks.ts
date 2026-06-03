import {
  normalizeAnswerLayout,
  readBracketHeadingText,
} from "./assistant-answer-layout"
import { isReadableBareHeading } from "./assistant-answer-heading"
import { readAnswerTable } from "./assistant-answer-table"

export type AssistantAnswerBlock =
  | { kind: "code"; code: string; language?: string }
  | { kind: "heading"; depth: number; text: string }
  | { kind: "list"; ordered: boolean; items: string[] }
  | { kind: "paragraph"; text: string }
  | { kind: "quote"; text: string }
  | { kind: "sources"; intro?: string; sources: AssistantAnswerSource[] }
  | { kind: "table"; header: string[]; rows: string[][] }

export type AssistantAnswerSource = {
  publishedAt?: string
  snippet?: string
  title: string
  url: string
}

const MAX_INLINE_HEADING_LENGTH = 28

export function parseAnswerBlocks(content: string): AssistantAnswerBlock[] {
  const normalized = normalizeAnswerLayout(content.replace(/\r\n/g, "\n")).trim()
  if (!normalized) return []

  const lines = normalized.split("\n")
  const blocks: AssistantAnswerBlock[] = []
  const paragraphLines: string[] = []
  let list: { ordered: boolean; items: string[] } | null = null
  let code: { language?: string; lines: string[] } | null = null

  const flushParagraph = () => {
    if (!paragraphLines.length) return
    const texts = shouldPreservePlainLineBreaks(paragraphLines)
      ? paragraphLines.map((line) => line.replace(/\s+/g, " ").trim())
      : [paragraphLines.join(" ").replace(/\s+/g, " ").trim()]

    for (const text of texts) {
      if (!text) continue
      const sourceList = readInlineSourceList(text)
      if (sourceList) {
        if (sourceList.before) {
          blocks.push({ kind: "paragraph", text: sourceList.before })
        }
        blocks.push({
          intro: sourceList.intro,
          kind: "sources",
          sources: sourceList.sources,
        })
      } else {
        blocks.push({ kind: "paragraph", text })
      }
    }
    paragraphLines.length = 0
  }

  const flushList = () => {
    if (!list) return
    if (list.items.length) blocks.push({ ...list, kind: "list" })
    list = null
  }

  const flushCode = () => {
    if (!code) return
    blocks.push({
      code: code.lines.join("\n"),
      kind: "code",
      language: code.language,
    })
    code = null
  }

  for (let index = 0; index < lines.length; index += 1) {
    const rawLine = lines[index] ?? ""
    const line = rawLine.trim()

    if (line.startsWith("```")) {
      if (code) {
        flushCode()
      } else {
        flushParagraph()
        flushList()
        code = { language: line.slice(3).trim() || undefined, lines: [] }
      }
      continue
    }

    if (code) {
      code.lines.push(rawLine)
      continue
    }

    const table = readAnswerTable(lines, index)
    if (table) {
      flushParagraph()
      flushList()
      blocks.push(table.block)
      index = table.nextIndex - 1
      continue
    }

    if (!line) {
      flushParagraph()
      flushList()
      continue
    }

    const heading = readHeading(line)
    if (heading) {
      flushParagraph()
      flushList()
      blocks.push(heading)
      continue
    }

    const quote = line.match(/^>\s+(.+)$/)
    if (quote) {
      flushParagraph()
      flushList()
      blocks.push({ kind: "quote", text: quote[1].trim() })
      continue
    }

    const listItem = readListItem(line)
    if (listItem) {
      flushParagraph()
      if (!list || list.ordered !== listItem.ordered) {
        flushList()
        list = { items: [], ordered: listItem.ordered }
      }
      list.items.push(listItem.text)
      continue
    }

    flushList()
    paragraphLines.push(line)
  }

  flushParagraph()
  flushList()
  flushCode()

  return blocks
}

function readInlineSourceList(text: string) {
  const marker = text.match(
    /实时搜索来源（[^）]+）：\s*\d+\s*个可采信来源/
  )
  if (!marker || marker.index === undefined) return null

  const markerEnd = marker.index + marker[0].length
  const before = text.slice(0, marker.index).trim()
  const rest = text.slice(markerEnd).trim()
  if (!rest.includes("http")) return null

  const sourceStartPattern =
    /(?:^|\s)([^|。！？\n]{2,120}?)\s*\|\s*([^|]{1,80})\s*\|\s*(https?:\/\/\S+)/g
  const matches = Array.from(rest.matchAll(sourceStartPattern))
  if (!matches.length) return null

  const sources: AssistantAnswerSource[] = []

  for (let index = 0; index < matches.length; index += 1) {
    const match = matches[index]
    const start = match.index ?? 0
    const nextStart = matches[index + 1]?.index ?? rest.length
    const segment = rest.slice(start, nextStart).trim()
    const parts = segment.split("|").map((part) => part.trim())
    const title = cleanupSourceTitle(parts[0] ?? "")
    const publishedAt = cleanupSourceDate(parts[1] ?? "")
    const url = cleanupSourceUrl(parts[2] ?? "")
    const snippet = parts.slice(3).join(" | ").trim()

    if (!title || !url.startsWith("http")) continue
    sources.push({
      publishedAt: publishedAt || undefined,
      snippet: snippet || undefined,
      title,
      url,
    })
  }

  if (!sources.length) return null

  return {
    before,
    intro: marker[0],
    sources,
  }
}

function cleanupSourceTitle(text: string) {
  return text.replace(/^[。！？\s]+/g, "").trim()
}

function cleanupSourceDate(text: string) {
  return text.replace(/^[：:\s]+/g, "").trim()
}

function cleanupSourceUrl(text: string) {
  return text.replace(/[，。；;、]+$/g, "").trim()
}

function readHeading(line: string): AssistantAnswerBlock | null {
  const markdownHeading = line.match(/^(#{1,4})\s+(.+)$/)
  if (markdownHeading) {
    return {
      depth: markdownHeading[1].length,
      kind: "heading",
      text: cleanupHeading(markdownHeading[2]),
    }
  }

  const strongHeading = line.match(/^\*\*(.+)\*\*[:：]?$/)
  if (strongHeading) {
    return { depth: 3, kind: "heading", text: cleanupHeading(strongHeading[1]) }
  }

  const bracketHeading = readBracketHeadingText(line)
  if (bracketHeading) {
    return { depth: 3, kind: "heading", text: cleanupHeading(bracketHeading) }
  }

  if (
    line.length <= MAX_INLINE_HEADING_LENGTH &&
    /^[\u4e00-\u9fa5A-Za-z0-9\s/·（）()]+[:：]$/.test(line)
  ) {
    return { depth: 3, kind: "heading", text: cleanupHeading(line) }
  }

  if (isReadableBareHeading(line)) {
    return { depth: 3, kind: "heading", text: cleanupHeading(line) }
  }

  return null
}

function readListItem(line: string) {
  const unordered = line.match(/^[-*]\s+(.+)$/)
  if (unordered) return { ordered: false, text: unordered[1].trim() }

  const ordered = line.match(/^\d+[.)、]\s*(.+)$/)
  if (ordered) return { ordered: true, text: ordered[1].trim() }

  return null
}

function cleanupHeading(text: string) {
  return text.replace(/[*#：:]+$/g, "").trim()
}

function shouldPreservePlainLineBreaks(lines: string[]) {
  if (lines.length < 3) return false
  const joined = lines.join(" ")
  if (joined.includes("http") || readInlineSourceList(joined)) return false
  return lines.some((line) => Array.from(line.trim()).length <= 48)
}
