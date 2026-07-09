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
      className={`flex items-center justify-center px-4 ${
        fullscreen ? "min-h-svh w-full" : "min-h-72 w-full py-12"
      }`}
    >
      <div className="grid justify-items-center gap-3 text-center">
        <div className="flex size-12 items-center justify-center text-(--app-primary)">
          <Spinner size="lg" />
        </div>
        <p className="text-sm font-semibold text-(--app-text)">{label}</p>
      </div>
    </div>
  )
}
