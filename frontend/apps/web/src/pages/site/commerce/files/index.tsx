import { useRef } from "react"
import { useTranslation } from "react-i18next"
import {
  CommerceBody,
  CommercePageHeader,
  CommercePanel,
  CommerceSurface,
  CommerceToolbar,
  InlineAlert,
} from "../components/commerce-ui"
import { FilesTable } from "./components/files-table"
import { FilesBulkSelectionActions } from "./components/files-bulk-selection-actions"
import { FilesFilters } from "./components/files-filters"
import { FilesDeleteDialog } from "./components/files-delete-dialog"
import { FilesDetailDialog } from "./components/files-detail-dialog"
import { FilesPageActions } from "./components/files-page-actions"
import { FilesFilterSummary } from "./components/files-filter-summary"
import { FilesTurnSelectionSummary } from "./components/files-turn-selection-summary"
import { FilesUsageSummary } from "./components/files-usage-summary"
import { useFilesPage } from "./use-files-page"

export default function FilesPage() {
  const { t } = useTranslation()
  const page = useFilesPage()
  const uploadInputRef = useRef<HTMLInputElement>(null)

  return (
    <CommerceSurface>
      <CommercePageHeader
        eyebrow={t("commerce.pages.files.eyebrow")}
        title={t("commerce.pages.files.title")}
        description={t("commerce.pages.files.description")}
        actions={
          <FilesPageActions
            canUploadFiles={page.canUploadFiles}
            onUploadChange={page.handleUploadChange}
            onUploadClick={() => uploadInputRef.current?.click()}
            t={t}
            uploadInputRef={uploadInputRef}
            uploading={page.uploading}
          />
        }
      />
      <CommerceBody>
        <InlineAlert
          title={t("commerce.pages.files.boundaryTitle")}
          description={t("commerce.pages.files.boundaryDescription")}
        />
        {page.error ? (
          <InlineAlert
            variant="error"
            title={t("commerce.files.errorTitle")}
            description={page.error}
          />
        ) : null}
        {!page.chatContextAvailable ? (
          <InlineAlert
            variant="warning"
            title={t("commerce.files.chatContextUnavailable.title")}
            description={t("commerce.files.chatContextUnavailable.description")}
          />
        ) : null}
        <CommercePanel
          title={t("commerce.files.panelTitle")}
          description={t("commerce.files.panelDescription")}
        >
          <div className="grid gap-3">
            <FilesUsageSummary summary={page.usageSummary} t={t} />
            <FilesTurnSelectionSummary
              canClearSelectedForTurn={page.canClearSelectedForTurn}
              onClearSelectedForTurn={page.clearSelectedForTurn}
              onShowSelectedForTurn={() =>
                page.setUsageFilter("selectedForTurn")
              }
              summary={page.turnSelectionSummary}
              t={t}
            />
            <CommerceToolbar
              actions={
                <FilesBulkSelectionActions
                  deleting={page.deleting}
                  onClearSelection={page.clearSelection}
                  onDeleteSelected={() => page.setDeleteDialogOpen(true)}
                  selectedCount={page.selectedFiles.length}
                  t={t}
                />
              }
              keyword={page.keyword}
              keywordPlaceholder={t("commerce.files.searchPlaceholder")}
              onKeywordChange={page.setKeyword}
              filters={
                <FilesFilters
                  hasActiveFilters={page.hasActiveFilters}
                  kindFilter={page.kindFilter}
                  onClearFilters={page.clearFilters}
                  onKindFilterChange={page.setKindFilter}
                  onReadStatusFilterChange={page.setReadStatusFilter}
                  onUsageFilterChange={page.setUsageFilter}
                  readStatusFilter={page.readStatusFilter}
                  t={t}
                  usageFilter={page.usageFilter}
                />
              }
            />
            <FilesFilterSummary summary={page.filterSummary} t={t} />
            <FilesTable
              canUploadEmpty={page.canUploadFiles}
              dataFileCount={page.dataFileCount}
              files={page.files}
              formatFileSize={page.formatFileSize}
              getFileConversationUseState={page.getFileConversationUseState}
              getFileName={page.getFileName}
              hasActiveFilters={page.hasActiveFilters}
              loading={page.loading}
              onClearFilters={page.clearFilters}
              onOpenFileDetail={page.openFileDetail}
              onUploadEmpty={() => uploadInputRef.current?.click()}
              onToggleCurrentTurn={(file) =>
                void page.toggleFileForCurrentTurn(file)
              }
              onToggleFileSelection={page.toggleFileSelection}
              selectedFileIds={page.selectedFileIds}
              turnActionsEnabled={page.chatContextAvailable}
              usingFileIds={page.usingFileIds}
            />
          </div>
        </CommercePanel>
      </CommerceBody>
      <FilesDeleteDialog
        deleteImpact={page.deleteImpact}
        deleting={page.deleting}
        onConfirm={() => void page.deleteSelectedFiles()}
        onOpenChange={page.setDeleteDialogOpen}
        open={page.deleteDialogOpen}
        selectedFiles={page.selectedFiles}
        t={t}
      />
      <FilesDetailDialog
        file={page.detailFile}
        onOpenChange={(open) => {
          if (!open) page.closeFileDetail()
        }}
        open={Boolean(page.detailFile)}
        t={t}
        useState={
          page.detailFile
            ? page.getFileConversationUseState(page.detailFile)
            : null
        }
      />
    </CommerceSurface>
  )
}
