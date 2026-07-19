import type { LucideIcon } from "lucide-react"

export function InlineHandoffItem({
  item,
  itemKey,
}: {
  item: {
    detail: string
    icon: LucideIcon
    label: string
  }
  itemKey: string
}) {
  const Icon = item.icon

  return (
    <div
      key={itemKey}
      className="flex min-w-0 items-center gap-2 rounded-lg border border-(--agent-chat-soft-border) bg-(--agent-chat-subtle-surface) px-3 py-2 text-xs text-muted-foreground"
    >
      <Icon className="size-3.5 shrink-0" />
      <span className="shrink-0 font-medium text-foreground">
        {item.label}
      </span>
      <span className="min-w-0 truncate">{item.detail}</span>
    </div>
  )
}
