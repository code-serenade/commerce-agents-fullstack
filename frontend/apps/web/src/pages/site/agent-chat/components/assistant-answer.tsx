import type { ReactNode } from "react"
import { useTranslation } from "react-i18next"
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
  const { t } = useTranslation()
  const blocks = parseAnswerBlocks(content)
  const sectionLabels = {
    action: t("agentChat.answer.sectionLabels.action"),
    final: t("agentChat.answer.sectionLabels.final"),
    team: t("agentChat.answer.sectionLabels.team"),
  }

  return (
    <div className="space-y-4 text-sm/7 text-foreground">
      {blocks.map((block, index) => (
        <AnswerBlockView
          block={block}
          key={`${index}-${block.kind}`}
          sectionLabels={sectionLabels}
        />
      ))}
    </div>
  )
}

type SectionLabels = {
  action: string
  final: string
  team: string
}

function AnswerBlockView({
  block,
  sectionLabels,
}: {
  block: AssistantAnswerBlock
  sectionLabels: SectionLabels
}) {
  if (block.kind === "heading") {
    return <HeadingBlock block={block} sectionLabels={sectionLabels} />
  }
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
  sectionLabels,
}: {
  block: Extract<AssistantAnswerBlock, { kind: "heading" }>
  sectionLabels: SectionLabels
}) {
  const isPrimary = block.depth <= 2
  const sectionTone = resolveSectionTone(block.text)
  const headingLabel = resolveSectionLabel(block.text, sectionLabels)

  return (
    <div
      className={`min-w-0 ${
        isPrimary ? "border-t border-border pt-4 first:border-t-0 first:pt-1" : ""
      } ${sectionTone}`}
    >
      <h3
        className={`wrap-break-word flex min-w-0 items-center gap-2 font-semibold tracking-normal ${
          isPrimary ? "text-base/7" : "text-sm/6"
        }`}
      >
        {headingLabel ? (
          <span className="shrink-0 rounded-full border border-border bg-background px-2 py-0.5 text-[11px] font-medium text-muted-foreground">
            {headingLabel}
          </span>
        ) : null}
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
    <div className="max-w-full overflow-x-auto rounded-lg border border-foreground/40 bg-background shadow-sm ring-1 ring-foreground/15 [&_table]:border-collapse [&_tbody_tr:nth-child(even)]:bg-muted/30 [&_td]:border [&_td]:border-foreground/20 [&_td]:px-3 [&_td]:py-2.5 [&_td]:align-top [&_th]:border [&_th]:border-foreground/25 [&_th]:bg-muted [&_th]:px-3 [&_th]:py-2.5 [&_th]:font-semibold [&_th]:text-foreground [&_thead]:border-b [&_thead]:border-foreground/30">
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
  if (/^(综合回答|综合结论|直接结论|最终建议|多智能体协作结论)/.test(text)) {
    return "rounded-lg border border-primary/25 bg-primary/5 px-3 py-2 text-primary"
  }
  if (/^(各岗位独立输出|虚拟员工分工|协作回答|岗位协作|各岗位|分工)/.test(text)) {
    return "rounded-lg border border-border bg-muted/35 px-3 py-2 text-foreground"
  }
  if (/^(主岗汇总方案|主岗综合方案|团队执行稿|丰富执行方案)/.test(text)) {
    return "rounded-lg border border-foreground/15 bg-card px-3 py-2"
  }
  if (/^(教学讲解|执行清单|下一步|交付物|可沉淀交付物)/.test(text)) {
    return "border-l-2 border-primary/40 pl-3"
  }
  return ""
}

function resolveSectionLabel(text: string, labels: SectionLabels) {
  if (/^(综合回答|综合结论|直接结论|多智能体协作结论)/.test(text)) {
    return labels.final
  }
  if (/^(各岗位独立输出|虚拟员工分工|岗位协作|协作回答|各岗位)/.test(text)) {
    return labels.team
  }
  if (/^(主岗汇总方案|主岗综合方案|丰富执行方案|执行清单|下一步|团队执行稿|交付物|可沉淀交付物)/.test(text)) {
    return labels.action
  }
  if (/^(教学讲解)/.test(text)) return labels.team
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
