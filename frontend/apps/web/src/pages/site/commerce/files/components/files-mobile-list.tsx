import { Checkbox } from "@workspace/ui-components"
import {
  fileReadableState,
  type FileConversationUseState,
} from "../domain/logic"
import type { TrainingFileItem } from "../types"
import { FilesNameCell } from "./files-name-cell"
import { FilesReadableStatus } from "./files-readable-status"
import { FilesRowActions } from "./files-row-actions"
import { FilesUseStateBadge } from "./files-use-state-badge"

export function FilesMobileList({
  files,
  formatFileSize,
  getFileConversationUseState,
  getFileName,
  onOpenFileDetail,
  onToggleCurrentTurn,
  onToggleFileSelection,
  selectedFileIds,
  t,
  turnActionsEnabled,
  usingFileIds,
}: {
  files: TrainingFileItem[]
  formatFileSize: (size: number | null | undefined) => string
  getFileConversationUseState: (file: TrainingFileItem) => FileConversationUseState
  getFileName: (file: TrainingFileItem) => string
  onOpenFileDetail: (file: TrainingFileItem) => void
  onToggleCurrentTurn: (file: TrainingFileItem) => void
  onToggleFileSelection: (fileId: string) => void
  selectedFileIds: Set<string>
  t: (key: string, options?: Record<string, unknown>) => string
  turnActionsEnabled: boolean
  usingFileIds: Set<string>
}) {
  return (
    <div className="divide-y divide-(--app-border) md:hidden">
      {files.map((file) => {
        const state = getFileConversationUseState(file)
        return (
          <article key={file.id} className="grid gap-3 py-3 first:pt-0 last:pb-0">
            <div className="flex min-w-0 items-start gap-2">
              <Checkbox
                checked={selectedFileIds.has(file.id)}
                onCheckedChange={() => onToggleFileSelection(file.id)}
              />
              <FilesNameCell
                file={file}
                formatFileSize={formatFileSize}
                getFileName={getFileName}
                onOpenFileDetail={onOpenFileDetail}
                t={t}
              />
            </div>
            <div className="flex flex-wrap items-center justify-between gap-2 pl-6">
              <div className="flex flex-wrap items-center gap-2">
                <FilesReadableStatus state={fileReadableState(file)} />
                <FilesUseStateBadge state={state} t={t} />
              </div>
              <FilesRowActions
                file={file}
                isUsing={usingFileIds.has(file.id)}
                onOpenFileDetail={onOpenFileDetail}
                onToggleCurrentTurn={onToggleCurrentTurn}
                state={state}
                turnActionsEnabled={turnActionsEnabled}
              />
            </div>
          </article>
        )
      })}
    </div>
  )
}
