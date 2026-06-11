import {
  Button,
  Dialog,
  DialogContent,
  DialogFooter,
  DialogHeader,
  DialogTitle,
} from "@workspace/ui-components"
import type { TrainingFilesDeleteImpact } from "../domain/logic"
import type { TrainingFileItem } from "../types"

export function FilesDeleteDialog({
  deleteImpact,
  deleting,
  onConfirm,
  onOpenChange,
  open,
  selectedFiles,
  t,
}: {
  deleteImpact: TrainingFilesDeleteImpact
  deleting: boolean
  onConfirm: () => void
  onOpenChange: (open: boolean) => void
  open: boolean
  selectedFiles: TrainingFileItem[]
  t: (key: string, values?: Record<string, number | string>) => string
}) {
  const targetNames = deleteImpact.targetNames.join(" / ")
  const hiddenText =
    deleteImpact.hiddenNames > 0
      ? t("commerce.files.deleteDialog.hiddenSuffix", {
          count: deleteImpact.hiddenNames,
        })
      : ""

  return (
    <Dialog open={open} onOpenChange={onOpenChange}>
      <DialogContent>
        <DialogHeader>
          <DialogTitle>{t("commerce.files.deleteDialog.title")}</DialogTitle>
        </DialogHeader>
        <p className="text-sm text-muted-foreground">
          {t("commerce.files.deleteDialog.description", {
            count: selectedFiles.length,
          })}
        </p>
        <div className="grid gap-2 rounded-lg border border-destructive/25 bg-destructive/5 p-3 text-xs text-muted-foreground">
          <p className="font-medium text-destructive">
            {t("commerce.files.deleteDialog.impactTitle")}
          </p>
          <p>
            {t("commerce.files.deleteDialog.impactSummary", {
              attached: deleteImpact.attached,
              readable: deleteImpact.readable,
              selectedForTurn: deleteImpact.selectedForTurn,
              total: deleteImpact.total,
            })}
          </p>
          {targetNames ? (
            <p>
              {t("commerce.files.deleteDialog.targetNames", {
                hiddenText,
                names: targetNames,
              })}
            </p>
          ) : null}
        </div>
        <DialogFooter>
          <Button
            variant="outline"
            disabled={deleting}
            onClick={() => onOpenChange(false)}
          >
            {t("commerce.files.deleteDialog.cancel")}
          </Button>
          <Button
            variant="destructive"
            disabled={deleting || selectedFiles.length === 0}
            onClick={onConfirm}
          >
            {t("commerce.files.deleteDialog.confirm")}
          </Button>
        </DialogFooter>
      </DialogContent>
    </Dialog>
  )
}
