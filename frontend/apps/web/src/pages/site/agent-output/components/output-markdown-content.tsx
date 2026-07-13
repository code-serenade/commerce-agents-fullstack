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
} from "@/pages/site/agent-chat/domain/assistant-answer-blocks"
import { renderInlineMarkdown } from "@/pages/site/agent-chat/components/assistant-inline-markdown"

export function OutputMarkdownContent({ content }: { content: string }) {
  const blocks = parseAnswerBlocks(content)

  if (!blocks.length) return null

  return (
    <div className="space-y-4 p-5 text-sm/7 text-foreground">
      {blocks.map((block, index) => (
        <OutputMarkdownBlock block={block} key={`${index}-${block.kind}`} />
      ))}
    </div>
  )
}

function OutputMarkdownBlock({ block }: { block: AssistantAnswerBlock }) {
  if (block.kind === "heading") return <HeadingBlock block={block} />
  if (block.kind === "list") return <ListBlock block={block} />
  if (block.kind === "quote") return <QuoteBlock text={block.text} />
  if (block.kind === "code") return <CodeBlock code={block.code} />
  if (block.kind === "table") return <TableBlock block={block} />
  if (block.kind === "sources") {
    return (
      <div className="space-y-2 rounded-lg border border-(--document-card-border) bg-muted/25 p-3">
        {block.intro ? (
          <p className="text-sm font-medium wrap-break-word">
            {renderInlineMarkdown(block.intro)}
          </p>
        ) : null}
        <ul className="space-y-2">
          {block.sources.map((source) => (
            <li key={source.url} className="text-sm wrap-break-word">
              <a
                className="font-medium text-primary underline-offset-4 hover:underline"
                href={source.url}
                rel="noreferrer"
                target="_blank"
              >
                {source.title}
              </a>
              {source.publishedAt ? (
                <span className="text-muted-foreground">
                  {" "}
                  / {source.publishedAt}
                </span>
              ) : null}
              {source.snippet ? (
                <p className="mt-1 text-muted-foreground">
                  {renderInlineMarkdown(source.snippet)}
                </p>
              ) : null}
            </li>
          ))}
        </ul>
      </div>
    )
  }

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
  const headingClassName = resolveHeadingClassName(block.depth)
  const className = `wrap-break-word tracking-normal text-foreground ${headingClassName}`

  if (block.depth <= 1) {
    return <h1 className={className}>{renderInlineMarkdown(block.text)}</h1>
  }
  if (block.depth === 2) {
    return <h2 className={className}>{renderInlineMarkdown(block.text)}</h2>
  }
  if (block.depth === 3) {
    return <h3 className={className}>{renderInlineMarkdown(block.text)}</h3>
  }

  return <h4 className={className}>{renderInlineMarkdown(block.text)}</h4>
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
        <li key={`${index}-${item}`} className="pl-1 wrap-break-word">
          {renderInlineMarkdown(item)}
        </li>
      ))}
    </ListTag>
  )
}

function QuoteBlock({ text }: { text: string }) {
  return (
    <blockquote className="border-l-2 border-primary/35 bg-muted/25 py-2 pr-2 pl-3 wrap-break-word text-muted-foreground">
      {renderInlineMarkdown(text)}
    </blockquote>
  )
}

function CodeBlock({ code }: { code: string }) {
  return (
    <pre className="max-w-full overflow-x-auto rounded-lg border border-(--document-card-border) bg-muted/50 p-3 text-xs/5 text-foreground">
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
    <div className="max-w-full overflow-x-auto rounded-lg border border-(--document-card-border) bg-background shadow-sm [&_table]:border-separate [&_table]:border-spacing-0 [&_tbody_tr:last-child_td]:border-b-0 [&_tbody_tr:nth-child(even)]:bg-muted/25 [&_td]:border-r [&_td]:border-b [&_td]:border-(--document-card-border) [&_td]:px-3 [&_td]:py-2.5 [&_td]:align-top [&_td]:text-sm/7 [&_td:last-child]:border-r-0 [&_th]:border-r [&_th]:border-b [&_th]:border-(--document-card-border) [&_th]:bg-muted/45 [&_th]:px-3 [&_th]:py-2.5 [&_th]:text-left [&_th]:text-sm/6 [&_th]:font-semibold [&_th]:text-foreground [&_th:last-child]:border-r-0">
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

function resolveHeadingClassName(depth: number) {
  if (depth <= 1) {
    return "border-b border-(--document-card-border) pb-3 text-2xl/tight font-semibold"
  }
  if (depth === 2) return "pt-2 text-lg/7 font-semibold"
  if (depth === 3) return "text-base/7 font-semibold"
  return "text-sm/6 font-semibold text-muted-foreground"
}
