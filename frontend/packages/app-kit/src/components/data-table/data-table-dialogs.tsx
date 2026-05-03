import {
  AdvancedSelect,
  Button,
  Dialog,
  DialogContent,
  DialogDescription,
  DialogFooter,
  DialogHeader,
  DialogTitle,
  Input,
} from "@workspace/ui-components"
import type { Key, ReactNode } from "react"

import type {
  DataTableBulkDeleteConfig,
  DataTableBulkUpdateConfig,
  DataTableBulkUpdateField,
  DataTableInsertActionConfig,
} from "./data-table.types"

function renderBulkUpdateControl({
  activeBulkUpdateField,
  bulkUpdateValue,
  updating,
  setBulkUpdateValue,
}: {
  activeBulkUpdateField: DataTableBulkUpdateField | null
  bulkUpdateValue: unknown
  updating: boolean
  setBulkUpdateValue: (value: string) => void
}) {
  if (activeBulkUpdateField === null) return null

  if (activeBulkUpdateField.type === "text") {
    return (
      <Input
        value={typeof bulkUpdateValue === "string" ? bulkUpdateValue : ""}
        onValueChange={setBulkUpdateValue}
        placeholder={activeBulkUpdateField.placeholder}
        disabled={updating}
        type={activeBulkUpdateField.inputType}
      />
    )
  }

  if (activeBulkUpdateField.type === "select") {
    return (
      <AdvancedSelect
        value={typeof bulkUpdateValue === "string" ? bulkUpdateValue : ""}
        onValueChange={setBulkUpdateValue}
        list={activeBulkUpdateField.options.map((option) => ({
          label: option.label,
          value: option.value,
          disabled: option.disabled,
        }))}
        placeholder={activeBulkUpdateField.placeholder}
        disabled={updating}
      />
    )
  }

  return null
}

export function DataTableDialogs<T>({
  insert,
  insertDialogOpen,
  setInsertDialogOpen,
  submittingInsert,
  handleInsertConfirm,
  bulkDelete,
  bulkDeleteDialogOpen,
  setBulkDeleteDialogOpen,
  deleting,
  selectedRowKeys,
  handleBulkDeleteAndClose,
  bulkUpdate,
  bulkUpdateDialogOpen,
  setBulkUpdateDialogOpen,
  updating,
  activeBulkUpdateField,
  bulkUpdateValue,
  setBulkUpdateFieldKey,
  setBulkUpdateValue,
  availableBulkUpdateFields,
  handleBulkUpdateSubmit,
  resolvedInsertLabel,
  resolvedCancelLabel,
  resolvedSaveLabel,
  resolvedDeleteDialogTitle,
  resolvedBulkDeleteDialogDescription,
  resolvedConfirmDeleteLabel,
  resolvedBulkUpdateTitle,
  resolvedBulkUpdateDescription,
  resolvedBulkUpdateFieldLabel,
  resolvedBulkUpdateValueLabel,
  resolvedBulkUpdateCancelLabel,
  resolvedBulkUpdateApplyLabel,
}: {
  insert: false | DataTableInsertActionConfig
  insertDialogOpen: boolean
  setInsertDialogOpen: (open: boolean) => void
  submittingInsert: boolean
  handleInsertConfirm: () => Promise<void>
  bulkDelete: false | DataTableBulkDeleteConfig<T>
  bulkDeleteDialogOpen: boolean
  setBulkDeleteDialogOpen: (open: boolean) => void
  deleting: boolean
  selectedRowKeys: Key[]
  handleBulkDeleteAndClose: () => Promise<void>
  bulkUpdate: false | DataTableBulkUpdateConfig<T>
  bulkUpdateDialogOpen: boolean
  setBulkUpdateDialogOpen: (open: boolean) => void
  updating: boolean
  activeBulkUpdateField: DataTableBulkUpdateField | null
  bulkUpdateValue: unknown
  setBulkUpdateFieldKey: (value: string) => void
  setBulkUpdateValue: (value: string) => void
  availableBulkUpdateFields: readonly DataTableBulkUpdateField[]
  handleBulkUpdateSubmit: () => Promise<void>
  resolvedInsertLabel: ReactNode
  resolvedCancelLabel: ReactNode
  resolvedSaveLabel: ReactNode
  resolvedDeleteDialogTitle: ReactNode
  resolvedBulkDeleteDialogDescription: (count: number) => ReactNode
  resolvedConfirmDeleteLabel: ReactNode
  resolvedBulkUpdateTitle: ReactNode
  resolvedBulkUpdateDescription: (count: number) => ReactNode
  resolvedBulkUpdateFieldLabel: ReactNode
  resolvedBulkUpdateValueLabel: ReactNode
  resolvedBulkUpdateCancelLabel: ReactNode
  resolvedBulkUpdateApplyLabel: ReactNode
}) {
  return (
    <>
      {bulkUpdate !== false && bulkUpdateDialogOpen ? (
        <Dialog
          open={bulkUpdateDialogOpen}
          onOpenChange={setBulkUpdateDialogOpen}
        >
          <DialogContent>
            <DialogHeader>
              <DialogTitle>
                {bulkUpdate.title ?? resolvedBulkUpdateTitle}
              </DialogTitle>
              <DialogDescription>
                {bulkUpdate.description ??
                  resolvedBulkUpdateDescription(selectedRowKeys.length)}
              </DialogDescription>
            </DialogHeader>

            <div className="flex flex-col gap-4">
              <div className="flex flex-col gap-2">
                <span className="text-sm font-medium">
                  {resolvedBulkUpdateFieldLabel}
                </span>
                <AdvancedSelect
                  value={activeBulkUpdateField?.key ?? ""}
                  onValueChange={(value) => {
                    setBulkUpdateFieldKey(value)
                    setBulkUpdateValue("")
                  }}
                  list={availableBulkUpdateFields.map((field) => ({
                    label: field.label,
                    value: field.key,
                  }))}
                  disabled={updating}
                />
              </div>

              <div className="flex flex-col gap-2">
                <span className="text-sm font-medium">
                  {resolvedBulkUpdateValueLabel}
                </span>
                {renderBulkUpdateControl({
                  activeBulkUpdateField,
                  bulkUpdateValue,
                  updating,
                  setBulkUpdateValue,
                })}
                {activeBulkUpdateField?.description ? (
                  <span className="text-xs text-muted-foreground">
                    {activeBulkUpdateField.description}
                  </span>
                ) : null}
              </div>
            </div>

            <DialogFooter>
              <Button
                type="button"
                variant="outline"
                disabled={updating}
                onClick={() => setBulkUpdateDialogOpen(false)}
              >
                {resolvedBulkUpdateCancelLabel}
              </Button>
              <Button
                type="button"
                disabled={activeBulkUpdateField === null || updating}
                onClick={() => {
                  void handleBulkUpdateSubmit()
                }}
              >
                {resolvedBulkUpdateApplyLabel}
              </Button>
            </DialogFooter>
          </DialogContent>
        </Dialog>
      ) : null}

      {bulkDelete !== false && bulkDeleteDialogOpen ? (
        <Dialog
          open={bulkDeleteDialogOpen}
          onOpenChange={setBulkDeleteDialogOpen}
        >
          <DialogContent>
            <DialogHeader>
              <DialogTitle>
                {bulkDelete.title ?? resolvedDeleteDialogTitle}
              </DialogTitle>
              <DialogDescription>
                {(typeof bulkDelete.description === "function"
                  ? bulkDelete.description(selectedRowKeys.length)
                  : bulkDelete.description) ??
                  resolvedBulkDeleteDialogDescription(selectedRowKeys.length)}
              </DialogDescription>
            </DialogHeader>

            <DialogFooter>
              <Button
                type="button"
                variant="outline"
                disabled={deleting}
                onClick={() => setBulkDeleteDialogOpen(false)}
              >
                {resolvedCancelLabel}
              </Button>
              <Button
                type="button"
                variant="destructive"
                disabled={deleting || selectedRowKeys.length === 0}
                onClick={() => {
                  void handleBulkDeleteAndClose()
                }}
              >
                {resolvedConfirmDeleteLabel}
              </Button>
            </DialogFooter>
          </DialogContent>
        </Dialog>
      ) : null}

      {insert !== false && insertDialogOpen ? (
        <Dialog open={insertDialogOpen} onOpenChange={setInsertDialogOpen}>
          <DialogContent>
            <DialogHeader>
              <DialogTitle>
                {insert.title ?? insert.label ?? resolvedInsertLabel}
              </DialogTitle>
              {insert.description ? (
                <DialogDescription>{insert.description}</DialogDescription>
              ) : null}
            </DialogHeader>

            {insert.renderContent?.({
              close: () => setInsertDialogOpen(false),
            }) ?? null}

            <DialogFooter>
              <Button
                type="button"
                variant="outline"
                disabled={submittingInsert}
                onClick={() => setInsertDialogOpen(false)}
              >
                {insert.cancelLabel ?? resolvedCancelLabel}
              </Button>
              <Button
                type="button"
                disabled={submittingInsert}
                onClick={() => {
                  void handleInsertConfirm()
                }}
              >
                {insert.confirmLabel ?? resolvedSaveLabel}
              </Button>
            </DialogFooter>
          </DialogContent>
        </Dialog>
      ) : null}
    </>
  )
}
