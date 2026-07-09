import { Eye, FileCheck2, MinusCircle, Plus, RefreshCw } from "lucide-react"
import { useTranslation } from "react-i18next"
import { Badge, Button, Checkbox } from "@workspace/ui-components"
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
        <FilesNameCell file={file} getFileName={getFileName} t={t} />
      ),
      minWidth: "18rem",
    },
    {
      key: "status",
      header: t("commerce.files.table.status"),
      render: (file) => <ReadableBadge state={fileReadableState(file)} />,
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
      key: "size",
      header: t("commerce.files.table.size"),
      render: (file) => (
        <span className="text-sm text-muted-foreground">
          {formatFileSize(file.fileSize)}
        </span>
      ),
      width: "6rem",
    },
    {
      key: "created",
      header: t("commerce.files.table.createdAt"),
      render: (file) => (
        <span className="text-sm text-muted-foreground">
          {new Date(file.createdAt).toLocaleString()}
        </span>
      ),
      width: "11rem",
    },
    {
      key: "actions",
      header: t("commerce.files.table.actions"),
      render: (file) => {
        const state = getFileConversationUseState(file)
        const isSelectedForTurn = state === "selectedForTurn"
        const isUsing = usingFileIds.has(file.id)
        return (
          <div className="flex flex-nowrap items-center justify-end gap-2">
            <Button
              size="sm"
              variant="outline"
              onClick={() => onOpenFileDetail(file)}
            >
              <Eye className="size-4" />
              {t("commerce.files.table.detail")}
            </Button>
            <Button
              size="sm"
              variant={isSelectedForTurn ? "outline" : "default"}
              disabled={isUsing || !turnActionsEnabled}
              onClick={() => onToggleCurrentTurn(file)}
            >
              {isUsing ? (
                <RefreshCw className="size-4 animate-spin" />
              ) : state === "selectedForTurn" ? (
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
    <CommerceTable
      columns={columns}
      emptyText={t("commerce.files.empty")}
      getRowId={(file) => file.id}
      items={files}
      loading={loading}
    />
  )
}

function ReadableBadge({
  state,
}: {
  state: "readable" | "pending" | "failed" | "unsupported"
}) {
  const { t } = useTranslation()
  const variant = state === "failed" ? "destructive" : "secondary"

  return <Badge variant={variant}>{t(`commerce.files.status.${state}`)}</Badge>
}
