import { Spinner } from "@workspace/ui-components"

export function LoadingState({ label }: { label: string }) {
  return (
    <div className="flex min-h-40 items-center justify-center gap-2 text-sm text-muted-foreground">
      <Spinner />
      <span>{label}</span>
    </div>
  )
}

export function EmptyState({
  detail,
  title,
}: {
  detail: string
  title: string
}) {
  return (
    <div className="flex min-h-40 flex-col items-center justify-center rounded-lg border border-dashed border-(--document-card-border) bg-muted/30 px-4 text-center">
      <p className="text-sm font-medium text-foreground">{title}</p>
      <p className="mt-1 max-w-lg text-sm/6 text-muted-foreground">{detail}</p>
    </div>
  )
}
