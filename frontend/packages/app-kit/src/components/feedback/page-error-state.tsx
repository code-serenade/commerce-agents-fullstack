import { Alert } from "@workspace/ui-components/stable/alert"

export interface PageErrorStateProps {
  title?: string
  message: string
}

export function PageErrorState({
  title = "Page Error",
  message,
}: PageErrorStateProps) {
  return (
    <div className="mx-auto flex min-h-[40vh] max-w-2xl items-center justify-center px-4 py-12">
      <div className="w-full">
        <Alert variant="error" title={title} description={message} />
      </div>
    </div>
  )
}
