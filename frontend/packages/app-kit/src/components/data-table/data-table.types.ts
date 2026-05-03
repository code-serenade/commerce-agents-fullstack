import type { Key, ReactNode } from "react"

import type { DateRangeValue } from "../date-time-picker"

export interface DataTableFetchResult<T> {
  items: T[]
  total: number
}

export interface DataTableColumn<T> {
  key: string
  header: ReactNode
  renderCell: (row: T, rowIndex: number) => ReactNode
  width?: number | string
  sticky?: "left" | "right"
  sortable?: boolean
}

export type DataTableSortDirection = "asc" | "desc"

export interface DataTableSortState {
  columnKey: string
  direction: DataTableSortDirection
}

export interface DataTableSelectOption {
  label: string
  value: string
  disabled?: boolean
}

export interface DataTableQueryFieldBase<TQuery> {
  key: keyof TQuery & string
  label?: ReactNode
  description?: ReactNode
  disabled?: boolean
}

export interface DataTableTextQueryField<
  TQuery,
> extends DataTableQueryFieldBase<TQuery> {
  type: "text"
  placeholder?: string
  inputType?: "text" | "search" | "email" | "number" | "tel" | "url"
}

export interface DataTableSelectQueryField<
  TQuery,
> extends DataTableQueryFieldBase<TQuery> {
  type: "select"
  placeholder?: string
  options: readonly DataTableSelectOption[]
}

export interface DataTableSearchQueryField<
  TQuery,
> extends DataTableQueryFieldBase<TQuery> {
  type: "search"
  placeholder?: string
}

export interface DataTableDateRangeQueryField<
  TQuery,
> extends DataTableQueryFieldBase<TQuery> {
  type: "date-range"
  placeholder?: string
}

export interface DataTableScopedDateRangeQueryField<
  TQuery,
> extends DataTableQueryFieldBase<TQuery> {
  type: "scoped-date-range"
  scopeKey: keyof TQuery & string
  placeholder?: string
  scopePlaceholder?: string
  rangePlaceholder?: string
  options: readonly DataTableSelectOption[]
}

export type DataTableBuiltInQueryField<TQuery> =
  | DataTableSearchQueryField<TQuery>
  | DataTableSelectQueryField<TQuery>

export type DataTableQueryField<TQuery> =
  | DataTableTextQueryField<TQuery>
  | DataTableSearchQueryField<TQuery>
  | DataTableSelectQueryField<TQuery>

export type DataTableRenderedQueryField<TQuery> =
  | DataTableQueryField<TQuery>
  | DataTableDateRangeQueryField<TQuery>
  | DataTableScopedDateRangeQueryField<TQuery>

export interface DataTableFetchParams<TQuery = object> {
  page: number
  pageSize: number
  signal: AbortSignal
  query: TQuery
  sort: DataTableSortState | null
}

export interface DataTableLocaleText {
  emptyText?: ReactNode
  errorText?: ReactNode
  loadingText?: ReactNode
  refreshLabel?: string
  resetLabel?: string
  totalLabel?: ReactNode
  insertLabel?: ReactNode
  actionsLabel?: ReactNode
  editLabel?: ReactNode
  deleteLabel?: ReactNode
  moreLabel?: string
  cancelLabel?: ReactNode
  saveLabel?: ReactNode
  confirmDeleteLabel?: ReactNode
  deleteDialogTitle?: ReactNode
  deleteDialogDescription?: ReactNode
  bulkDeleteDialogDescription?: (count: number) => ReactNode
  sortAscendingLabel?: string
  sortDescendingLabel?: string
  clearSortLabel?: string
  bulkDeleteLabel?: (count: number) => ReactNode
  bulkUpdateLabel?: (count: number) => ReactNode
  bulkUpdateTitle?: ReactNode
  bulkUpdateDescription?: (count: number) => ReactNode
  bulkUpdateFieldLabel?: ReactNode
  bulkUpdateValueLabel?: ReactNode
  bulkUpdateCancelLabel?: ReactNode
  bulkUpdateApplyLabel?: ReactNode
  auditEmptyText?: ReactNode
}

export type DataTableAuditColumnKey = "createdAt" | "updatedAt"

export interface DataTableSelectionContext<T> {
  clearSelection: () => void
  selectedRowKeys: Key[]
  selectedRows: T[]
}

export interface DataTableBulkDeleteConfig<T> {
  label?: ReactNode
  title?: ReactNode
  description?: ReactNode | ((count: number) => ReactNode)
  onDelete: (context: DataTableSelectionContext<T>) => Promise<void> | void
}

export interface DataTableInsertActionConfig {
  label?: ReactNode
  disabled?: boolean
  title?: ReactNode
  description?: ReactNode
  cancelLabel?: ReactNode
  confirmLabel?: ReactNode
  renderContent?: (context: { close: () => void }) => ReactNode
  onConfirm?: () => Promise<void> | void
}

export interface DataTableBulkUpdateFieldBase {
  key: string
  label: ReactNode
  description?: ReactNode
  disabled?: boolean
}

export interface DataTableBulkUpdateTextField
  extends DataTableBulkUpdateFieldBase {
  type: "text"
  placeholder?: string
  inputType?: "text" | "search" | "email" | "number" | "tel" | "url"
}

export interface DataTableBulkUpdateSelectField
  extends DataTableBulkUpdateFieldBase {
  type: "select"
  placeholder?: string
  options: readonly DataTableSelectOption[]
}

export type DataTableBulkUpdateField =
  | DataTableBulkUpdateTextField
  | DataTableBulkUpdateSelectField

export interface DataTableBulkUpdateSubmitContext<
  T,
> extends DataTableSelectionContext<T> {
  fieldKey: string
  value: unknown
}

export interface DataTableBulkUpdateConfig<T> {
  label?: ReactNode
  title?: ReactNode
  description?: ReactNode
  fields: readonly DataTableBulkUpdateField[]
  onSubmit: (
    context: DataTableBulkUpdateSubmitContext<T>
  ) => Promise<void> | void
}

export interface DataTableRowActionItem<T> {
  key: string
  label: ReactNode | ((row: T, rowIndex: number) => ReactNode)
  variant?: "default" | "destructive"
  disabled?: boolean | ((row: T, rowIndex: number) => boolean)
  onClick?: (row: T, rowIndex: number) => Promise<void> | void
}

export interface DataTableEditActionConfig<T> {
  label?: ReactNode
  title?: ReactNode | ((row: T, rowIndex: number) => ReactNode)
  description?: ReactNode | ((row: T, rowIndex: number) => ReactNode)
  cancelLabel?: ReactNode
  confirmLabel?: ReactNode
  renderContent?: (context: {
    row: T
    rowIndex: number
    close: () => void
  }) => ReactNode
  onConfirm?: (row: T, rowIndex: number) => Promise<void> | void
}

export interface DataTableDeleteActionConfig<T> {
  label?: ReactNode
  title?: ReactNode | ((row: T, rowIndex: number) => ReactNode)
  description?: ReactNode | ((row: T, rowIndex: number) => ReactNode)
  cancelLabel?: ReactNode
  confirmLabel?: ReactNode
  onConfirm?: (row: T, rowIndex: number) => Promise<void> | void
}

export interface DataTableRowActionsConfig<T> {
  header?: ReactNode
  columnWidth?: number | string
  sticky?: "left" | "right" | false
  moreLabel?: string
  edit?: false | DataTableEditActionConfig<T>
  delete?: false | DataTableDeleteActionConfig<T>
  moreItems?: readonly DataTableRowActionItem<T>[]
}

export interface DataTableQueryConfig<TQuery extends object> {
  builtInFields?: readonly DataTableBuiltInQueryField<TQuery>[]
  fields?: readonly DataTableQueryField<TQuery>[]
  audit?: boolean
  tools?: boolean
}

export interface DataTableStickyColumnsConfig {
  left?: number
  right?: number
}

export interface DataTableProps<T, TQuery extends object = object> {
  columns: readonly DataTableColumn<T>[]
  auditColumns?: readonly DataTableAuditColumnKey[]
  stickyColumns?: DataTableStickyColumnsConfig
  fetchData: (
    params: DataTableFetchParams<TQuery>
  ) => Promise<DataTableFetchResult<T>>
  getRowId: (row: T, rowIndex: number) => Key
  caption?: ReactNode
  renderEmpty?: () => ReactNode
  renderError?: (error: unknown, retry: () => void) => ReactNode
  renderLoading?: () => ReactNode
  initialPage?: number
  initialPageSize?: number
  pageSizeOptions?: readonly number[]
  onError?: (error: unknown) => void
  initialQuery?: TQuery
  query?: DataTableQueryConfig<TQuery>
  toolbarActions?: ReactNode
  insert?: false | DataTableInsertActionConfig
  bulkDelete?: false | DataTableBulkDeleteConfig<T>
  bulkUpdate?: false | DataTableBulkUpdateConfig<T>
  rowActions?: false | DataTableRowActionsConfig<T>
  stripedRows?: boolean
  compactColumns?: boolean
  compactRows?: boolean
  fillWidth?: boolean
  height?: number | string
  initialSort?: DataTableSortState | null
  localeText?: DataTableLocaleText
}

export type { DateRangeValue }
