import { Spinner } from "@workspace/ui-components"

export interface PageLoadingProps {
  label?: string
  fullscreen?: boolean
}

export function PageLoading({
  label = "Loading...",
  fullscreen = false,
}: PageLoadingProps) {
  return (
    <div
      className={`flex items-center justify-center ${
        fullscreen ? "min-h-svh w-full" : "w-full py-12"
      }`}
    >
      <div className="flex items-center gap-3 rounded-full border border-(--app-border) bg-(--app-surface) px-4 py-2 text-sm text-(--app-muted-text) shadow-[var(--ui-shadow-soft)]">
        <Spinner size="sm" />
        <span>{label}</span>
      </div>
    </div>
  )
}
