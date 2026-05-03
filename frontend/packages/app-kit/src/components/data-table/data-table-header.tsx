import { Plus, RefreshCw, RotateCcw, SquarePen, Trash } from "lucide-react"
import {
  Tooltip,
  TooltipContent,
  TooltipTrigger,
} from "../tooltip"
import type { ReactNode } from "react"

import type {
  DataTableBuiltInQueryField,
  DataTableQueryField,
  DataTableRenderedQueryField,
} from "./data-table.types"
import { getQueryFieldLayoutStyle } from "./data-table.utils"

function getCustomQueryFieldLayoutStyle(
  field: DataTableRenderedQueryField<object>
) {
  if (field.type === "select") {
    return {
      flex: "0 0 auto",
      maxWidth: "none",
    }
  }

  if (field.type === "date-range") {
    return {
      flex: "0 1 220px",
      maxWidth: "240px",
    }
  }

  if (field.type === "scoped-date-range") {
    return {
      flex: "0 0 auto",
      maxWidth: "none",
    }
  }

  return getQueryFieldLayoutStyle(field)
}

function QueryFieldItem<TQuery extends object>({
  field,
  renderQueryFieldControl,
  compact = false,
}: {
  field: DataTableRenderedQueryField<TQuery>
  renderQueryFieldControl: (
    field: DataTableRenderedQueryField<TQuery>
  ) => ReactNode
  compact?: boolean
}) {
  return (
    <label
      key={field.key}
      className="flex flex-col gap-1"
      style={
        compact
          ? getCustomQueryFieldLayoutStyle(
              field as DataTableRenderedQueryField<object>
            )
          : getQueryFieldLayoutStyle(
              field as DataTableRenderedQueryField<object>
            )
      }
    >
      {renderQueryFieldControl(field)}
      {field.description ? (
        <span className="text-xs leading-4 text-muted-foreground">
          {field.description}
        </span>
      ) : null}
    </label>
  )
}

function IconToolButton({
  icon,
  label,
  disabled = false,
  onClick,
  className,
}: {
  icon: ReactNode
  label: ReactNode
  disabled?: boolean
  onClick: () => void
  className: string
}) {
  return (
    <Tooltip>
      <TooltipTrigger>
        <button type="button" disabled={disabled} onClick={onClick} className={className}>
          {icon}
          <span className="sr-only">{label}</span>
        </button>
      </TooltipTrigger>
      <TooltipContent>{label}</TooltipContent>
    </Tooltip>
  )
}

function QueryToolGroup({
  loading,
  resolvedResetLabel,
  resolvedRefreshLabel,
  onResetQuery,
  onRetry,
}: {
  loading: boolean
  resolvedResetLabel: ReactNode
  resolvedRefreshLabel: ReactNode
  onResetQuery: () => void
  onRetry: () => void
}) {
  return (
    <div className="flex shrink-0 items-center gap-1">
      <IconToolButton
        icon={<RotateCcw aria-hidden="true" className="size-4.5" />}
        label={resolvedResetLabel}
        disabled={loading}
        onClick={onResetQuery}
        className="inline-flex size-8 items-center justify-center rounded-md text-muted-foreground transition hover:bg-accent hover:text-accent-foreground disabled:pointer-events-none disabled:opacity-50"
      />
      <IconToolButton
        icon={<RefreshCw aria-hidden="true" className="size-4.5" />}
        label={resolvedRefreshLabel}
        disabled={loading}
        onClick={onRetry}
        className="inline-flex size-8 items-center justify-center rounded-md text-muted-foreground transition hover:bg-accent hover:text-accent-foreground disabled:pointer-events-none disabled:opacity-50"
      />
    </div>
  )
}

function ToolbarActions({
  insert,
  bulkDelete,
  bulkUpdate,
  toolbarActions,
  rowSelectionEnabled,
  deleting,
  selectedRowKeysCount,
  resolvedInsertLabel,
  resolvedBulkUpdateLabel,
  resolvedBulkDeleteLabel,
  onOpenInsert,
  onOpenBulkUpdate,
  onOpenBulkDelete,
}: {
  insert: false | { disabled?: boolean; label?: ReactNode }
  bulkDelete: false | { label?: ReactNode }
  bulkUpdate: false | { label?: ReactNode }
  toolbarActions?: ReactNode
  rowSelectionEnabled: boolean
  deleting: boolean
  selectedRowKeysCount: number
  resolvedInsertLabel: ReactNode
  resolvedBulkUpdateLabel: (count: number) => ReactNode
  resolvedBulkDeleteLabel: (count: number) => ReactNode
  onOpenInsert: () => void
  onOpenBulkUpdate: () => void
  onOpenBulkDelete: () => void
}) {
  if (
    insert === false &&
    bulkDelete === false &&
    bulkUpdate === false &&
    !toolbarActions
  ) {
    return null
  }

  const insertTooltip =
    insert !== false ? insert.label ?? resolvedInsertLabel : null
  const bulkUpdateTooltip =
    bulkUpdate !== false
      ? bulkUpdate.label ?? resolvedBulkUpdateLabel(selectedRowKeysCount)
      : null
  const bulkDeleteTooltip =
    bulkDelete !== false
      ? bulkDelete.label ?? resolvedBulkDeleteLabel(selectedRowKeysCount)
      : null
  return (
    <div className="flex-none self-start">
      <div className="flex items-center justify-end">
        <div className="flex flex-col items-center justify-center gap-1.5 sm:flex-row sm:items-center sm:justify-end sm:gap-1">
          {insert !== false ? (
            <IconToolButton
              icon={<Plus aria-hidden="true" className="size-4" />}
              label={insertTooltip}
              disabled={insert.disabled === true}
              onClick={onOpenInsert}
              className="inline-flex size-8 items-center justify-center rounded-md bg-primary text-primary-foreground transition hover:opacity-90 disabled:pointer-events-none disabled:opacity-50"
            />
          ) : null}
          {bulkUpdate !== false ? (
            <IconToolButton
              icon={<SquarePen aria-hidden="true" className="size-4" />}
              label={bulkUpdateTooltip}
              disabled={selectedRowKeysCount === 0}
              onClick={onOpenBulkUpdate}
              className="inline-flex size-8 items-center justify-center rounded-md bg-secondary text-secondary-foreground transition hover:opacity-90 disabled:pointer-events-none disabled:opacity-50"
            />
          ) : null}
          {bulkDelete !== false && rowSelectionEnabled ? (
            <IconToolButton
              icon={<Trash aria-hidden="true" className="size-4" />}
              label={bulkDeleteTooltip}
              disabled={selectedRowKeysCount === 0 || deleting}
              onClick={onOpenBulkDelete}
              className="inline-flex size-8 items-center justify-center rounded-md bg-destructive text-destructive-foreground transition hover:opacity-90 disabled:pointer-events-none disabled:opacity-50"
            />
          ) : null}
          {toolbarActions}
        </div>
      </div>
    </div>
  )
}

export function DataTableHeader<TQuery extends object>({
  hasAnyQueryFields,
  leadingBuiltInSearchField,
  trailingBuiltInQueryFields,
  queryFields,
  auditQueryField,
  queryTools,
  loading,
  insert,
  bulkDelete,
  bulkUpdate,
  toolbarActions,
  rowSelectionEnabled,
  deleting,
  selectedRowKeysCount,
  resolvedResetLabel,
  resolvedRefreshLabel,
  resolvedInsertLabel,
  resolvedBulkUpdateLabel,
  resolvedBulkDeleteLabel,
  renderQueryFieldControl,
  onResetQuery,
  onRetry,
  onOpenInsert,
  onOpenBulkUpdate,
  onOpenBulkDelete,
}: {
  hasAnyQueryFields: boolean
  leadingBuiltInSearchField: DataTableBuiltInQueryField<TQuery> | null
  trailingBuiltInQueryFields: readonly DataTableBuiltInQueryField<TQuery>[]
  queryFields: readonly DataTableQueryField<TQuery>[]
  auditQueryField: DataTableRenderedQueryField<TQuery> | null
  queryTools: boolean
  loading: boolean
  insert: false | { disabled?: boolean; label?: ReactNode }
  bulkDelete: false | { label?: ReactNode }
  bulkUpdate: false | { label?: ReactNode }
  toolbarActions?: ReactNode
  rowSelectionEnabled: boolean
  deleting: boolean
  selectedRowKeysCount: number
  resolvedResetLabel: ReactNode
  resolvedRefreshLabel: ReactNode
  resolvedInsertLabel: ReactNode
  resolvedBulkUpdateLabel: (count: number) => ReactNode
  resolvedBulkDeleteLabel: (count: number) => ReactNode
  renderQueryFieldControl: (
    field: DataTableRenderedQueryField<TQuery>
  ) => ReactNode
  onResetQuery: () => void
  onRetry: () => void
  onOpenInsert: () => void
  onOpenBulkUpdate: () => void
  onOpenBulkDelete: () => void
}) {
  return hasAnyQueryFields ? (
    <div className="flex min-w-0 items-start gap-4">
      <div className="min-w-0 flex-1 overflow-x-auto pb-2">
        <div className="flex w-max items-center gap-1.5 sm:gap-2.5">
          {leadingBuiltInSearchField ? (
            <QueryFieldItem
              key={leadingBuiltInSearchField.key}
              field={leadingBuiltInSearchField}
              renderQueryFieldControl={renderQueryFieldControl}
            />
          ) : null}

          {queryTools ? (
            <QueryToolGroup
              loading={loading}
              resolvedResetLabel={resolvedResetLabel}
              resolvedRefreshLabel={resolvedRefreshLabel}
              onResetQuery={onResetQuery}
              onRetry={onRetry}
            />
          ) : null}

          {trailingBuiltInQueryFields.map((field) => (
            <QueryFieldItem
              key={field.key}
              field={field}
              renderQueryFieldControl={renderQueryFieldControl}
            />
          ))}

          {auditQueryField ? (
            <QueryFieldItem
              key={auditQueryField.key}
              field={auditQueryField}
              renderQueryFieldControl={renderQueryFieldControl}
              compact
            />
          ) : null}

          {queryFields.map((field) => (
            <QueryFieldItem
              key={field.key}
              field={field}
              renderQueryFieldControl={renderQueryFieldControl}
              compact
            />
          ))}
        </div>
      </div>

      <ToolbarActions
        insert={insert}
        bulkDelete={bulkDelete}
        bulkUpdate={bulkUpdate}
        toolbarActions={toolbarActions}
        rowSelectionEnabled={rowSelectionEnabled}
        deleting={deleting}
        selectedRowKeysCount={selectedRowKeysCount}
        resolvedInsertLabel={resolvedInsertLabel}
        resolvedBulkUpdateLabel={resolvedBulkUpdateLabel}
        resolvedBulkDeleteLabel={resolvedBulkDeleteLabel}
        onOpenInsert={onOpenInsert}
        onOpenBulkUpdate={onOpenBulkUpdate}
        onOpenBulkDelete={onOpenBulkDelete}
      />
    </div>
  ) : (
    <div className="flex items-start justify-between gap-4">
      <div className="flex flex-1 items-center gap-3">
        {queryTools ? (
          <QueryToolGroup
            loading={loading}
            resolvedResetLabel={resolvedResetLabel}
            resolvedRefreshLabel={resolvedRefreshLabel}
            onResetQuery={onResetQuery}
            onRetry={onRetry}
          />
        ) : null}
      </div>

      <ToolbarActions
        insert={insert}
        bulkDelete={bulkDelete}
        bulkUpdate={bulkUpdate}
        toolbarActions={toolbarActions}
        rowSelectionEnabled={rowSelectionEnabled}
        deleting={deleting}
        selectedRowKeysCount={selectedRowKeysCount}
        resolvedInsertLabel={resolvedInsertLabel}
        resolvedBulkUpdateLabel={resolvedBulkUpdateLabel}
        resolvedBulkDeleteLabel={resolvedBulkDeleteLabel}
        onOpenInsert={onOpenInsert}
        onOpenBulkUpdate={onOpenBulkUpdate}
        onOpenBulkDelete={onOpenBulkDelete}
      />
    </div>
  )
}
