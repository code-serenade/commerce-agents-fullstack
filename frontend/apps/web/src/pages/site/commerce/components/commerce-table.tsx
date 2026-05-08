import type { ReactNode } from "react"
import { useTranslation } from "react-i18next"
import {
  Spinner,
  Table,
  TableBody,
  TableCell,
  TableHead,
  TableHeader,
  TableRow,
} from "@workspace/ui-components"

export interface CommerceColumn<T> {
  key: string
  header: ReactNode
  render: (item: T) => ReactNode
  width?: string
}

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

  return (
    <div className="overflow-x-auto">
      <Table>
        <TableHeader>
          <TableRow>
            {columns.map((column) => (
              <TableHead key={column.key} scope="col">
                <span
                  className={column.width ? "block" : undefined}
                  style={column.width ? { width: column.width } : undefined}
                >
                  {column.header}
                </span>
              </TableHead>
            ))}
          </TableRow>
        </TableHeader>
        <TableBody>
          {items.map((item, index) => (
            <TableRow key={getRowId(item, index)}>
              {columns.map((column) => (
                <TableCell key={column.key}>{column.render(item)}</TableCell>
              ))}
            </TableRow>
          ))}
        </TableBody>
      </Table>
    </div>
  )
}
