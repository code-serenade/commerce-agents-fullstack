import { X } from "lucide-react"
import { Button, NativeSelect } from "@workspace/ui-components"
import {
  fileKindFilterOptions,
  fileReadStatusFilterOptions,
  fileUsageFilterOptions,
} from "../../domain/enum-mappings"
import type {
  TrainingFileKindFilter,
  TrainingFileReadStatusFilter,
} from "../domain/logic"
import type { TrainingFileUsageFilter } from "../domain/usage-filter"

interface FilesFiltersProps {
  kindFilter: TrainingFileKindFilter
  hasActiveFilters: boolean
  onClearFilters: () => void
  onKindFilterChange: (value: TrainingFileKindFilter) => void
  onReadStatusFilterChange: (value: TrainingFileReadStatusFilter) => void
  onUsageFilterChange: (value: TrainingFileUsageFilter) => void
  readStatusFilter: TrainingFileReadStatusFilter
  t: (key: string) => string
  usageFilter: TrainingFileUsageFilter
}

export function FilesFilters({
  hasActiveFilters,
  kindFilter,
  onClearFilters,
  onKindFilterChange,
  onReadStatusFilterChange,
  onUsageFilterChange,
  readStatusFilter,
  t,
  usageFilter,
}: FilesFiltersProps) {
  return (
    <div className="flex w-full flex-col gap-2 sm:w-auto sm:flex-row">
      <NativeSelect
        value={kindFilter}
        options={fileKindFilterOptions(t)}
        onValueChange={(value) =>
          onKindFilterChange(value as TrainingFileKindFilter)
        }
      />
      <NativeSelect
        value={readStatusFilter}
        options={fileReadStatusFilterOptions(t)}
        onValueChange={(value) =>
          onReadStatusFilterChange(value as TrainingFileReadStatusFilter)
        }
      />
      <NativeSelect
        value={usageFilter}
        options={fileUsageFilterOptions(t)}
        onValueChange={(value) =>
          onUsageFilterChange(value as TrainingFileUsageFilter)
        }
      />
      {hasActiveFilters ? (
        <Button size="sm" variant="outline" onClick={onClearFilters}>
          <X className="size-4" />
          {t("commerce.files.clearFilters")}
        </Button>
      ) : null}
    </div>
  )
}
