import {
  AdvancedSelect,
  Checkbox,
  Pagination,
  SearchInput,
  Spinner,
  Input,
} from "@workspace/ui-components"
import { TooltipProvider } from "../tooltip"
import {
  DataTableSurface,
  DataTableSurfaceBody,
  DataTableSurfaceCaption,
  DataTableSurfaceCell,
  DataTableSurfaceHead,
  DataTableSurfaceHeader,
  DataTableSurfaceRow,
} from "./data-table-surface"
import { getDataTableCopy, normalizeLanguage } from "@workspace/locales"
import { useTranslation } from "react-i18next"
import {
  Fragment,
  useCallback,
  useEffect,
  useLayoutEffect,
  useMemo,
  useRef,
  useState,
  type Key,
} from "react"

import { DateRangePicker } from "../date-time-picker"
import { DataTableDialogs } from "./data-table-dialogs"
import { DataTableHeader } from "./data-table-header"
import { DataTableRowActionsCell } from "./data-table-row-actions-cell"
import type {
  DataTableAuditColumnKey,
  DataTableBuiltInQueryField,
  DataTableColumn,
  DataTableProps,
  DataTableQueryConfig,
  DataTableRenderedQueryField,
  DataTableStickyColumnsConfig,
  DataTableSortState,
} from "./data-table.types"
import {
  DEFAULT_PAGE_SIZE_OPTIONS,
  DEFAULT_SELECTION_COLUMN_WIDTH,
  DEFAULT_STICKY_COLUMN_WIDTH,
  asDateRangeValue,
  asStringValue,
  createQueryState,
  isLeftStickyColumn,
  isRightStickyColumn,
  resolveColumnMinWidth,
  resolveColumnPixelWidth,
  resolveRowActionsColumnWidth,
  resolveTableHeight,
} from "./data-table.utils"

export type {
  DataTableBulkDeleteConfig,
  DataTableBulkUpdateConfig,
  DataTableBulkUpdateField,
  DataTableBulkUpdateSubmitContext,
  DataTableBuiltInQueryField,
  DataTableColumn,
  DataTableDeleteActionConfig,
  DataTableEditActionConfig,
  DataTableFetchParams,
  DataTableFetchResult,
  DataTableInsertActionConfig,
  DataTableLocaleText,
  DataTableProps,
  DataTableQueryConfig,
  DataTableQueryField,
  DataTableRowActionItem,
  DataTableRowActionsConfig,
  DataTableSearchQueryField,
  DataTableSelectOption,
  DataTableSelectionContext,
  DataTableStickyColumnsConfig,
  DataTableSortDirection,
  DataTableSortState,
  DataTableTextQueryField,
  DataTableAuditColumnKey,
} from "./data-table.types"

const DEFAULT_AUDIT_FIELD_KEY = "auditField"
const DEFAULT_AUDIT_RANGE_KEY = "auditRange"

function getAuditRawValue<T>(row: T, column: DataTableAuditColumnKey) {
  if (typeof row !== "object" || row === null) return undefined

  return (row as Record<DataTableAuditColumnKey, unknown>)[column]
}

function toAuditDate(value: unknown) {
  if (value instanceof Date) return Number.isNaN(value.getTime()) ? null : value
  if (typeof value !== "string" && typeof value !== "number") return null

  const date = new Date(value)
  return Number.isNaN(date.getTime()) ? null : date
}

function hasEmptyDateRange(value: unknown) {
  const range = asDateRangeValue(value)
  return !range?.from && !range?.to
}

function isScopedDateRangeScopeOnlyChange<TQuery extends object>({
  previousQuery,
  nextQuery,
  fields,
}: {
  previousQuery: TQuery | null
  nextQuery: TQuery
  fields: readonly DataTableRenderedQueryField<TQuery>[]
}) {
  if (!previousQuery) return false

  const scopedFields = fields.filter(
    (field) => field.type === "scoped-date-range"
  )

  if (scopedFields.length === 0) return false

  const changedKeys = new Set<string>()
  const previousRecord = previousQuery as Record<string, unknown>
  const nextRecord = nextQuery as Record<string, unknown>

  for (const key of new Set([
    ...Object.keys(previousRecord),
    ...Object.keys(nextRecord),
  ])) {
    if (!Object.is(previousRecord[key], nextRecord[key])) {
      changedKeys.add(key)
    }
  }

  if (changedKeys.size === 0) return false

  return [...changedKeys].every((key) =>
    scopedFields.some(
      (field) =>
        field.scopeKey === key && hasEmptyDateRange(nextRecord[field.key])
    )
  )
}

function formatDefaultAuditDateTime(value: Date, language: string) {
  return new Intl.DateTimeFormat(language === "zhCN" ? "zh-CN" : "en-US", {
    year: "numeric",
    month: "2-digit",
    day: "2-digit",
    hour: "2-digit",
    minute: "2-digit",
  }).format(value)
}

function areMeasuredWidthsEqual(
  current: readonly number[],
  next: readonly number[]
) {
  if (current.length !== next.length) return false

  return current.every((value, index) => value === next[index])
}

export function DataTable<T, TQuery extends object = object>({
  columns,
  auditColumns = [],
  stickyColumns,
  fetchData,
  getRowId,
  caption,
  renderEmpty,
  renderError,
  renderLoading,
  initialPage = 1,
  initialPageSize = 15,
  pageSizeOptions = DEFAULT_PAGE_SIZE_OPTIONS,
  onError,
  initialQuery,
  query,
  toolbarActions,
  insert = false,
  bulkDelete = false,
  bulkUpdate = false,
  rowActions = false,
  stripedRows = true,
  compactColumns = false,
  compactRows = false,
  fillWidth = true,
  height,
  initialSort = null,
  localeText,
}: DataTableProps<T, TQuery>) {
  const { i18n } = useTranslation()
  const language = normalizeLanguage(i18n.language)
  const copy = getDataTableCopy(language)
  const queryConfig: DataTableQueryConfig<TQuery> = query ?? {}
  const stickyColumnsConfig: DataTableStickyColumnsConfig = stickyColumns ?? {}
  const fixedLeftColumns = stickyColumnsConfig.left ?? 0
  const fixedRightColumns = stickyColumnsConfig.right ?? 0
  const builtInQueryFields = queryConfig.builtInFields ?? []
  const queryFields = queryConfig.fields ?? []
  const auditQuery = queryConfig.audit ?? false
  const queryTools = queryConfig.tools ?? true

  const [rows, setRows] = useState<T[]>([])
  const [page, setPage] = useState(initialPage)
  const [pageSize, setPageSize] = useState(initialPageSize)
  const [total, setTotal] = useState(0)
  const [loading, setLoading] = useState(false)
  const [error, setError] = useState<unknown>(null)
  const [reloadToken, setReloadToken] = useState(0)
  const [measuredColumnWidths, setMeasuredColumnWidths] = useState<number[]>([])
  const [surfaceTableWidth, setSurfaceTableWidth] = useState(0)
  const [sort, setSort] = useState<DataTableSortState | null>(initialSort)
  const [internalSelectedRowKeys, setInternalSelectedRowKeys] = useState<Key[]>(
    []
  )
  const [insertDialogOpen, setInsertDialogOpen] = useState(false)
  const [submittingInsert, setSubmittingInsert] = useState(false)
  const [bulkDeleteDialogOpen, setBulkDeleteDialogOpen] = useState(false)
  const [deleting, setDeleting] = useState(false)
  const [bulkUpdateDialogOpen, setBulkUpdateDialogOpen] = useState(false)
  const [bulkUpdateFieldKey, setBulkUpdateFieldKey] = useState("")
  const [bulkUpdateValue, setBulkUpdateValue] = useState<unknown>("")
  const [updating, setUpdating] = useState(false)
  const [draftQuery, setDraftQuery] = useState<TQuery>(() =>
    createQueryState(initialQuery)
  )
  const headerCellRefs = useRef<Array<HTMLTableCellElement | null>>([])
  const tableViewportRef = useRef<HTMLDivElement | null>(null)
  const previousFetchQueryRef = useRef<TQuery | null>(null)

  const rowSelectionEnabled = bulkDelete !== false || bulkUpdate !== false
  const selectionColumnWidth = DEFAULT_SELECTION_COLUMN_WIDTH
  const selectedRowKeys = internalSelectedRowKeys

  const safePageSizeOptions = useMemo(() => {
    const values = new Set<number>([...pageSizeOptions, initialPageSize])
    return Array.from(values).sort((left, right) => left - right)
  }, [initialPageSize, pageSizeOptions])

  const totalPages = Math.max(1, Math.ceil(total / pageSize))
  const hasRows = rows.length > 0
  const resolvedEmptyText = localeText?.emptyText ?? copy.emptyText
  const resolvedErrorText = localeText?.errorText ?? copy.errorText
  const resolvedLoadingText = localeText?.loadingText ?? copy.loadingText
  const resolvedRefreshLabel = localeText?.refreshLabel ?? copy.refreshLabel
  const resolvedResetLabel = localeText?.resetLabel ?? copy.resetLabel
  const resolvedTotalLabel = localeText?.totalLabel ?? copy.totalLabel
  const resolvedInsertLabel = localeText?.insertLabel ?? copy.insertLabel
  const resolvedActionsLabel = localeText?.actionsLabel ?? copy.actionsLabel
  const resolvedEditLabel = localeText?.editLabel ?? copy.editLabel
  const resolvedDeleteLabel = localeText?.deleteLabel ?? copy.deleteLabel
  const resolvedMoreLabel = localeText?.moreLabel ?? copy.moreLabel
  const resolvedCancelLabel = localeText?.cancelLabel ?? copy.cancelLabel
  const resolvedSaveLabel = localeText?.saveLabel ?? copy.saveLabel
  const resolvedConfirmDeleteLabel =
    localeText?.confirmDeleteLabel ?? copy.confirmDeleteLabel
  const resolvedDeleteDialogTitle =
    localeText?.deleteDialogTitle ?? copy.deleteDialogTitle
  const resolvedDeleteDialogDescription =
    localeText?.deleteDialogDescription ?? copy.deleteDialogDescription
  const resolvedBulkDeleteDialogDescription =
    localeText?.bulkDeleteDialogDescription ?? copy.bulkDeleteDialogDescription
  const resolvedSortAscendingLabel =
    localeText?.sortAscendingLabel ?? copy.sortAscendingLabel
  const resolvedSortDescendingLabel =
    localeText?.sortDescendingLabel ?? copy.sortDescendingLabel
  const resolvedClearSortLabel =
    localeText?.clearSortLabel ?? copy.clearSortLabel
  const resolvedBulkDeleteLabel =
    localeText?.bulkDeleteLabel ?? copy.bulkDeleteLabel
  const resolvedBulkUpdateLabel =
    localeText?.bulkUpdateLabel ?? copy.bulkUpdateLabel
  const resolvedBulkUpdateTitle =
    localeText?.bulkUpdateTitle ?? copy.bulkUpdateTitle
  const resolvedBulkUpdateDescription =
    localeText?.bulkUpdateDescription ?? copy.bulkUpdateDescription
  const resolvedBulkUpdateFieldLabel =
    localeText?.bulkUpdateFieldLabel ?? copy.bulkUpdateFieldLabel
  const resolvedBulkUpdateValueLabel =
    localeText?.bulkUpdateValueLabel ?? copy.bulkUpdateValueLabel
  const resolvedBulkUpdateCancelLabel =
    localeText?.bulkUpdateCancelLabel ?? copy.bulkUpdateCancelLabel
  const resolvedBulkUpdateApplyLabel =
    localeText?.bulkUpdateApplyLabel ?? copy.bulkUpdateApplyLabel
  const resolvedCreatedAtLabel = copy.createdAtLabel
  const resolvedUpdatedAtLabel = copy.updatedAtLabel
  const resolvedAuditQueryLabel = copy.auditQueryLabel
  const resolvedAuditFieldPlaceholder = copy.auditFieldPlaceholder
  const resolvedAuditRangePlaceholder = copy.auditRangePlaceholder
  const resolvedCreatedAtRangePlaceholder = copy.createdAtRangePlaceholder
  const resolvedUpdatedAtRangePlaceholder = copy.updatedAtRangePlaceholder
  const resolvedAuditEmptyText = localeText?.auditEmptyText ?? copy.auditEmptyText
  const auditColumnKeys = useMemo(() => [...auditColumns], [auditColumns])
  const auditQueryField = useMemo<DataTableRenderedQueryField<TQuery> | null>(
    () => {
      if (auditQuery !== true) return null

      if (auditColumnKeys.length === 0) return null

      const [firstColumn] = auditColumnKeys
      const getAuditQueryLabel = (column: DataTableAuditColumnKey) =>
        column === "createdAt"
          ? resolvedCreatedAtLabel
          : resolvedUpdatedAtLabel

      if (auditColumnKeys.length === 1) {
        return {
          key: firstColumn as keyof TQuery & string,
          type: "date-range",
          label: getAuditQueryLabel(firstColumn),
          placeholder:
            firstColumn === "createdAt"
              ? resolvedCreatedAtRangePlaceholder
              : resolvedUpdatedAtRangePlaceholder,
        }
      }

      return {
        key: DEFAULT_AUDIT_RANGE_KEY as keyof TQuery & string,
        type: "scoped-date-range",
        scopeKey: DEFAULT_AUDIT_FIELD_KEY as keyof TQuery & string,
        label: resolvedAuditQueryLabel,
        scopePlaceholder: resolvedAuditFieldPlaceholder,
        rangePlaceholder: resolvedAuditRangePlaceholder,
        options: auditColumnKeys.map((column) => ({
          label: String(getAuditQueryLabel(column)),
          value: column,
        })),
      }
    },
    [
      auditColumnKeys,
      auditQuery,
      resolvedAuditFieldPlaceholder,
      resolvedAuditQueryLabel,
      resolvedAuditRangePlaceholder,
      resolvedCreatedAtRangePlaceholder,
      resolvedCreatedAtLabel,
      resolvedUpdatedAtRangePlaceholder,
      resolvedUpdatedAtLabel,
    ]
  )
  const hasBuiltInQueryFields = builtInQueryFields.length > 0
  const hasUserQueryFields = queryFields.length > 0
  const hasAuditQueryField = auditQueryField !== null
  const hasAnyQueryFields =
    hasBuiltInQueryFields || hasUserQueryFields || hasAuditQueryField
  const leadingBuiltInSearchField =
    builtInQueryFields.find((field) => field.type === "search") ?? null
  const trailingBuiltInQueryFields = leadingBuiltInSearchField
    ? builtInQueryFields.filter((field) => field !== leadingBuiltInSearchField)
    : builtInQueryFields
  const allQueryFields = useMemo(
    () => [
      ...builtInQueryFields,
      ...(auditQueryField ? [auditQueryField] : []),
      ...queryFields,
    ],
    [auditQueryField, builtInQueryFields, queryFields]
  )

  const bulkUpdateFields = bulkUpdate !== false ? bulkUpdate.fields : []
  const availableBulkUpdateFields = useMemo(
    () => bulkUpdateFields.filter((field) => field.disabled !== true),
    [bulkUpdateFields]
  )

  const handleRetry = useCallback(() => {
    setReloadToken((current: number) => current + 1)
  }, [])

  const resolvedColumns = useMemo(() => {
    const auditResolvedColumns: DataTableColumn<T>[] = auditColumnKeys.map(
      (columnKey) => ({
        key: columnKey,
        header:
          columnKey === "createdAt"
            ? resolvedCreatedAtLabel
            : resolvedUpdatedAtLabel,
        width: 160,
        sortable: true,
        renderCell: (row) => {
          const date = toAuditDate(getAuditRawValue(row, columnKey))

          if (!date) {
            return (
              <span className="text-sm text-(--app-muted-text)">
                {resolvedAuditEmptyText}
              </span>
            )
          }

          return (
            <span className="whitespace-nowrap">
              {formatDefaultAuditDateTime(date, language)}
            </span>
          )
        },
      })
    )

    const baseColumns = [...columns, ...auditResolvedColumns]

    if (rowActions === false) {
      return baseColumns
    }

    const moreItems = rowActions.moreItems ?? []
    const actionColumn: DataTableColumn<T> = {
      key: "__actions__",
      header: rowActions.header ?? resolvedActionsLabel,
      width: resolveRowActionsColumnWidth(rowActions),
      sticky:
        rowActions.sticky === false
          ? undefined
          : (rowActions.sticky ?? "right"),
      renderCell: (row, rowIndex) => (
        <DataTableRowActionsCell
          row={row}
          rowIndex={rowIndex}
          rowActions={rowActions}
          moreItems={moreItems}
          resolvedEditLabel={resolvedEditLabel}
          resolvedDeleteLabel={resolvedDeleteLabel}
          resolvedMoreLabel={resolvedMoreLabel}
          resolvedCancelLabel={resolvedCancelLabel}
          resolvedSaveLabel={resolvedSaveLabel}
          resolvedConfirmDeleteLabel={resolvedConfirmDeleteLabel}
          resolvedDeleteDialogTitle={resolvedDeleteDialogTitle}
          resolvedDeleteDialogDescription={resolvedDeleteDialogDescription}
          onActionComplete={handleRetry}
        />
      ),
    }

    return [...baseColumns, actionColumn]
  }, [
    auditColumnKeys,
    columns,
    resolvedAuditEmptyText,
    resolvedActionsLabel,
    resolvedCreatedAtLabel,
    resolvedUpdatedAtLabel,
    resolvedCancelLabel,
    resolvedConfirmDeleteLabel,
    resolvedDeleteDialogDescription,
    resolvedDeleteDialogTitle,
    resolvedDeleteLabel,
    resolvedEditLabel,
    resolvedMoreLabel,
    resolvedSaveLabel,
    handleRetry,
    language,
    rowActions,
  ])

  useLayoutEffect(() => {
    const updateWidths = () => {
      const nextWidths = resolvedColumns.map((_, columnIndex) => {
        const cell = headerCellRefs.current[columnIndex]
        return cell?.getBoundingClientRect().width ?? 0
      })

      setMeasuredColumnWidths((current) =>
        areMeasuredWidthsEqual(current, nextWidths) ? current : nextWidths
      )

      const tableElement = tableViewportRef.current?.querySelector(
        "table[data-slot='table']"
      )
      const nextSurfaceWidth = Math.ceil(
        tableElement?.getBoundingClientRect().width ?? 0
      )

      setSurfaceTableWidth((current) =>
        current === nextSurfaceWidth ? current : nextSurfaceWidth
      )
    }

    updateWidths()

    if (typeof ResizeObserver === "undefined") {
      window.addEventListener("resize", updateWidths)
      return () => {
        window.removeEventListener("resize", updateWidths)
      }
    }

    const observer = new ResizeObserver(() => {
      updateWidths()
    })

    headerCellRefs.current.forEach((cell) => {
      if (cell) observer.observe(cell)
    })

    return () => {
      observer.disconnect()
    }
  }, [resolvedColumns])

  const stickyLeftOffsets = useMemo(() => {
    let currentLeft = rowSelectionEnabled ? selectionColumnWidth : 0

    return resolvedColumns.map((column, columnIndex) => {
      if (!isLeftStickyColumn(column, columnIndex, fixedLeftColumns)) {
        return undefined
      }

      const leftOffset = currentLeft
      const columnWidth =
        measuredColumnWidths[columnIndex] ||
        resolveColumnPixelWidth(column as DataTableColumn<object>) ||
        DEFAULT_STICKY_COLUMN_WIDTH
      currentLeft += columnWidth

      return leftOffset
    })
  }, [
    fixedLeftColumns,
    measuredColumnWidths,
    resolvedColumns,
    rowSelectionEnabled,
    selectionColumnWidth,
  ])

  const stickyRightOffsets = useMemo(() => {
    let currentRight = 0
    const offsets = Array<number | undefined>(resolvedColumns.length).fill(
      undefined
    )

    for (
      let columnIndex = resolvedColumns.length - 1;
      columnIndex >= 0;
      columnIndex -= 1
    ) {
      const column = resolvedColumns[columnIndex]

      if (
        !isRightStickyColumn(
          column,
          columnIndex,
          resolvedColumns.length,
          fixedLeftColumns,
          fixedRightColumns
        )
      ) {
        continue
      }

      const columnWidth =
        measuredColumnWidths[columnIndex] ||
        resolveColumnPixelWidth(column as DataTableColumn<object>) ||
        DEFAULT_STICKY_COLUMN_WIDTH
      offsets[columnIndex] = currentRight
      currentRight += columnWidth
    }

    return offsets
  }, [
    fixedLeftColumns,
    fixedRightColumns,
    measuredColumnWidths,
    resolvedColumns,
  ])

  const tableContentWidth = useMemo(() => {
    const baseWidth = resolvedColumns.reduce((totalWidth, column, columnIndex) => {
      const columnWidth =
        measuredColumnWidths[columnIndex] ||
        resolveColumnPixelWidth(column as DataTableColumn<object>) ||
        DEFAULT_STICKY_COLUMN_WIDTH

      return totalWidth + columnWidth
    }, 0)

    return baseWidth + (rowSelectionEnabled ? selectionColumnWidth : 0)
  }, [
    measuredColumnWidths,
    resolvedColumns,
    rowSelectionEnabled,
    selectionColumnWidth,
  ])
  const resolvedSurfaceWidth = surfaceTableWidth || tableContentWidth

  useEffect(() => {
    if (
      isScopedDateRangeScopeOnlyChange({
        previousQuery: previousFetchQueryRef.current,
        nextQuery: draftQuery,
        fields: allQueryFields,
      })
    ) {
      previousFetchQueryRef.current = draftQuery
      return
    }

    previousFetchQueryRef.current = draftQuery
    const controller = new AbortController()

    setLoading(true)
    setError(null)

    void (async () => {
      try {
        const result = await fetchData({
          page,
          pageSize,
          signal: controller.signal,
          query: draftQuery,
          sort,
        })

        if (controller.signal.aborted) return

        setRows(result.items)
        setTotal(result.total)
      } catch (loadError) {
        if (!controller.signal.aborted) {
          setError(loadError)
          onError?.(loadError)
        }
      } finally {
        if (!controller.signal.aborted) {
          setLoading(false)
        }
      }
    })()

    return () => controller.abort()
  }, [
    allQueryFields,
    draftQuery,
    fetchData,
    onError,
    page,
    pageSize,
    reloadToken,
    sort,
  ])

  const updateSelectedRowKeys = (nextKeys: Key[]) => {
    setInternalSelectedRowKeys(nextKeys)
  }

  const clearSelection = () => {
    updateSelectedRowKeys([])
  }

  const handleResetQuery = () => {
    setPage(1)
    setDraftQuery(createQueryState(initialQuery))
  }

  const activeBulkUpdateField = useMemo(
    () =>
      availableBulkUpdateFields.find(
        (field) => field.key === bulkUpdateFieldKey
      ) ??
      availableBulkUpdateFields[0] ??
      null,
    [availableBulkUpdateFields, bulkUpdateFieldKey]
  )

  const handleOpenBulkUpdate = () => {
    if (bulkUpdate === false || selectedRowKeys.length === 0) return

    const defaultField = availableBulkUpdateFields[0] ?? null
    setBulkUpdateFieldKey(defaultField?.key ?? "")
    setBulkUpdateValue("")
    setBulkUpdateDialogOpen(true)
  }

  const selectedRowKeySet = useMemo(
    () => new Set<Key>(selectedRowKeys),
    [selectedRowKeys]
  )
  const currentPageRowKeys = useMemo(
    () => rows.map((row, rowIndex) => getRowId(row, rowIndex)),
    [getRowId, rows]
  )
  const selectedRows = useMemo(
    () =>
      rows.filter((row, rowIndex) =>
        selectedRowKeySet.has(getRowId(row, rowIndex))
      ),
    [getRowId, rows, selectedRowKeySet]
  )

  const handleBulkUpdateSubmit = async () => {
    if (
      bulkUpdate === false ||
      activeBulkUpdateField === null ||
      updating ||
      selectedRowKeys.length === 0
    ) {
      return
    }

    setUpdating(true)

    try {
      await bulkUpdate.onSubmit({
        clearSelection,
        fieldKey: activeBulkUpdateField.key,
        selectedRowKeys,
        selectedRows,
        value: bulkUpdateValue,
      })
      setBulkUpdateDialogOpen(false)
      setBulkUpdateValue("")
      clearSelection()
      handleRetry()
    } finally {
      setUpdating(false)
    }
  }

  const updateDraftQueryValue = <K extends keyof TQuery>(
    key: K,
    value: TQuery[K]
  ) => {
    setPage(1)
    setDraftQuery((current) => ({
      ...current,
      [key]: value,
    }))
  }

  const toggleSort = (column: DataTableColumn<T>) => {
    if (column.sortable !== true) return

    setPage(1)
    setSort((current) => {
      if (current?.columnKey !== column.key) {
        return {
          columnKey: column.key,
          direction: "asc",
        }
      }

      if (current.direction === "asc") {
        return {
          columnKey: column.key,
          direction: "desc",
        }
      }

      return null
    })
  }

  const getSortIndicator = (column: DataTableColumn<T>) => {
    if (column.sortable !== true) return null
    if (sort?.columnKey !== column.key) return "↕"
    return sort.direction === "asc" ? "↑" : "↓"
  }

  const getSortAriaLabel = (column: DataTableColumn<T>) => {
    if (column.sortable !== true) return undefined
    if (sort?.columnKey !== column.key) return resolvedSortAscendingLabel
    return sort.direction === "asc"
      ? resolvedSortDescendingLabel
      : resolvedClearSortLabel
  }

  const getAriaSort = (column: DataTableColumn<T>) => {
    if (column.sortable !== true || sort?.columnKey !== column.key) {
      return "none" as const
    }

    return sort.direction === "asc" ? "ascending" : "descending"
  }

  const allCurrentPageRowsSelected =
    rowSelectionEnabled &&
    currentPageRowKeys.length > 0 &&
    currentPageRowKeys.every((key) => selectedRowKeySet.has(key))
  const someCurrentPageRowsSelected =
    rowSelectionEnabled &&
    currentPageRowKeys.some((key) => selectedRowKeySet.has(key))
  const headerSelectionState: boolean | "indeterminate" =
    allCurrentPageRowsSelected
      ? true
      : someCurrentPageRowsSelected
        ? "indeterminate"
        : false
  const totalColumnCount =
    resolvedColumns.length + (rowSelectionEnabled ? 1 : 0)

  const handleBulkDelete = async () => {
    if (bulkDelete === false || deleting || selectedRowKeys.length === 0) return

    setDeleting(true)

    try {
      await bulkDelete.onDelete({
        clearSelection,
        selectedRowKeys,
        selectedRows,
      })
      clearSelection()
      handleRetry()
    } catch (deleteError) {
      console.error("DataTable bulk delete failed:", deleteError)
    } finally {
      setDeleting(false)
    }
  }

  const handleBulkDeleteAndClose = async () => {
    await handleBulkDelete()
    setBulkDeleteDialogOpen(false)
  }

  const handleInsertConfirm = async () => {
    if (insert === false || !insert.onConfirm || submittingInsert) {
      setInsertDialogOpen(false)
      return
    }

    setSubmittingInsert(true)

    try {
      await insert.onConfirm()
      setInsertDialogOpen(false)
      handleRetry()
    } catch (insertError) {
      console.error("DataTable insert action failed:", insertError)
    } finally {
      setSubmittingInsert(false)
    }
  }

  const emptyContent = renderEmpty ? renderEmpty() : resolvedEmptyText
  const errorContent = renderError
    ? renderError(error, handleRetry)
    : resolvedErrorText
  const loadingContentResolved = renderLoading
    ? renderLoading()
    : resolvedLoadingText

  const renderQueryFieldControl = (field: DataTableRenderedQueryField<TQuery>) => {
    const value = draftQuery[field.key]
    const disabled = loading || field.disabled === true
    const setValue = (nextValue: TQuery[keyof TQuery & string]) => {
      updateDraftQueryValue(field.key, nextValue as TQuery[typeof field.key])
    }

    if (field.type === "text") {
      return (
        <Input
          value={asStringValue(value)}
          onValueChange={(nextValue) =>
            setValue(nextValue as TQuery[keyof TQuery & string])
          }
          placeholder={field.placeholder}
          disabled={disabled}
          type={field.inputType}
        />
      )
    }

    if (field.type === "search") {
      return (
        <SearchInput
          value={asStringValue(value)}
          onValueChange={(nextValue) =>
            setValue(nextValue as TQuery[keyof TQuery & string])
          }
          placeholder={field.placeholder}
          disabled={disabled}
          updateStrategy="enter"
        />
      )
    }

    if (field.type === "select") {
      return (
        <AdvancedSelect
          value={asStringValue(value)}
          onValueChange={(nextValue) =>
            setValue(nextValue as TQuery[keyof TQuery & string])
          }
          list={field.options.map((option) => ({
            label: option.label,
            value: option.value,
            disabled: option.disabled,
          }))}
          disabled={disabled}
          placeholder={field.placeholder}
          allowClear
        />
      )
    }

    if (field.type === "date-range") {
      return (
        <DateRangePicker
          value={asDateRangeValue(value)}
          onValueChange={(nextValue) =>
            setValue(nextValue as TQuery[keyof TQuery & string])
          }
          placeholder={field.placeholder}
          disabled={disabled}
        />
      )
    }

    if (field.type === "scoped-date-range") {
      const scopeValue = draftQuery[field.scopeKey]

      return (
        <div className="flex items-center gap-1.5">
          <AdvancedSelect
            value={asStringValue(scopeValue)}
            onValueChange={(nextValue) =>
              updateDraftQueryValue(
                field.scopeKey,
                nextValue as TQuery[typeof field.scopeKey]
              )
            }
            list={field.options.map((option) => ({
              label: option.label,
              value: option.value,
              disabled: option.disabled,
            }))}
            disabled={disabled}
            placeholder={field.scopePlaceholder}
          />
          <DateRangePicker
            value={asDateRangeValue(value)}
            onValueChange={(nextValue) =>
              setValue(nextValue as TQuery[keyof TQuery & string])
            }
            placeholder={field.rangePlaceholder ?? field.placeholder}
            disabled={disabled}
          />
        </div>
      )
    }

    return null
  }

  return (
    <TooltipProvider>
      <div
        className="flex h-full min-h-0 max-w-full min-w-0 flex-col overflow-hidden rounded-xl bg-background"
        data-slot="data-table"
        style={{
          height: resolveTableHeight(height),
          width: fillWidth
            ? "100%"
            : `min(100%, ${resolvedSurfaceWidth}px)`,
        }}
      >
        <div
          className="shrink-0 px-3 pt-2"
          data-slot="data-table-header"
        >
          <div className="max-w-full">
            <DataTableHeader
              hasAnyQueryFields={hasAnyQueryFields}
              leadingBuiltInSearchField={
                leadingBuiltInSearchField as DataTableBuiltInQueryField<TQuery> | null
              }
              trailingBuiltInQueryFields={trailingBuiltInQueryFields}
              queryFields={queryFields}
              auditQueryField={auditQueryField}
              queryTools={queryTools}
              loading={loading}
              insert={insert}
              bulkDelete={bulkDelete}
              bulkUpdate={bulkUpdate}
              toolbarActions={toolbarActions}
              rowSelectionEnabled={rowSelectionEnabled}
              deleting={deleting}
              selectedRowKeysCount={selectedRowKeys.length}
              resolvedResetLabel={resolvedResetLabel}
              resolvedRefreshLabel={resolvedRefreshLabel}
              resolvedInsertLabel={resolvedInsertLabel}
              resolvedBulkUpdateLabel={resolvedBulkUpdateLabel}
              resolvedBulkDeleteLabel={resolvedBulkDeleteLabel}
              renderQueryFieldControl={renderQueryFieldControl}
              onResetQuery={handleResetQuery}
              onRetry={handleRetry}
              onOpenInsert={() => setInsertDialogOpen(true)}
              onOpenBulkUpdate={handleOpenBulkUpdate}
              onOpenBulkDelete={() => setBulkDeleteDialogOpen(true)}
            />
          </div>
        </div>

        <div
          className="mt-3 min-h-0 flex-1 overflow-hidden"
          data-slot="data-table-body"
        >
          <div ref={tableViewportRef} className="h-full overflow-auto">
          <DataTableSurface fillWidth={fillWidth}>
              {caption ? <DataTableSurfaceCaption>{caption}</DataTableSurfaceCaption> : null}
              <DataTableSurfaceHeader>
                <DataTableSurfaceRow>
                  {rowSelectionEnabled ? (
                    <DataTableSurfaceHead
                      stickySide="left"
                      stickyOffset={0}
                      minWidth={`${selectionColumnWidth}px`}
                      width={`${selectionColumnWidth}px`}
                      priority="selection"
                      compactColumns={compactColumns}
                      compactRows={compactRows}
                    >
                      <div className="flex items-center justify-center">
                        <Checkbox
                          checked={headerSelectionState}
                          onCheckedChange={(checked) => {
                            if (checked !== true) {
                              updateSelectedRowKeys(
                                selectedRowKeys.filter(
                                  (key) => !currentPageRowKeys.includes(key)
                                )
                              )
                              return
                            }

                            updateSelectedRowKeys(
                              Array.from(
                                new Set<Key>([
                                  ...selectedRowKeys,
                                  ...currentPageRowKeys,
                                ])
                              )
                            )
                          }}
                          disabled={!hasRows || loading}
                        />
                      </div>
                    </DataTableSurfaceHead>
                  ) : null}
                  {resolvedColumns.map((column, columnIndex) => {
                    const leftOffset = stickyLeftOffsets[columnIndex]
                    const rightOffset = stickyRightOffsets[columnIndex]
                    const isStickyLeft = leftOffset !== undefined
                    const isStickyRight = rightOffset !== undefined

                    return (
                      <DataTableSurfaceHead
                        key={column.key}
                        ref={(element) => {
                          headerCellRefs.current[columnIndex] = element
                        }}
                        ariaSort={getAriaSort(column)}
                        stickySide={
                          isStickyLeft ? "left" : isStickyRight ? "right" : undefined
                        }
                        stickyOffset={isStickyLeft ? leftOffset : rightOffset}
                        priority={isStickyLeft || isStickyRight ? "sticky" : "base"}
                        minWidth={resolveColumnMinWidth(
                          column as DataTableColumn<object>
                        )}
                        width={resolveColumnMinWidth(
                          column as DataTableColumn<object>
                        )}
                        compactColumns={compactColumns}
                        compactRows={compactRows}
                      >
                        <button
                          type="button"
                          className="flex w-full cursor-pointer items-center gap-2 text-left disabled:cursor-default"
                          onClick={() => toggleSort(column)}
                          disabled={column.sortable !== true}
                          aria-label={getSortAriaLabel(column)}
                        >
                          <span className="min-w-0 truncate">
                            {column.header}
                          </span>
                          {column.sortable !== true ? null : (
                            <span
                              aria-hidden="true"
                              className="shrink-0 text-xs text-muted-foreground"
                            >
                              {getSortIndicator(column)}
                            </span>
                          )}
                        </button>
                      </DataTableSurfaceHead>
                    )
                  })}
                </DataTableSurfaceRow>
              </DataTableSurfaceHeader>

              <DataTableSurfaceBody>
                {loading ? (
                  <DataTableSurfaceRow compactRows={compactRows}>
                    <DataTableSurfaceCell
                      colSpan={totalColumnCount}
                      compactColumns={compactColumns}
                      compactRows={compactRows}
                    >
                      {renderLoading ? (
                        loadingContentResolved
                      ) : (
                        <div
                          style={{
                            alignItems: "center",
                            display: "flex",
                            gap: "8px",
                          }}
                        >
                          <Spinner size="sm" />
                          <span>{loadingContentResolved}</span>
                        </div>
                      )}
                    </DataTableSurfaceCell>
                  </DataTableSurfaceRow>
                ) : null}

                {!loading && error ? (
                  <DataTableSurfaceRow compactRows={compactRows}>
                    <DataTableSurfaceCell
                      colSpan={totalColumnCount}
                      compactColumns={compactColumns}
                      compactRows={compactRows}
                    >
                      {errorContent}
                    </DataTableSurfaceCell>
                  </DataTableSurfaceRow>
                ) : null}

                {!loading && !error && !hasRows ? (
                  <DataTableSurfaceRow compactRows={compactRows}>
                    <DataTableSurfaceCell
                      colSpan={totalColumnCount}
                      compactColumns={compactColumns}
                      compactRows={compactRows}
                    >
                      {emptyContent}
                    </DataTableSurfaceCell>
                  </DataTableSurfaceRow>
                ) : null}

                {!loading && !error
                  ? rows.map((row: T, rowIndex: number) => {
                      const isStriped = stripedRows && rowIndex % 2 === 1

                      return (
                      <DataTableSurfaceRow
                        key={getRowId(row, rowIndex)}
                        striped={isStriped}
                        compactRows={compactRows}
                      >
                        {rowSelectionEnabled ? (
                          <DataTableSurfaceCell
                            stickySide="left"
                            stickyOffset={0}
                            minWidth={`${selectionColumnWidth}px`}
                            width={`${selectionColumnWidth}px`}
                            striped={isStriped}
                            priority="selection"
                            compactColumns={compactColumns}
                            compactRows={compactRows}
                          >
                            <div className="flex items-center justify-center">
                              <Checkbox
                                checked={selectedRowKeySet.has(
                                  getRowId(row, rowIndex)
                                )}
                                onCheckedChange={(checked) => {
                                  const rowKey = getRowId(row, rowIndex)

                                  if (!checked) {
                                    updateSelectedRowKeys(
                                      selectedRowKeys.filter(
                                        (key) => key !== rowKey
                                      )
                                    )
                                    return
                                  }

                                  updateSelectedRowKeys(
                                    Array.from(
                                      new Set<Key>([...selectedRowKeys, rowKey])
                                    )
                                  )
                                }}
                              />
                            </div>
                          </DataTableSurfaceCell>
                        ) : null}
                        {resolvedColumns.map((column, columnIndex) => (
                          <DataTableSurfaceCell
                            key={column.key}
                            stickySide={
                              stickyLeftOffsets[columnIndex] !== undefined
                                ? "left"
                                : stickyRightOffsets[columnIndex] !== undefined
                                  ? "right"
                                  : undefined
                            }
                            stickyOffset={
                              stickyLeftOffsets[columnIndex] ??
                              stickyRightOffsets[columnIndex]
                            }
                            striped={isStriped}
                            priority={
                              stickyLeftOffsets[columnIndex] !== undefined ||
                              stickyRightOffsets[columnIndex] !== undefined
                                ? "sticky"
                                : "base"
                            }
                            minWidth={resolveColumnMinWidth(
                              column as DataTableColumn<object>
                            )}
                            width={resolveColumnMinWidth(
                              column as DataTableColumn<object>
                            )}
                            compactColumns={compactColumns}
                            compactRows={compactRows}
                          >
                            <Fragment>
                              {column.renderCell(row, rowIndex)}
                            </Fragment>
                          </DataTableSurfaceCell>
                        ))}
                      </DataTableSurfaceRow>
                    )})
                  : null}
              </DataTableSurfaceBody>
            </DataTableSurface>
          </div>
        </div>

        <div
          className="mt-2 overflow-x-auto px-3 py-2"
          data-slot="data-table-tail"
        >
          <div
            className="max-w-full"
            style={
              fillWidth ? undefined : { width: `${resolvedSurfaceWidth}px` }
            }
          >
            <div className="flex min-w-max flex-nowrap items-center justify-between">
              <div className="flex shrink-0 items-center gap-2 text-sm">
                <span>
                  <strong>{resolvedTotalLabel}:</strong> {total}
                </span>
                <AdvancedSelect
                  value={String(pageSize)}
                  onValueChange={(value: string) => {
                    setPage(1)
                    setPageSize(Number(value))
                  }}
                  list={safePageSizeOptions.map((value) => ({
                    label: String(value),
                    value: String(value),
                  }))}
                  disabled={loading}
                />
              </div>

              <div className="shrink-0">
                <Pagination
                  page={Math.min(page, totalPages)}
                  totalPages={totalPages}
                  onPageChange={setPage}
                />
              </div>
            </div>
          </div>
        </div>

        <DataTableDialogs
          insert={insert}
          insertDialogOpen={insertDialogOpen}
          setInsertDialogOpen={setInsertDialogOpen}
          submittingInsert={submittingInsert}
          handleInsertConfirm={handleInsertConfirm}
          bulkDelete={bulkDelete}
          bulkDeleteDialogOpen={bulkDeleteDialogOpen}
          setBulkDeleteDialogOpen={setBulkDeleteDialogOpen}
          deleting={deleting}
          selectedRowKeys={selectedRowKeys}
          handleBulkDeleteAndClose={handleBulkDeleteAndClose}
          bulkUpdate={bulkUpdate}
          bulkUpdateDialogOpen={bulkUpdateDialogOpen}
          setBulkUpdateDialogOpen={setBulkUpdateDialogOpen}
          updating={updating}
          activeBulkUpdateField={activeBulkUpdateField}
          bulkUpdateValue={bulkUpdateValue}
          setBulkUpdateFieldKey={setBulkUpdateFieldKey}
          setBulkUpdateValue={(value) => setBulkUpdateValue(value)}
          availableBulkUpdateFields={availableBulkUpdateFields}
          handleBulkUpdateSubmit={handleBulkUpdateSubmit}
          resolvedInsertLabel={resolvedInsertLabel}
          resolvedCancelLabel={resolvedCancelLabel}
          resolvedSaveLabel={resolvedSaveLabel}
          resolvedDeleteDialogTitle={resolvedDeleteDialogTitle}
          resolvedBulkDeleteDialogDescription={resolvedBulkDeleteDialogDescription}
          resolvedConfirmDeleteLabel={resolvedConfirmDeleteLabel}
          resolvedBulkUpdateTitle={resolvedBulkUpdateTitle}
          resolvedBulkUpdateDescription={resolvedBulkUpdateDescription}
          resolvedBulkUpdateFieldLabel={resolvedBulkUpdateFieldLabel}
          resolvedBulkUpdateValueLabel={resolvedBulkUpdateValueLabel}
          resolvedBulkUpdateCancelLabel={resolvedBulkUpdateCancelLabel}
          resolvedBulkUpdateApplyLabel={resolvedBulkUpdateApplyLabel}
        />
      </div>
    </TooltipProvider>
  )
}
