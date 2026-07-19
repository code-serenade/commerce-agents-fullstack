import {
  AlignLeft,
  FileSpreadsheet,
  FileText,
  type LucideIcon,
} from "lucide-react"
import type { TrainingFileItem } from "../types"
import { FilesImageThumbnail } from "./files-image-thumbnail"
import { filesIconKind, type FilesIconKind } from "./files-type-icon-kind"

const ICON_CONFIG: Record<
  Exclude<FilesIconKind, "image">,
  { className: string; icon: LucideIcon }
> = {
  document: {
    className:
      "border-blue-200 bg-blue-50 text-blue-600 dark:border-blue-900 dark:bg-blue-950/40 dark:text-blue-300",
    icon: FileText,
  },
  sheet: {
    className:
      "border-emerald-200 bg-emerald-50 text-emerald-600 dark:border-emerald-900 dark:bg-emerald-950/40 dark:text-emerald-300",
    icon: FileSpreadsheet,
  },
  text: {
    className:
      "border-amber-200 bg-amber-50 text-amber-600 dark:border-amber-900 dark:bg-amber-950/40 dark:text-amber-300",
    icon: AlignLeft,
  },
}

export function FilesTypeIcon({ file }: { file: TrainingFileItem }) {
  const kind = filesIconKind(file)

  if (kind === "image") return <FilesImageThumbnail file={file} />

  const config = ICON_CONFIG[kind]
  const Icon = config.icon

  return (
    <div
      aria-hidden="true"
      className={`flex size-12 shrink-0 items-center justify-center rounded-md border ${config.className}`}
      data-file-icon-kind={kind}
    >
      <Icon className="size-5" strokeWidth={1.75} />
    </div>
  )
}
