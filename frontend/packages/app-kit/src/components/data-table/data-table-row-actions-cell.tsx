import {
  Button,
  Dialog,
  DialogContent,
  DialogDescription,
  DialogFooter,
  DialogHeader,
  DialogTitle,
  IconButton,
} from "@workspace/ui-components"
import {
  DropdownMenu,
  DropdownMenuContent,
  DropdownMenuItem,
  DropdownMenuTrigger,
} from "@workspace/ui-core/components/dropdown-menu"
import { MoreHorizontal, Pencil, Trash2 } from "lucide-react"
import { useState, type ReactNode } from "react"

import type {
  DataTableRowActionItem,
  DataTableRowActionsConfig,
} from "./data-table.types"
import {
  resolveRowActionContent,
  resolveRowActionDisabled,
} from "./data-table.utils"

export function DataTableRowActionsCell<T>({
  row,
  rowIndex,
  rowActions,
  moreItems,
  resolvedEditLabel,
  resolvedDeleteLabel,
  resolvedMoreLabel,
  resolvedCancelLabel,
  resolvedSaveLabel,
  resolvedConfirmDeleteLabel,
  resolvedDeleteDialogTitle,
  resolvedDeleteDialogDescription,
  onActionComplete,
}: {
  row: T
  rowIndex: number
  rowActions: DataTableRowActionsConfig<T>
  moreItems: readonly DataTableRowActionItem<T>[]
  resolvedEditLabel: ReactNode
  resolvedDeleteLabel: ReactNode
  resolvedMoreLabel: string
  resolvedCancelLabel: ReactNode
  resolvedSaveLabel: ReactNode
  resolvedConfirmDeleteLabel: ReactNode
  resolvedDeleteDialogTitle: ReactNode
  resolvedDeleteDialogDescription: ReactNode
  onActionComplete: () => void
}) {
  const [editDialogOpen, setEditDialogOpen] = useState(false)
  const [deleteDialogOpen, setDeleteDialogOpen] = useState(false)
  const [submittingEdit, setSubmittingEdit] = useState(false)
  const [submittingDelete, setSubmittingDelete] = useState(false)
  const editAction = rowActions.edit || null
  const deleteAction = rowActions.delete || null
  const hasMoreActions = moreItems.length > 0

  const handleEditConfirm = async () => {
    if (!editAction?.onConfirm || submittingEdit) {
      setEditDialogOpen(false)
      return
    }

    setSubmittingEdit(true)

    try {
      await editAction.onConfirm(row, rowIndex)
      setEditDialogOpen(false)
      onActionComplete()
    } catch (editError) {
      console.error("DataTable edit action failed:", editError)
    } finally {
      setSubmittingEdit(false)
    }
  }

  const handleDeleteConfirm = async () => {
    if (!deleteAction?.onConfirm || submittingDelete) {
      setDeleteDialogOpen(false)
      return
    }

    setSubmittingDelete(true)

    try {
      await deleteAction.onConfirm(row, rowIndex)
      setDeleteDialogOpen(false)
      onActionComplete()
    } catch (deleteError) {
      console.error("DataTable delete action failed:", deleteError)
    } finally {
      setSubmittingDelete(false)
    }
  }

  const handleMoreItemSelect = async (item: DataTableRowActionItem<T>) => {
    try {
      await item.onClick?.(row, rowIndex)
      onActionComplete()
    } catch (actionError) {
      console.error("DataTable row action failed:", actionError)
    }
  }

  return (
    <>
      <div className="flex items-center justify-end gap-1.5">
        {editAction ? (
          <IconButton
            label={editAction.label ?? resolvedEditLabel}
            onClick={() => setEditDialogOpen(true)}
          >
            <Pencil aria-hidden="true" className="size-4" />
          </IconButton>
        ) : null}

        {deleteAction ? (
          <IconButton
            label={deleteAction.label ?? resolvedDeleteLabel}
            variant="destructive"
            onClick={() => setDeleteDialogOpen(true)}
          >
            <Trash2 aria-hidden="true" className="size-4" />
          </IconButton>
        ) : null}

        {hasMoreActions ? (
          <DropdownMenu>
            <DropdownMenuTrigger
              className="inline-flex size-8 items-center justify-center rounded-md text-muted-foreground transition hover:bg-accent hover:text-accent-foreground disabled:pointer-events-none disabled:opacity-50"
              aria-label={rowActions.moreLabel ?? resolvedMoreLabel}
            >
              <MoreHorizontal aria-hidden="true" className="size-4.5" />
              <span className="sr-only">{rowActions.moreLabel ?? resolvedMoreLabel}</span>
            </DropdownMenuTrigger>
            <DropdownMenuContent
              mode="primitive"
              align="end"
              sideOffset={8}
              className="z-[2000] min-w-36 overflow-hidden rounded-lg border border-border bg-popover p-1 text-popover-foreground shadow-lg outline-hidden"
            >
              {moreItems.map((item) => (
                <DropdownMenuItem
                  key={item.key}
                  mode="primitive"
                  variant={item.variant ?? "default"}
                  className="flex min-h-8 cursor-default items-center gap-2 rounded-md px-2 py-1.5 text-sm leading-none outline-hidden transition focus:bg-accent focus:text-accent-foreground data-[variant=destructive]:text-destructive data-[variant=destructive]:focus:bg-destructive/10 data-disabled:pointer-events-none data-disabled:opacity-50"
                  disabled={resolveRowActionDisabled(
                    item.disabled,
                    row,
                    rowIndex
                  )}
                  onSelect={() => {
                    void handleMoreItemSelect(item)
                  }}
                >
                  {resolveRowActionContent(item.label, row, rowIndex)}
                </DropdownMenuItem>
              ))}
            </DropdownMenuContent>
          </DropdownMenu>
        ) : null}
      </div>

      {editAction && editDialogOpen ? (
        <Dialog open={editDialogOpen} onOpenChange={setEditDialogOpen}>
          <DialogContent>
            <DialogHeader>
              <DialogTitle>
                {resolveRowActionContent(editAction.title, row, rowIndex) ??
                  editAction.label ??
                  resolvedEditLabel}
              </DialogTitle>
              {editAction.description ? (
                <DialogDescription>
                  {resolveRowActionContent(
                    editAction.description,
                    row,
                    rowIndex
                  )}
                </DialogDescription>
              ) : null}
            </DialogHeader>

            {editAction.renderContent?.({
              row,
              rowIndex,
              close: () => setEditDialogOpen(false),
            }) ?? null}

            <DialogFooter>
              <Button
                type="button"
                variant="outline"
                onClick={() => setEditDialogOpen(false)}
                disabled={submittingEdit}
              >
                {editAction.cancelLabel ?? resolvedCancelLabel}
              </Button>
              <Button
                type="button"
                onClick={() => {
                  void handleEditConfirm()
                }}
                disabled={submittingEdit}
              >
                {editAction.confirmLabel ?? resolvedSaveLabel}
              </Button>
            </DialogFooter>
          </DialogContent>
        </Dialog>
      ) : null}

      {deleteAction && deleteDialogOpen ? (
        <Dialog open={deleteDialogOpen} onOpenChange={setDeleteDialogOpen}>
          <DialogContent>
            <DialogHeader>
              <DialogTitle>
                {resolveRowActionContent(deleteAction.title, row, rowIndex) ??
                  deleteAction.label ??
                  resolvedDeleteDialogTitle}
              </DialogTitle>
              {deleteAction.description || resolvedDeleteDialogDescription ? (
                <DialogDescription>
                  {resolveRowActionContent(
                    deleteAction.description,
                    row,
                    rowIndex
                  ) ?? resolvedDeleteDialogDescription}
                </DialogDescription>
              ) : null}
            </DialogHeader>

            <DialogFooter>
              <Button
                type="button"
                variant="outline"
                onClick={() => setDeleteDialogOpen(false)}
                disabled={submittingDelete}
              >
                {deleteAction.cancelLabel ?? resolvedCancelLabel}
              </Button>
              <Button
                type="button"
                variant="destructive"
                onClick={() => {
                  void handleDeleteConfirm()
                }}
                disabled={submittingDelete}
              >
                {deleteAction.confirmLabel ?? resolvedConfirmDeleteLabel}
              </Button>
            </DialogFooter>
          </DialogContent>
        </Dialog>
      ) : null}
    </>
  )
}
