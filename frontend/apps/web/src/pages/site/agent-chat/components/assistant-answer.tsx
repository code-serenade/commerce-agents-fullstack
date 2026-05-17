import type { ReactNode } from "react"
import {
  Table,
  TableBody,
  TableCell,
  TableHead,
  TableHeader,
  TableRow,
} from "@workspace/ui-components/stable/table"
import {
  parseAnswerBlocks,
  type AssistantAnswerBlock,
} from "../domain/assistant-answer-blocks"

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

function AnswerBlockView({ block }: { block: AssistantAnswerBlock }) {
  if (block.kind === "heading") return <HeadingBlock block={block} />
  if (block.kind === "list") return <ListBlock block={block} />
  if (block.kind === "quote") return <QuoteBlock text={block.text} />
  if (block.kind === "code") return <CodeBlock code={block.code} />
  if (block.kind === "table") return <TableBlock block={block} />

  return (
    <p className="wrap-break-word text-foreground">
      {renderInlineMarkdown(block.text)}
    </p>
  )
}

function HeadingBlock({
  block,
}: {
  block: Extract<AssistantAnswerBlock, { kind: "heading" }>
}) {
  const isPrimary = block.depth <= 2
  const sectionTone = resolveSectionTone(block.text)

  return (
    <div
      className={`min-w-0 ${
        isPrimary ? "border-t border-border pt-4 first:border-t-0 first:pt-1" : ""
      } ${sectionTone}`}
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

function ListBlock({
  block,
}: {
  block: Extract<AssistantAnswerBlock, { kind: "list" }>
}) {
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

function QuoteBlock({ text }: { text: string }) {
  return (
    <blockquote className="wrap-break-word border-l-2 border-border pl-3 text-muted-foreground">
      {renderInlineMarkdown(text)}
    </blockquote>
  )
}

function CodeBlock({ code }: { code: string }) {
  return (
    <pre className="max-w-full overflow-x-auto rounded-lg border border-border bg-muted p-3 text-xs/5">
      <code>{code}</code>
    </pre>
  )
}

function TableBlock({
  block,
}: {
  block: Extract<AssistantAnswerBlock, { kind: "table" }>
}) {
  return (
    <div className="max-w-full overflow-hidden rounded-lg border border-border">
      <Table>
        <TableHeader>
          <TableRow>
            {block.header.map((cell, index) => (
              <TableHead key={`${index}-${cell}`}>
                {renderInlineMarkdown(cell)}
              </TableHead>
            ))}
          </TableRow>
        </TableHeader>
        <TableBody>
          {block.rows.map((row, rowIndex) => (
            <TableRow key={rowIndex}>
              {row.map((cell, cellIndex) => (
                <TableCell key={`${rowIndex}-${cellIndex}`}>
                  <span className="wrap-break-word whitespace-normal">
                    {renderInlineMarkdown(cell)}
                  </span>
                </TableCell>
              ))}
            </TableRow>
          ))}
        </TableBody>
      </Table>
    </div>
  )
}

function resolveSectionTone(text: string) {
  if (/^(主回答|直接结论|最终建议|多智能体协作结论)/.test(text)) {
    return "rounded-md bg-muted/40 px-3 py-2"
  }
  if (/^(虚拟员工分工|协作回答|岗位协作|各岗位|分工)/.test(text)) {
    return "border-l-2 border-border pl-3"
  }
  return ""
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
