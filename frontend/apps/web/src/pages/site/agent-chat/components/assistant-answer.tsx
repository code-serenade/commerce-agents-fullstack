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
import { renderInlineMarkdown } from "./assistant-inline-markdown"
import { SourcesBlock } from "./assistant-source-block"

export function AssistantAnswer({ content }: { content: string }) {
  const { t } = useTranslation()
  const blocks = parseAnswerBlocks(content)
  const sectionLabels = {
    action: t("agentChat.answer.sectionLabels.action"),
    direct: t("agentChat.answer.sectionLabels.direct"),
    final: t("agentChat.answer.sectionLabels.final"),
    reason: t("agentChat.answer.sectionLabels.reason"),
    team: t("agentChat.answer.sectionLabels.team"),
  }
  const safeHeadings = {
    action: t("agentChat.answer.safeHeadings.action"),
    explanation: t("agentChat.answer.safeHeadings.explanation"),
    final: t("agentChat.answer.safeHeadings.final"),
    references: t("agentChat.answer.safeHeadings.references"),
  }

  return (
    <div className="space-y-4 text-sm/7 text-foreground">
      {blocks.map((block, index) => (
        <AnswerBlockView
          block={block}
          key={`${index}-${block.kind}`}
          safeHeadings={safeHeadings}
          sectionLabels={sectionLabels}
        />
      ))}
    </div>
  )
}

type SectionLabels = {
  action: string
  direct: string
  final: string
  reason: string
  team: string
}

type SafeHeadings = {
  action: string
  explanation: string
  final: string
  references: string
}

function AnswerBlockView({
  block,
  safeHeadings,
  sectionLabels,
}: {
  block: AssistantAnswerBlock
  safeHeadings: SafeHeadings
  sectionLabels: SectionLabels
}) {
  if (block.kind === "heading") {
    return (
      <HeadingBlock
        block={block}
        safeHeadings={safeHeadings}
        sectionLabels={sectionLabels}
      />
    )
  }
  if (block.kind === "list") return <ListBlock block={block} />
  if (block.kind === "quote") return <QuoteBlock text={block.text} />
  if (block.kind === "code") return <CodeBlock code={block.code} />
  if (block.kind === "sources") return <SourcesBlock block={block} />
  if (block.kind === "table") return <TableBlock block={block} />

  return (
    <p className="wrap-break-word text-foreground">
      {renderInlineMarkdown(block.text)}
    </p>
  )
}

function HeadingBlock({
  block,
  safeHeadings,
  sectionLabels,
}: {
  block: Extract<AssistantAnswerBlock, { kind: "heading" }>
  safeHeadings: SafeHeadings
  sectionLabels: SectionLabels
}) {
  const isPrimary = block.depth <= 2
  const sectionTone = resolveSectionTone(block.text)
  const headingLabel = resolveSectionLabel(block.text, sectionLabels)
  const displayHeading = resolveDisplayHeading(block.text, safeHeadings)

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
          <span className="shrink-0 rounded-full border border-(--agent-chat-soft-border) bg-(--agent-chat-elevated-strong) px-2 py-0.5 text-[11px] font-medium text-(--agent-chat-accent-foreground)">
            {headingLabel}
          </span>
        ) : null}
        {renderInlineMarkdown(displayHeading)}
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
    <div className="max-w-full overflow-x-auto rounded-2xl border border-(--agent-chat-soft-border) bg-(--agent-chat-elevated-strong) shadow-(--agent-chat-card-shadow) backdrop-blur-xl [&_table]:border-separate [&_table]:border-spacing-0 [&_tbody_tr:last-child_td]:border-b-0 [&_tbody_tr:nth-child(even)]:bg-(--agent-chat-subtle-surface) [&_td]:border-r [&_td]:border-b [&_td]:border-(--agent-chat-soft-border) [&_td]:px-4 [&_td]:py-3.5 [&_td]:align-top [&_td]:text-sm/7 [&_td:last-child]:border-r-0 [&_th]:border-r [&_th]:border-b [&_th]:border-(--agent-chat-soft-border) [&_th]:bg-(--agent-chat-accent-soft) [&_th]:px-4 [&_th]:py-3 [&_th]:text-left [&_th]:text-sm/6 [&_th]:font-semibold [&_th]:text-(--agent-chat-accent-foreground) [&_th:first-child]:rounded-tl-2xl [&_th:last-child]:rounded-tr-2xl [&_th:last-child]:border-r-0">
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
  if (/^(结论|结论摘要|综合回答|综合结论|直接结论|最终建议|多智能体协作结论)/.test(text)) {
    return "rounded-lg border border-primary/25 bg-primary/5 px-3 py-2 text-primary"
  }
  if (/^(各岗位独立输出|虚拟员工分工|协作回答|岗位协作|各岗位|分工)/.test(text)) {
    return "rounded-lg border border-border bg-muted/35 px-3 py-2 text-foreground"
  }
  if (/^(可直接使用内容|可直接使用|直接可用|交付内容|主岗汇总方案|主岗综合方案|团队执行稿|丰富执行方案)/.test(text)) {
    return "rounded-xl border border-(--agent-chat-soft-border) bg-(--agent-chat-subtle-surface) px-3 py-2 shadow-none [&_h3]:border-l-2 [&_h3]:border-primary/50 [&_h3]:pl-2"
  }
  if (/^(简短原因|原因|为什么|教学讲解|执行清单|下一步|交付物|可沉淀交付物)/.test(text)) {
    return "border-l-2 border-primary/40 pl-3"
  }
  return ""
}

function resolveSectionLabel(text: string, labels: SectionLabels) {
  if (/^(结论|结论摘要|综合回答|综合结论|直接结论|多智能体协作结论)/.test(text)) {
    return labels.final
  }
  if (/^(可直接使用内容|可直接使用|直接可用|交付内容)/.test(text)) {
    return labels.direct
  }
  if (/^(简短原因|原因|为什么)/.test(text)) return labels.reason
  if (/^(各岗位独立输出|虚拟员工分工|岗位协作|协作回答|各岗位)/.test(text)) {
    return labels.team
  }
  if (/^(主岗汇总方案|主岗综合方案|丰富执行方案|执行清单|下一步|团队执行稿|交付物|可沉淀交付物)/.test(text)) {
    return labels.action
  }
  if (/^(教学讲解)/.test(text)) return labels.team
  return ""
}

function resolveDisplayHeading(text: string, headings: SafeHeadings) {
  if (/^(多智能体协作结论)(?:\s|$|[：:])/.test(text)) {
    return headings.final
  }
  if (
    /^(各岗位独立输出|虚拟员工分工|岗位协作|协作回答|各岗位|分工)(?:\s|$|[：:])/.test(
      text
    )
  ) {
    return headings.references
  }
  if (/^(教学讲解)(?:\s|$|[：:])/.test(text)) {
    return headings.explanation
  }
  if (
    /^(主岗汇总方案|主岗综合方案|团队执行稿)(?:\s|$|[：:])/.test(text)
  ) {
    return headings.action
  }
  return text
}
