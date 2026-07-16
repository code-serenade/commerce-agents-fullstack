import { StatusBadge, type StatusTone } from "../../components/commerce-ui"
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
      {t(`commerce.files.detailDialog.usageState.${state}`)}
    </StatusBadge>
  )
}
