import { Badge } from "@workspace/ui-components"
import type { FileConversationUseState } from "../domain/logic"

interface FilesUseStateBadgeProps {
  state: FileConversationUseState
  t: (key: string) => string
}

export function FilesUseStateBadge({ state, t }: FilesUseStateBadgeProps) {
  const variant = state === "selectedForTurn" ? "default" : "outline"

  return (
    <Badge variant={variant}>
      {t(`commerce.files.detailDialog.usageState.${state}`)}
    </Badge>
  )
}
