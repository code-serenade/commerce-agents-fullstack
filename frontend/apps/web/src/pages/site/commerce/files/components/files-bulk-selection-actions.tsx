import { Trash2 } from "lucide-react"
import { Button } from "@workspace/ui-components"

interface FilesBulkSelectionActionsProps {
  deleting: boolean
  onClearSelection: () => void
  onDeleteSelected: () => void
  selectedCount: number
  t: (key: string, options?: Record<string, unknown>) => string
}

export function FilesBulkSelectionActions({
  deleting,
  onClearSelection,
  onDeleteSelected,
  selectedCount,
  t,
}: FilesBulkSelectionActionsProps) {
  if (selectedCount === 0) return null

  return (
    <div className="flex flex-wrap items-center gap-2">
      <Button
        size="sm"
        variant="destructive"
        disabled={deleting}
        onClick={onDeleteSelected}
      >
        <Trash2 className="size-4" />
        {t("commerce.files.deleteSelected", {
          count: selectedCount,
        })}
      </Button>
      <Button
        size="sm"
        variant="outline"
        disabled={deleting}
        onClick={onClearSelection}
      >
        {t("commerce.files.clearSelection")}
      </Button>
    </div>
  )
}
