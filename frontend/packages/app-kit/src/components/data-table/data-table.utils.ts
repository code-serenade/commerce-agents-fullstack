import type {
  DataTableColumn,
  DateRangeValue,
  DataTableRenderedQueryField,
  DataTableRowActionItem,
  DataTableRowActionsConfig,
} from "./data-table.types"
import type { ReactNode } from "react"

export const DEFAULT_PAGE_SIZE_OPTIONS = [10, 15, 30, 50] as const
export const DEFAULT_STICKY_COLUMN_WIDTH = 160
export const DEFAULT_SELECTION_COLUMN_WIDTH = 52
const DEFAULT_ROW_ACTION_BUTTON_WIDTH = 32
const DEFAULT_ROW_ACTION_GAP_WIDTH = 6
const DEFAULT_ROW_ACTION_CELL_PADDING = 16

export function createQueryState<TQuery extends object>(initialQuery?: TQuery) {
  return { ...(initialQuery ?? {}) } as TQuery
}

export function asStringValue(value: unknown) {
  return typeof value === "string" ? value : ""
}

export function asDateRangeValue(value: unknown) {
  if (typeof value !== "object" || value === null || Array.isArray(value)) {
    return undefined
  }

  return value as DateRangeValue
}

export function resolveTableHeight(height?: number | string) {
  if (height === undefined) return undefined
  return typeof height === "number" ? `${height}px` : height
}

export function resolveColumnMinWidth(column: DataTableColumn<object>) {
  if (column.width === undefined) return undefined

  return typeof column.width === "number" ? `${column.width}px` : column.width
}

export function resolveRowActionsColumnWidth<T>(
  rowActions: DataTableRowActionsConfig<T>
) {
  if (rowActions.columnWidth !== undefined) {
    return rowActions.columnWidth
  }

  const buttonCount =
    (rowActions.edit ? 1 : 0) +
    (rowActions.delete ? 1 : 0) +
    ((rowActions.moreItems?.length ?? 0) > 0 ? 1 : 0)

  if (buttonCount <= 0) {
    return 0
  }

  return (
    buttonCount * DEFAULT_ROW_ACTION_BUTTON_WIDTH +
    Math.max(0, buttonCount - 1) * DEFAULT_ROW_ACTION_GAP_WIDTH +
    DEFAULT_ROW_ACTION_CELL_PADDING
  )
}

export function resolveColumnPixelWidth(column: DataTableColumn<object>) {
  if (typeof column.width === "number") return column.width
  if (typeof column.width !== "string") return undefined

  const matched = /^(\d+(?:\.\d+)?)px$/.exec(column.width.trim())
  return matched ? Number(matched[1]) : undefined
}

export function isLeftStickyColumn<T>(
  column: DataTableColumn<T>,
  columnIndex: number,
  fixedLeftColumns: number
) {
  return column.sticky === "left" || columnIndex < fixedLeftColumns
}

export function isRightStickyColumn<T>(
  column: DataTableColumn<T>,
  columnIndex: number,
  columnCount: number,
  fixedLeftColumns: number,
  fixedRightColumns: number
) {
  if (column.sticky === "left" || columnIndex < fixedLeftColumns) {
    return false
  }

  return (
    column.sticky === "right" ||
    (fixedRightColumns > 0 && columnIndex >= columnCount - fixedRightColumns)
  )
}

export function getStickyColumnStyles({
  leftOffset,
  rightOffset,
}: {
  leftOffset?: number
  rightOffset?: number
}) {
  if (leftOffset === undefined && rightOffset === undefined) return undefined

  return {
    ...(leftOffset !== undefined ? { left: `${leftOffset}px` } : null),
    position: "sticky" as const,
    ...(rightOffset !== undefined ? { right: `${rightOffset}px` } : null),
  }
}

export function getQueryFieldLayoutStyle(
  field: DataTableRenderedQueryField<object>
) {
  if (field.type === "search") {
    return {
      flex: "0 1 360px",
      maxWidth: "360px",
    }
  }

  if (field.type === "text") {
    return {
      flex: "1 1 280px",
      maxWidth: "360px",
    }
  }

  if (field.type === "date-range") {
    return {
      flex: "0 1 240px",
      maxWidth: "280px",
    }
  }

  if (field.type === "scoped-date-range") {
    return {
      flex: "0 0 auto",
      maxWidth: "none",
    }
  }

  if (field.type === "select") {
    return {
      flex: "0 1 180px",
      maxWidth: "220px",
    }
  }

  return {
    flex: "0 1 220px",
    maxWidth: "280px",
  }
}

export function resolveRowActionDisabled<T>(
  disabled: DataTableRowActionItem<T>["disabled"],
  row: T,
  rowIndex: number
) {
  if (typeof disabled === "function") {
    return disabled(row, rowIndex)
  }

  return disabled === true
}

export function resolveRowActionContent<T>(
  value: ReactNode | ((row: T, rowIndex: number) => ReactNode) | undefined,
  row: T,
  rowIndex: number
) {
  if (typeof value === "function") {
    return value(row, rowIndex)
  }

  return value
}
