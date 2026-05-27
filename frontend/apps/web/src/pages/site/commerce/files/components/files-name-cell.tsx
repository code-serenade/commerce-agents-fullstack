import { FileText } from "lucide-react"
import { FilesKindBadge } from "./files-kind-badge"
import { FilesReadSummary } from "./files-read-summary"
import type { TrainingFileItem } from "../types"

interface FilesNameCellProps {
  file: TrainingFileItem
  getFileName: (file: TrainingFileItem) => string
  t: (key: string, options?: Record<string, unknown>) => string
}

export function FilesNameCell({ file, getFileName, t }: FilesNameCellProps) {
  return (
    <div className="flex min-w-0 items-start gap-2">
      <FileText className="mt-0.5 size-4 shrink-0 text-muted-foreground" />
      <div className="min-w-0">
        <div className="max-w-md truncate text-sm font-medium text-foreground">
          {getFileName(file)}
        </div>
        <div className="mt-1 flex max-w-md min-w-0 flex-wrap items-center gap-1.5">
          <FilesKindBadge file={file} t={t} />
          <span className="min-w-0 truncate text-xs text-muted-foreground">
            {file.mimeType || file.objectKey}
          </span>
        </div>
        <FilesReadSummary file={file} t={t} />
      </div>
    </div>
  )
}
