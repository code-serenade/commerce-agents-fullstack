import { FilesKindBadge } from "./files-kind-badge"
import { FilesTypeIcon } from "./files-type-icon"
import type { TrainingFileItem } from "../types"

interface FilesNameCellProps {
  file: TrainingFileItem
  formatFileSize: (size: number | null | undefined) => string
  getFileName: (file: TrainingFileItem) => string
  onOpenFileDetail: (file: TrainingFileItem) => void
  t: (key: string, options?: Record<string, unknown>) => string
}

export function FilesNameCell({
  file,
  formatFileSize,
  getFileName,
  onOpenFileDetail,
  t,
}: FilesNameCellProps) {
  const fileName = getFileName(file)

  return (
    <div className="flex min-w-0 items-center gap-3">
      <FilesTypeIcon file={file} />
      <div className="min-w-0">
        <button
          type="button"
          className="block max-w-md cursor-pointer truncate text-left font-medium text-foreground hover:text-primary"
          title={fileName}
          onClick={() => onOpenFileDetail(file)}
        >
          {fileName}
        </button>
        <div className="mt-1 flex max-w-md min-w-0 items-center gap-1.5">
          <FilesKindBadge file={file} t={t} />
          <span className="text-xs text-muted-foreground" aria-hidden="true">
            /
          </span>
          <span className="truncate text-xs text-muted-foreground">
            {formatFileSize(file.fileSize)}
          </span>
        </div>
      </div>
    </div>
  )
}
