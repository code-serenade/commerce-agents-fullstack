import { X } from "lucide-react"
import { Button, NativeSelect } from "@workspace/ui-components"
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

const KIND_FILTERS: TrainingFileKindFilter[] = [
  "all",
  "image",
  "pdf",
  "word",
  "sheet",
  "text",
  "other",
]

const READ_STATUS_FILTERS: TrainingFileReadStatusFilter[] = [
  "all",
  "readable",
  "pending",
  "failed",
  "unsupported",
]

const USAGE_FILTERS: TrainingFileUsageFilter[] = [
  "all",
  "available",
  "attached",
  "selectedForTurn",
]

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
        options={KIND_FILTERS.map((value) => ({
          label: t(`commerce.files.filters.kind.${value}`),
          value,
        }))}
        onValueChange={(value) =>
          onKindFilterChange(value as TrainingFileKindFilter)
        }
      />
      <NativeSelect
        value={readStatusFilter}
        options={READ_STATUS_FILTERS.map((value) => ({
          label: t(`commerce.files.filters.readStatus.${value}`),
          value,
        }))}
        onValueChange={(value) =>
          onReadStatusFilterChange(value as TrainingFileReadStatusFilter)
        }
      />
      <NativeSelect
        value={usageFilter}
        options={USAGE_FILTERS.map((value) => ({
          label: t(`commerce.files.filters.usage.${value}`),
          value,
        }))}
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
