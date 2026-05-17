import type { ReactNode } from "react"

type AnswerBlock =
  | { kind: "code"; code: string; language?: string }
  | { kind: "heading"; depth: number; text: string }
  | { kind: "list"; ordered: boolean; items: string[] }
  | { kind: "paragraph"; text: string }
  | { kind: "quote"; text: string }

const MAX_INLINE_HEADING_LENGTH = 28

export function AssistantAnswer({ content }: { content: string }) {
  const blocks = parseAnswerBlocks(content)

  return (
    <div className="space-y-4 text-sm/7 text-foreground">
      {blocks.map((block, index) => (
        <AnswerBlockView block={block} key={`${index}-${block.kind}`} />
      ))}
    </div>
  )
}

function AnswerBlockView({ block }: { block: AnswerBlock }) {
  if (block.kind === "heading") {
    const isPrimary = block.depth <= 2
    return (
      <div
        className={`min-w-0 ${isPrimary ? "pt-1" : ""}`}
      >
        <h3
          className={`wrap-break-word font-semibold tracking-normal ${
            isPrimary ? "text-base/7" : "text-sm/6"
          }`}
        >
          {renderInlineMarkdown(block.text)}
        </h3>
      </div>
    )
  }

  if (block.kind === "list") {
    const ListTag = block.ordered ? "ol" : "ul"
    return (
      <ListTag
        className={`space-y-2 pl-5 ${
          block.ordered ? "list-decimal" : "list-disc"
        }`}
      >
        {block.items.map((item, index) => (
          <li key={`${index}-${item}`} className="wrap-break-word pl-1">
            {renderInlineMarkdown(item)}
          </li>
        ))}
      </ListTag>
    )
  }

  if (block.kind === "quote") {
    return (
      <blockquote className="wrap-break-word border-l-2 border-border pl-3 text-muted-foreground">
        {renderInlineMarkdown(block.text)}
      </blockquote>
    )
  }

  if (block.kind === "code") {
    return (
      <pre className="max-w-full overflow-x-auto rounded-lg border border-border bg-muted p-3 text-xs/5">
        <code>{block.code}</code>
      </pre>
    )
  }

  return (
    <p className="wrap-break-word text-foreground">
      {renderInlineMarkdown(block.text)}
    </p>
  )
}

function parseAnswerBlocks(content: string): AnswerBlock[] {
  const normalized = content.replace(/\r\n/g, "\n").trim()
  if (!normalized) return []

  const blocks: AnswerBlock[] = []
  const paragraphLines: string[] = []
  let list: { ordered: boolean; items: string[] } | null = null
  let code: { language?: string; lines: string[] } | null = null

  const flushParagraph = () => {
    if (!paragraphLines.length) return
    const text = paragraphLines.join(" ").replace(/\s+/g, " ").trim()
    if (text) blocks.push({ kind: "paragraph", text })
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

  for (const rawLine of normalized.split("\n")) {
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

function readHeading(line: string): AnswerBlock | null {
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

  if (
    line.length <= MAX_INLINE_HEADING_LENGTH &&
    /^[\u4e00-\u9fa5A-Za-z0-9\s/·（）()]+[:：]$/.test(line)
  ) {
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

function renderInlineMarkdown(text: string): ReactNode[] {
  const nodes: ReactNode[] = []
  const pattern = /\*\*([^*]+)\*\*/g
  let lastIndex = 0
  let match: RegExpExecArray | null

  while ((match = pattern.exec(text)) !== null) {
    if (match.index > lastIndex) nodes.push(text.slice(lastIndex, match.index))
    nodes.push(
      <strong key={`${match.index}-${match[1]}`} className="font-semibold">
        {match[1]}
      </strong>
    )
    lastIndex = pattern.lastIndex
  }

  if (lastIndex < text.length) nodes.push(text.slice(lastIndex))
  return nodes.length ? nodes : [text]
}
