import {
  Button,
  Dialog,
  DialogContent,
  DialogDescription,
  DialogFooter,
  DialogHeader,
  DialogTitle,
} from "@workspace/ui-components"
import {
  type FileConversationUseState,
  fileDisplayName,
  fileObjectKey,
  fileReadableState,
  formatFileSize,
} from "../domain/logic"
import { readTrainingFileSummary } from "../domain/read-summary"
import type { TrainingFileItem } from "../types"
import { FilesDetailRow, FilesDetailSection } from "./files-detail-fields"
import { FilesKindBadge } from "./files-kind-badge"
import { FilesMetadataSection } from "./files-metadata-section"
import { FilesUseStateSection } from "./files-use-state-section"

interface FilesDetailDialogProps {
  file: TrainingFileItem | null
  onOpenChange: (open: boolean) => void
  open: boolean
  t: (key: string, values?: Record<string, unknown>) => string
  useState: FileConversationUseState | null
}

export function FilesDetailDialog({
  file,
  onOpenChange,
  open,
  t,
  useState,
}: FilesDetailDialogProps) {
  if (!file) return null

  const readSummary = readTrainingFileSummary(file)
  const readStatus = fileReadableState(file)

  return (
    <Dialog open={open} onOpenChange={onOpenChange}>
      <DialogContent>
        <DialogHeader>
          <DialogTitle>{t("commerce.files.detailDialog.title")}</DialogTitle>
          <DialogDescription>
            {t("commerce.files.detailDialog.description")}
          </DialogDescription>
        </DialogHeader>
        <div className="grid max-h-[70vh] gap-4 overflow-y-auto pr-1">
          <FilesDetailSection
            title={t("commerce.files.detailDialog.sections.file")}
          >
            <FilesDetailRow
              label={t("commerce.files.detailDialog.fields.displayName")}
              copyValue={fileDisplayName(file)}
              value={fileDisplayName(file)}
            />
            <FilesDetailRow
              label={t("commerce.files.detailDialog.fields.type")}
              value={<FilesKindBadge file={file} t={t} />}
            />
            <FilesDetailRow
              label={t("commerce.files.detailDialog.fields.mimeType")}
              copyValue={file.mimeType}
              value={file.mimeType}
            />
            <FilesDetailRow
              label={t("commerce.files.detailDialog.fields.size")}
              value={formatFileSize(file.fileSize)}
            />
            <FilesDetailRow
              label={t("commerce.files.detailDialog.fields.recordStatus")}
              copyValue={file.status}
              value={file.status}
            />
            <FilesDetailRow
              label={t("commerce.files.detailDialog.fields.createdAt")}
              value={new Date(file.createdAt).toLocaleString()}
            />
          </FilesDetailSection>
          <FilesDetailSection
            title={t("commerce.files.detailDialog.sections.storage")}
          >
            <FilesDetailRow
              label={t("commerce.files.detailDialog.fields.fileId")}
              copyValue={file.id}
              value={file.id}
            />
            <FilesDetailRow
              label={t("commerce.files.detailDialog.fields.conversationId")}
              copyValue={file.conversationId}
              value={file.conversationId}
            />
            <FilesDetailRow
              label={t("commerce.files.detailDialog.fields.messageId")}
              copyValue={file.messageId}
              value={file.messageId}
            />
            <FilesDetailRow
              label={t("commerce.files.detailDialog.fields.bucket")}
              copyValue={file.bucket}
              value={file.bucket}
            />
            <FilesDetailRow
              label={t("commerce.files.detailDialog.fields.objectKey")}
              copyValue={file.objectKey}
              value={file.objectKey}
            />
            <FilesDetailRow
              label={t("commerce.files.detailDialog.fields.objectRef")}
              copyValue={fileObjectKey(file)}
              value={fileObjectKey(file)}
            />
            <FilesDetailRow
              label={t("commerce.files.detailDialog.fields.sha256")}
              copyValue={file.sha256}
              value={file.sha256}
            />
          </FilesDetailSection>
          <FilesDetailSection
            title={t("commerce.files.detailDialog.sections.reading")}
          >
            <FilesDetailRow
              label={t("commerce.files.detailDialog.fields.readStatus")}
              value={t(`commerce.files.status.${readStatus}`)}
            />
            <FilesDetailRow
              label={t("commerce.files.detailDialog.fields.summary")}
              copyValue={readSummary.summary}
              value={readSummary.summary}
            />
            <FilesDetailRow
              label={t("commerce.files.detailDialog.fields.failureReason")}
              copyValue={readSummary.failureReason}
              value={readSummary.failureReason}
            />
            <FilesDetailRow
              label={t("commerce.files.detailDialog.fields.textChars")}
              value={readSummary.textChars}
            />
            <FilesDetailRow
              label={t("commerce.files.detailDialog.fields.parser")}
              copyValue={readSummary.parser}
              value={readSummary.parser}
            />
            <FilesDetailRow
              label={t("commerce.files.detailDialog.fields.contentKind")}
              copyValue={readSummary.contentKind}
              value={readSummary.contentKind}
            />
            <FilesDetailRow
              label={t("commerce.files.detailDialog.fields.truncated")}
              value={
                readSummary.truncated
                  ? t("commerce.files.detailDialog.yes")
                  : t("commerce.files.detailDialog.no")
              }
            />
          </FilesDetailSection>
          {useState ? <FilesUseStateSection state={useState} t={t} /> : null}
          <FilesMetadataSection file={file} t={t} />
        </div>
        <DialogFooter>
          <Button variant="outline" onClick={() => onOpenChange(false)}>
            {t("commerce.files.detailDialog.close")}
          </Button>
        </DialogFooter>
      </DialogContent>
    </Dialog>
  )
}
