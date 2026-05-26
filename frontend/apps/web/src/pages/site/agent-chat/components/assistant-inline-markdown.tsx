import type { ReactNode } from "react"

export function renderInlineMarkdown(text: string): ReactNode[] {
  const nodes: ReactNode[] = []
  const pattern = /(\*\*([^*]+)\*\*)|(https?:\/\/[^\s)）]+[^\s)），。；;、])/g
  let lastIndex = 0
  let match: RegExpExecArray | null

  while ((match = pattern.exec(text)) !== null) {
    if (match.index > lastIndex) nodes.push(text.slice(lastIndex, match.index))
    if (match[2]) {
      nodes.push(
        <strong key={`${match.index}-${match[2]}`} className="font-semibold">
          {match[2]}
        </strong>
      )
    } else if (match[3]) {
      nodes.push(
        <a
          className="wrap-break-word text-primary underline-offset-4 hover:underline"
          href={match[3]}
          key={`${match.index}-${match[3]}`}
          rel="noreferrer"
          target="_blank"
        >
          {match[3]}
        </a>
      )
    }
    lastIndex = pattern.lastIndex
  }

  if (lastIndex < text.length) nodes.push(text.slice(lastIndex))
  return nodes.length ? nodes : [text]
}
