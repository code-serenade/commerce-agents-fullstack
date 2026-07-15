import type { CSSProperties, ReactNode } from "react"
import { useTranslation } from "react-i18next"
import { Spinner } from "@workspace/ui-components"

export interface CommerceColumn<T> {
  key: string
  header: ReactNode
  render: (item: T) => ReactNode
  minWidth?: string
  sticky?: "left" | "right"
  width?: string
}

const DEFAULT_COLUMN_WIDTH = "10rem"
const STICKY_LEFT_OFFSET = "0px"
const STICKY_RIGHT_OFFSET = "0px"

export function CommerceTable<T>({
  columns,
  emptyText,
  getRowId,
  items,
  loading,
}: {
  columns: CommerceColumn<T>[]
  emptyText: string
  getRowId: (item: T, index: number) => string
  items: T[]
  loading?: boolean
}) {
  const { t } = useTranslation()

  if (loading) {
    return (
      <div className="flex min-h-40 items-center justify-center gap-2 text-sm text-muted-foreground">
        <Spinner />
        <span>{t("commerce.common.loading")}</span>
      </div>
    )
  }

  if (!items.length) {
    return (
      <div className="flex min-h-40 items-center justify-center rounded-lg border border-dashed border-(--app-border) bg-background px-4 text-center text-sm text-muted-foreground">
        {emptyText}
      </div>
    )
  }

  const resolvedColumns = columns.map((column) => ({
    ...column,
    sticky:
      column.sticky ??
      (column.key === "selection"
        ? "left"
        : column.key === "actions"
          ? "right"
          : undefined),
  }))
  const tableMinWidth = `max(100%, calc(${resolvedColumns
    .map((column) => column.width ?? column.minWidth ?? DEFAULT_COLUMN_WIDTH)
    .join(" + ")}))`

  return (
    <div className="max-w-full overflow-x-auto">
      <div style={{ minWidth: tableMinWidth, width: "100%" }}>
        <table className="w-full table-fixed caption-bottom text-sm">
          <thead className="[&_tr]:border-b">
            <tr className="border-b transition-colors hover:bg-muted/50 data-[state=selected]:bg-muted">
              {resolvedColumns.map((column) => (
                <th
                  key={column.key}
                  scope="col"
                  className={getHeadClassName(column.sticky)}
                  style={getColumnStyle(column)}
                >
                  <span
                    className={column.width ? "block" : undefined}
                    style={column.width ? { width: column.width } : undefined}
                  >
                    {column.header}
                  </span>
                </th>
              ))}
            </tr>
          </thead>
          <tbody className="[&_tr:last-child]:border-0">
            {items.map((item, index) => (
              <tr
                key={getRowId(item, index)}
                className="group/commerce-table-row border-b"
              >
                {resolvedColumns.map((column) => (
                  <td
                    key={column.key}
                    className={getCellClassName(column.sticky)}
                    style={getColumnStyle(column)}
                  >
                    {column.render(item)}
                  </td>
                ))}
              </tr>
            ))}
          </tbody>
        </table>
      </div>
    </div>
  )
}

function getColumnStyle<T>(column: CommerceColumn<T>): CSSProperties {
  const minWidth = column.minWidth ?? column.width ?? DEFAULT_COLUMN_WIDTH

  return {
    minWidth,
    ...(column.width ? { width: column.width } : null),
    ...(column.width ? { maxWidth: column.width } : null),
    ...(column.sticky === "left" ? { left: STICKY_LEFT_OFFSET } : null),
    ...(column.sticky === "right" ? { right: STICKY_RIGHT_OFFSET } : null),
  }
}

function getHeadClassName(sticky: CommerceColumn<unknown>["sticky"]) {
  return cx(
    "h-10 overflow-hidden whitespace-nowrap bg-(--surface) px-2 text-left align-middle font-medium text-foreground",
    sticky ? getStickyCellClassName(sticky, "head") : undefined
  )
}

function getCellClassName(sticky: CommerceColumn<unknown>["sticky"]) {
  return cx(
    "overflow-hidden whitespace-normal bg-background p-2 align-top wrap-anywhere transition-colors duration-500 group-hover/commerce-table-row:bg-[color-mix(in_oklch,var(--background)_88%,var(--primary)_12%)] [&>*]:max-w-full",
    sticky ? getStickyCellClassName(sticky, "cell") : undefined
  )
}

function getStickyCellClassName(
  sticky: CommerceColumn<unknown>["sticky"],
  kind: "cell" | "head"
) {
  if (!sticky) return undefined

  return cx(
    "sticky",
    sticky === "left"
      ? "left-0 shadow-[inset_-1px_0_0_var(--border)]"
      : "right-0 shadow-[inset_1px_0_0_var(--border)]",
    kind === "head" ? "z-20 bg-(--surface)" : "z-10 bg-background"
  )
}

function cx(...classes: Array<string | false | null | undefined>) {
  return classes.filter(Boolean).join(" ")
}
