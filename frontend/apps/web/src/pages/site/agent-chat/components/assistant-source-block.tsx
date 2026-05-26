import type { AssistantAnswerBlock } from "../domain/assistant-answer-blocks"
import { renderInlineMarkdown } from "./assistant-inline-markdown"

export function SourcesBlock({
  block,
}: {
  block: Extract<AssistantAnswerBlock, { kind: "sources" }>
}) {
  return (
    <section className="space-y-3 rounded-lg border border-border bg-card p-3 shadow-sm">
      {block.intro ? (
        <p className="wrap-break-word text-sm/6 font-medium text-foreground">
          {renderInlineMarkdown(block.intro)}
        </p>
      ) : null}
      <ol className="space-y-3">
        {block.sources.map((source, index) => (
          <li
            className="grid gap-1 rounded-md border border-border bg-background p-3"
            key={`${index}-${source.url}`}
          >
            <div className="flex min-w-0 flex-wrap items-center gap-2">
              <span className="rounded-full border border-border px-2 py-0.5 text-[11px] font-medium text-muted-foreground">
                {index + 1}
              </span>
              <a
                className="wrap-break-word font-medium text-primary underline-offset-4 hover:underline"
                href={source.url}
                rel="noreferrer"
                target="_blank"
              >
                {renderInlineMarkdown(source.title)}
              </a>
              {source.publishedAt ? (
                <span className="text-xs/5 text-muted-foreground">
                  {source.publishedAt}
                </span>
              ) : null}
            </div>
            <a
              className="wrap-break-word text-xs/5 text-muted-foreground underline-offset-4 hover:underline"
              href={source.url}
              rel="noreferrer"
              target="_blank"
            >
              {source.url}
            </a>
            {source.snippet ? (
              <p className="wrap-break-word text-sm/6 text-muted-foreground">
                {renderInlineMarkdown(source.snippet)}
              </p>
            ) : null}
          </li>
        ))}
      </ol>
    </section>
  )
}
