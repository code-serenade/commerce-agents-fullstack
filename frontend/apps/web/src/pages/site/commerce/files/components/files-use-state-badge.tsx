import { StatusBadge, type StatusTone } from "../../components/commerce-ui"
import { fileUsageLabel } from "../../domain/enum-mappings"
import type { FileConversationUseState } from "../domain/logic"

interface FilesUseStateBadgeProps {
  state: FileConversationUseState
  t: (key: string) => string
}

export function FilesUseStateBadge({ state, t }: FilesUseStateBadgeProps) {
  const tone: StatusTone =
    state === "selectedForTurn"
      ? "success"
      : state === "available"
        ? "muted"
        : "default"

  return (
    <StatusBadge tone={tone}>
      {fileUsageLabel(t, state)}
    </StatusBadge>
  )
}
