import type { ReactNode } from "react"
import {
  Table as CoreTable,
  TableBody as CoreTableBody,
  TableCaption as CoreTableCaption,
  TableCell as CoreTableCell,
  TableFooter as CoreTableFooter,
  TableHead as CoreTableHead,
  TableHeader as CoreTableHeader,
  TableRow as CoreTableRow,
} from "@workspace/ui-core/components/table"

export interface TableProps {
  children: ReactNode
  ariaLabel?: string
  ariaLabelledBy?: string
  ariaDescribedBy?: string
}

export interface TableHeaderProps {
  children: ReactNode
}

export interface TableBodyProps {
  children: ReactNode
}

export interface TableFooterProps {
  children: ReactNode
}

export interface TableRowProps {
  children: ReactNode
}

export interface TableHeadProps {
  children: ReactNode
  colSpan?: number
  rowSpan?: number
  scope?: "col" | "row" | "colgroup" | "rowgroup"
  ariaSort?: "none" | "ascending" | "descending" | "other"
}

export interface TableCellProps {
  children: ReactNode
  colSpan?: number
  rowSpan?: number
}

export interface TableCaptionProps {
  children: ReactNode
}

export function Table({
  children,
  ariaLabel,
  ariaLabelledBy,
  ariaDescribedBy,
}: TableProps) {
  return (
    <CoreTable
      aria-label={ariaLabel}
      aria-labelledby={ariaLabelledBy}
      aria-describedby={ariaDescribedBy}
    >
      {children}
    </CoreTable>
  )
}

export function TableHeader({ children }: TableHeaderProps) {
  return <CoreTableHeader>{children}</CoreTableHeader>
}

export function TableBody({ children }: TableBodyProps) {
  return <CoreTableBody>{children}</CoreTableBody>
}

export function TableFooter({ children }: TableFooterProps) {
  return <CoreTableFooter>{children}</CoreTableFooter>
}

export function TableRow({ children }: TableRowProps) {
  return <CoreTableRow>{children}</CoreTableRow>
}

export function TableHead({
  children,
  colSpan,
  rowSpan,
  scope,
  ariaSort,
}: TableHeadProps) {
  return (
    <CoreTableHead
      colSpan={colSpan}
      rowSpan={rowSpan}
      scope={scope}
      aria-sort={ariaSort}
    >
      {children}
    </CoreTableHead>
  )
}

export function TableCell({ children, colSpan, rowSpan }: TableCellProps) {
  return (
    <CoreTableCell colSpan={colSpan} rowSpan={rowSpan}>
      {children}
    </CoreTableCell>
  )
}

export function TableCaption({ children }: TableCaptionProps) {
  return <CoreTableCaption>{children}</CoreTableCaption>
}
