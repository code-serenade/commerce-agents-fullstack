import type { AssistantAnswerBlock } from "./assistant-answer-blocks"

type AssistantAnswerTableBlock = Extract<AssistantAnswerBlock, { kind: "table" }>

export function readAnswerTable(lines: string[], startIndex: number) {
  return (
    readMarkdownTable(lines, startIndex) ??
    readTabSeparatedTable(lines, startIndex)
  )
}

function readMarkdownTable(lines: string[], startIndex: number) {
  const headerLine = lines[startIndex]?.trim() ?? ""
  const separatorLine = lines[startIndex + 1]?.trim() ?? ""
  if (!isTableRow(headerLine) || !isTableSeparator(separatorLine)) return null

  const header = splitTableCells(headerLine)
  const rows: string[][] = []
  let nextIndex = startIndex + 2

  while (nextIndex < lines.length) {
    const line = lines[nextIndex]?.trim() ?? ""
    if (!isTableRow(line) || isTableSeparator(line)) break
    const row = splitTableCells(line)
    rows.push(normalizeTableRow(row, header.length))
    nextIndex += 1
  }

  return buildTableResult(header, rows, nextIndex)
}

function readTabSeparatedTable(lines: string[], startIndex: number) {
  const headerLine = lines[startIndex]?.trim() ?? ""
  if (!isTabSeparatedTableRow(headerLine)) return null

  const header = splitTabSeparatedCells(headerLine)
  const rows: string[][] = []
  let nextIndex = startIndex + 1

  while (nextIndex < lines.length) {
    const line = lines[nextIndex]?.trim() ?? ""
    if (!isTabSeparatedTableRow(line)) break
    const row = splitTabSeparatedCells(line)
    rows.push(normalizeTableRow(row, header.length))
    nextIndex += 1
  }

  return buildTableResult(header, rows, nextIndex)
}

function buildTableResult(
  header: string[],
  rows: string[][],
  nextIndex: number
): { block: AssistantAnswerTableBlock; nextIndex: number } | null {
  if (header.length === 0 || rows.length === 0) return null
  return {
    block: {
      header,
      kind: "table",
      rows,
    },
    nextIndex,
  }
}

function isTableRow(line: string) {
  return line.includes("|") && splitTableCells(line).length >= 2
}

function isTableSeparator(line: string) {
  const cells = splitTableCells(line)
  return (
    cells.length >= 2 &&
    cells.every((cell) => /^:?-{3,}:?$/.test(cell.replace(/\s+/g, "")))
  )
}

function splitTableCells(line: string) {
  const trimmed = line.trim().replace(/^\|/, "").replace(/\|$/, "")
  return trimmed.split("|").map((cell) => cell.trim())
}

function isTabSeparatedTableRow(line: string) {
  return (
    line.includes("\t") &&
    splitTabSeparatedCells(line).filter(Boolean).length >= 2
  )
}

function splitTabSeparatedCells(line: string) {
  return line
    .trim()
    .split(/\t+/)
    .map((cell) => cell.trim())
}

function normalizeTableRow(row: string[], cellCount: number) {
  if (row.length === cellCount) return row
  if (row.length > cellCount) return row.slice(0, cellCount)
  return [...row, ...Array.from({ length: cellCount - row.length }, () => "")]
}
