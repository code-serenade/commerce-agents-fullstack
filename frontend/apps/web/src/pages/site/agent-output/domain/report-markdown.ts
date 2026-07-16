import type {
  AgentOutputMetric,
  AgentOutputSection,
  AgentOutputStep,
} from "../types"
import { cleanInline, safeId } from "./report-parser-utils"

export function parseMarkdownReport(content: string) {
  const lines = content.split(/\r?\n/)
  let title = ""
  let current: { title: string; lines: string[] } | null = null
  const rawSections: Array<{ title: string; lines: string[] }> = []
  const intro: string[] = []

  for (const rawLine of lines) {
    const line = rawLine.trim()
    const heading = /^(#{1,4})\s+(.+)$/.exec(line)
    if (heading) {
      if (heading[1].length === 1 && !title) {
        title = cleanInline(heading[2])
        continue
      }
      if (current) rawSections.push(current)
      current = { title: cleanInline(heading[2]), lines: [] }
      continue
    }
    if (current) current.lines.push(line)
    else intro.push(line)
  }
  if (current) rawSections.push(current)

  const sections = rawSections
    .map(makeMarkdownSection)
    .filter((section) => section.content || section.bullets.length)
  const actionsSection = rawSections.find((section) =>
    /行动|下一步|待办|action|next|todo/i.test(section.title)
  )

  return {
    actions: readBullets(actionsSection?.lines ?? []),
    description: cleanInline(
      intro.filter((line) => line && !line.includes("|")).join(" ")
    ),
    metrics: parseMarkdownMetrics(lines),
    sections,
    steps: parseMarkdownSteps(lines),
    title,
  }
}

function makeMarkdownSection(
  section: { title: string; lines: string[] },
  index: number
): AgentOutputSection {
  const paragraphs = section.lines.filter(
    (line) =>
      line &&
      !/^[-*+]\s+/.test(line) &&
      !/^\d+[.)、]\s+/.test(line) &&
      !line.includes("|")
  )
  return {
    id: `${safeId(section.title) || "section"}-${index + 1}`,
    title: section.title,
    eyebrow: "",
    content: cleanInline(paragraphs.join(" ")),
    bullets: readBullets(section.lines),
  }
}

function readBullets(lines: string[]) {
  return lines
    .map((line) => /^[-*+]\s+(.+)$/.exec(line)?.[1])
    .filter((value): value is string => Boolean(value))
    .map(cleanInline)
}

function parseMarkdownSteps(lines: string[]): AgentOutputStep[] {
  return lines.flatMap((line) => {
    const match =
      /^\s*\d+[.)、]\s*(?:\*\*)?([^:：*]+)(?:\*\*)?[:：]?\s*(.*)$/.exec(line)
    return match
      ? [
          {
            title: cleanInline(match[1]),
            owner: "",
            timing: "",
            detail: cleanInline(match[2]),
          },
        ]
      : []
  })
}

function parseMarkdownMetrics(lines: string[]): AgentOutputMetric[] {
  const metrics: AgentOutputMetric[] = []
  for (let index = 0; index < lines.length - 1; index += 1) {
    if (
      !lines[index].includes("|") ||
      !/^\s*\|?\s*:?-{3,}/.test(lines[index + 1])
    ) {
      continue
    }
    for (
      let row = index + 2;
      row < lines.length && lines[row].includes("|");
      row += 1
    ) {
      const cells = lines[row].split("|").map(cleanInline).filter(Boolean)
      if (cells.length >= 2) {
        metrics.push({
          label: cells[0],
          value: cells[1],
          description: cells.slice(2).join(" / "),
        })
      }
    }
    break
  }
  return metrics.slice(0, 4)
}
