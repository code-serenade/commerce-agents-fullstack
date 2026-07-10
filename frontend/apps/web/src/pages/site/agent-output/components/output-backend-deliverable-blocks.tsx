import { Spinner } from "@workspace/ui-components"

export function LoadingState({ label }: { label: string }) {
  return (
    <div className="flex min-h-40 items-center justify-center gap-2 text-sm text-muted-foreground">
      <Spinner />
      <span>{label}</span>
    </div>
  )
}

export function EmptyState({ detail, title }: { detail: string; title: string }) {
  return (
    <div className="flex min-h-40 flex-col items-center justify-center rounded-lg border border-dashed border-(--app-border) bg-muted/20 px-4 text-center">
      <p className="text-sm font-medium text-foreground">{title}</p>
      <p className="mt-1 max-w-lg text-sm/6 text-muted-foreground">{detail}</p>
    </div>
  )
}

export function ArtifactMeta({ label, value }: { label: string; value: string }) {
  return (
    <div className="min-w-0 rounded-lg border border-(--app-border) bg-(--app-surface) px-3 py-2 shadow-sm">
      <div className="text-[11px] font-medium text-muted-foreground">
        {label}
      </div>
      <div className="mt-0.5 truncate text-sm font-semibold text-foreground">
        {value}
      </div>
    </div>
  )
}
