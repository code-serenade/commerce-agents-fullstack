import { Eye, FileCheck2, MinusCircle, Plus, RefreshCw } from "lucide-react"
import { useTranslation } from "react-i18next"
import { Button } from "@workspace/ui-components"
import type { FileConversationUseState } from "../domain/logic"
import type { TrainingFileItem } from "../types"

export function FilesRowActions({
  file,
  isUsing,
  onOpenFileDetail,
  onToggleCurrentTurn,
  state,
  turnActionsEnabled,
}: {
  file: TrainingFileItem
  isUsing: boolean
  onOpenFileDetail: (file: TrainingFileItem) => void
  onToggleCurrentTurn: (file: TrainingFileItem) => void
  state: FileConversationUseState
  turnActionsEnabled: boolean
}) {
  const { t } = useTranslation()
  const isSelectedForTurn = state === "selectedForTurn"
  const detailLabel = t("commerce.files.table.detail")

  return (
    <div className="flex flex-nowrap items-center justify-end gap-1.5">
      <Button
        size="xs"
        variant="outline"
        aria-label={detailLabel}
        title={detailLabel}
        onClick={() => onOpenFileDetail(file)}
      >
        <Eye className="size-4" />
        {detailLabel}
      </Button>
      <Button
        size="xs"
        variant="outline"
        disabled={isUsing || !turnActionsEnabled}
        onClick={() => onToggleCurrentTurn(file)}
      >
        {isUsing ? (
          <RefreshCw className="size-4 animate-spin" />
        ) : isSelectedForTurn ? (
          <MinusCircle className="size-4" />
        ) : state === "attached" ? (
          <FileCheck2 className="size-4" />
        ) : (
          <Plus className="size-4" />
        )}
        {isUsing
          ? t("commerce.files.table.using")
          : t(`commerce.files.table.useState.${state}`)}
      </Button>
    </div>
  )
}
