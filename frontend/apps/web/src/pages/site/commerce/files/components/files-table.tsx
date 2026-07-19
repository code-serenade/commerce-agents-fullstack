import { Plus } from "lucide-react"
import { useTranslation } from "react-i18next"
import { Button, Checkbox } from "@workspace/ui-components"
import {
  CommerceTable,
  type CommerceColumn,
} from "../../components/commerce-ui"
import {
  fileReadableState,
  type FileConversationUseState,
} from "../domain/logic"
import type { TrainingFileItem } from "../types"
import { FilesNameCell } from "./files-name-cell"
import { FilesMobileList } from "./files-mobile-list"
import { FilesReadableStatus } from "./files-readable-status"
import { FilesRowActions } from "./files-row-actions"
import { FilesUseStateBadge } from "./files-use-state-badge"

export function FilesTable({
  dataFileCount,
  files,
  formatFileSize,
  getFileConversationUseState,
  getFileName,
  hasActiveFilters,
  loading,
  canUploadEmpty = true,
  onClearFilters,
  onUploadEmpty,
  onOpenFileDetail,
  onToggleCurrentTurn,
  onToggleFileSelection,
  turnActionsEnabled,
  selectedFileIds,
  usingFileIds,
}: {
  dataFileCount: number
  files: TrainingFileItem[]
  formatFileSize: (size: number | null | undefined) => string
  getFileConversationUseState: (
    file: TrainingFileItem
  ) => FileConversationUseState
  getFileName: (file: TrainingFileItem) => string
  hasActiveFilters: boolean
  loading: boolean
  canUploadEmpty?: boolean
  onClearFilters: () => void
  onUploadEmpty?: () => void
  onOpenFileDetail: (file: TrainingFileItem) => void
  onToggleCurrentTurn: (file: TrainingFileItem) => void
  onToggleFileSelection: (fileId: string) => void
  turnActionsEnabled: boolean
  selectedFileIds: Set<string>
  usingFileIds: Set<string>
}) {
  const { t } = useTranslation()
  const columns: CommerceColumn<TrainingFileItem>[] = [
    {
      key: "selection",
      header: "",
      render: (file) => (
        <Checkbox
          checked={selectedFileIds.has(file.id)}
          onCheckedChange={() => onToggleFileSelection(file.id)}
        />
      ),
      sticky: "left",
      width: "2.25rem",
    },
    {
      key: "name",
      header: t("commerce.files.table.name"),
      render: (file) => (
        <FilesNameCell
          file={file}
          formatFileSize={formatFileSize}
          getFileName={getFileName}
          onOpenFileDetail={onOpenFileDetail}
          t={t}
        />
      ),
      minWidth: "18rem",
    },
    {
      key: "status",
      header: t("commerce.files.table.status"),
      render: (file) => <FilesReadableStatus state={fileReadableState(file)} />,
      width: "7rem",
    },
    {
      key: "use-state",
      header: t("commerce.files.table.useStateHeader"),
      render: (file) => (
        <FilesUseStateBadge state={getFileConversationUseState(file)} t={t} />
      ),
      width: "7.5rem",
    },
    {
      key: "created",
      header: t("commerce.files.table.createdAt"),
      render: (file) => (
        <span className="text-sm text-muted-foreground">
          {new Date(file.createdAt).toLocaleDateString()}
        </span>
      ),
      width: "11rem",
    },
    {
      key: "actions",
      header: t("commerce.files.table.actions"),
      render: (file) => {
        const state = getFileConversationUseState(file)
        return (
          <FilesRowActions
            file={file}
            isUsing={usingFileIds.has(file.id)}
            onOpenFileDetail={onOpenFileDetail}
            onToggleCurrentTurn={onToggleCurrentTurn}
            state={state}
            turnActionsEnabled={turnActionsEnabled}
          />
        )
      },
      sticky: "right",
      width: "13rem",
    },
  ]

  if (!loading && files.length === 0 && hasActiveFilters && dataFileCount > 0) {
    return (
      <div className="flex min-h-44 flex-col items-center justify-center gap-3 rounded-lg border border-dashed border-(--app-border) bg-background px-4 text-center">
        <div className="grid gap-1">
          <p className="text-sm font-medium text-foreground">
            {t("commerce.files.noFilterResultTitle")}
          </p>
          <p className="max-w-xl text-sm text-muted-foreground">
            {t("commerce.files.noFilterResult")}
          </p>
        </div>
        <Button size="sm" variant="outline" onClick={onClearFilters}>
          {t("commerce.files.clearFilters")}
        </Button>
      </div>
    )
  }

  if (!loading && files.length === 0) {
    return (
      <div className="flex min-h-44 flex-col items-center justify-center gap-3 rounded-lg border border-dashed border-(--app-border) bg-background px-4 text-center">
        <div className="grid gap-1">
          <p className="text-sm font-medium text-foreground">
            {t("commerce.files.emptyTitle")}
          </p>
          <p className="max-w-xl text-sm text-muted-foreground">
            {t("commerce.files.empty")}
          </p>
        </div>
        <Button
          size="sm"
          variant="primary"
          disabled={!canUploadEmpty}
          className="disabled:border-primary/35 disabled:bg-(--agent-chat-accent-soft) disabled:text-(--agent-chat-accent-foreground) disabled:opacity-70"
          onClick={onUploadEmpty ?? onClearFilters}
        >
          <Plus className="size-4" />
          {t("commerce.files.emptyAction")}
        </Button>
      </div>
    )
  }

  return (
    <>
      <FilesMobileList
        files={files}
        formatFileSize={formatFileSize}
        getFileConversationUseState={getFileConversationUseState}
        getFileName={getFileName}
        onOpenFileDetail={onOpenFileDetail}
        onToggleCurrentTurn={onToggleCurrentTurn}
        onToggleFileSelection={onToggleFileSelection}
        selectedFileIds={selectedFileIds}
        t={t}
        turnActionsEnabled={turnActionsEnabled}
        usingFileIds={usingFileIds}
      />
      <div className="hidden md:block">
        <CommerceTable
          columns={columns}
          emptyText={t("commerce.files.empty")}
          getRowId={(file) => file.id}
          items={files}
          loading={loading}
        />
      </div>
    </>
  )
}
