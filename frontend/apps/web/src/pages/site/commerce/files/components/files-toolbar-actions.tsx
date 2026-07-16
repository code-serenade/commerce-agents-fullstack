import type { TrainingFilesFilterSummary } from "../domain/logic"
import { FilesBulkSelectionActions } from "./files-bulk-selection-actions"
import { FilesFilterSummary } from "./files-filter-summary"

export function FilesToolbarActions({
  deleting,
  filterSummary,
  onClearSelection,
  onDeleteSelected,
  selectedCount,
  t,
}: {
  deleting: boolean
  filterSummary: TrainingFilesFilterSummary
  onClearSelection: () => void
  onDeleteSelected: () => void
  selectedCount: number
  t: (key: string, options?: Record<string, unknown>) => string
}) {
  return (
    <div className="flex flex-wrap items-center justify-end gap-2">
      <FilesFilterSummary summary={filterSummary} t={t} />
      <FilesBulkSelectionActions
        deleting={deleting}
        onClearSelection={onClearSelection}
        onDeleteSelected={onDeleteSelected}
        selectedCount={selectedCount}
        t={t}
      />
    </div>
  )
}
