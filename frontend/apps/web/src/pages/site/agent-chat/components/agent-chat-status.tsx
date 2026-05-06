import { Alert } from "@workspace/ui-components/stable/alert"

export interface AgentChatStatusProps {
  error: string | null
}

export function AgentChatStatus({ error }: AgentChatStatusProps) {
  if (!error) return null

  return (
    <div className="mx-auto w-full max-w-6xl shrink-0 px-4 pt-4 sm:px-6">
      <Alert variant="error" description={error} />
    </div>
  )
}
