import type { ChangeEvent, RefObject } from "react"
import { Loader2, Upload } from "lucide-react"
import { Button } from "@workspace/ui-components"
import { AI_READABLE_FILE_ACCEPT } from "../../../agent-chat/domain/agent-chat-upload-validation"

interface FilesPageActionsProps {
  canUploadFiles: boolean
  onUploadChange: (event: ChangeEvent<HTMLInputElement>) => void
  onUploadClick: () => void
  t: (key: string) => string
  uploadInputRef: RefObject<HTMLInputElement | null>
  uploading: boolean
}

export function FilesPageActions({
  canUploadFiles,
  onUploadChange,
  onUploadClick,
  t,
  uploadInputRef,
  uploading,
}: FilesPageActionsProps) {
  return (
    <div className="flex flex-wrap items-center justify-end gap-2">
      <input
        ref={uploadInputRef}
        accept={AI_READABLE_FILE_ACCEPT}
        className="hidden"
        multiple
        type="file"
        onChange={onUploadChange}
      />
      <Button
        size="lg"
        variant="primary"
        disabled={!canUploadFiles}
        className="disabled:border-primary/35 disabled:bg-(--agent-chat-accent-soft) disabled:text-(--agent-chat-accent-foreground) disabled:opacity-70"
        onClick={onUploadClick}
      >
        {uploading ? (
          <Loader2 className="size-4 animate-spin" />
        ) : (
          <Upload className="size-4" />
        )}
        {uploading
          ? t("commerce.files.uploadingAction")
          : t("commerce.files.uploadAction")}
      </Button>
    </div>
  )
}
