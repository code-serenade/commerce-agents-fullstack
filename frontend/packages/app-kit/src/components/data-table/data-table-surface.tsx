import { forwardRef, type CSSProperties, type ReactNode } from "react"
import {
  Table as CoreTable,
  TableBody as CoreTableBody,
  TableCaption as CoreTableCaption,
  TableCell as CoreTableCell,
  TableHead as CoreTableHead,
  TableHeader as CoreTableHeader,
  TableRow as CoreTableRow,
} from "@workspace/ui-core/components/table"
import { cn } from "@workspace/ui-core/lib/utils.js"

type StickySide = "left" | "right"

const stickyBlurStyle = {
  WebkitBackdropFilter: "blur(14px)",
  backdropFilter: "blur(14px)",
} as const

const stickyHeaderSurfaceStyle = {
  ...stickyBlurStyle,
  backgroundColor:
    "color-mix(in oklab, var(--surface) 82%, transparent)",
} as const

function getStickyBodySurfaceStyle(striped: boolean) {
  return {
    ...stickyBlurStyle,
    backgroundColor: striped
      ? "color-mix(in oklab, var(--muted) 76%, transparent)"
      : "color-mix(in oklab, var(--background) 82%, transparent)",
  } as const
}

function getStickyInsetShadow(side: StickySide) {
  return side === "left"
    ? "shadow-[inset_-1px_0_0_var(--border)]"
    : "shadow-[inset_1px_0_0_var(--border)]"
}

function getStickyOffsetStyle(
  stickySide?: StickySide,
  stickyOffset?: number
): CSSProperties | undefined {
  if (!stickySide || stickyOffset === undefined) {
    return undefined
  }

  return stickySide === "left"
    ? { left: stickyOffset }
    : { right: stickyOffset }
}

export interface DataTableSurfaceProps {
  children: ReactNode
  fillWidth?: boolean
}

export interface DataTableSurfaceHeaderProps {
  children: ReactNode
}

export interface DataTableSurfaceBodyProps {
  children: ReactNode
}

export interface DataTableSurfaceRowProps {
  children: ReactNode
  striped?: boolean
  compactRows?: boolean
}

export interface DataTableSurfaceHeadProps {
  children: ReactNode
  minWidth?: CSSProperties["minWidth"]
  width?: CSSProperties["width"]
  stickySide?: StickySide
  stickyOffset?: number
  priority?: "base" | "sticky" | "selection"
  ariaSort?: "none" | "ascending" | "descending" | "other"
  compactColumns?: boolean
  compactRows?: boolean
}

export interface DataTableSurfaceCellProps {
  children: ReactNode
  colSpan?: number
  minWidth?: CSSProperties["minWidth"]
  width?: CSSProperties["width"]
  stickySide?: StickySide
  stickyOffset?: number
  striped?: boolean
  priority?: "base" | "sticky" | "selection"
  compactColumns?: boolean
  compactRows?: boolean
}

export interface DataTableSurfaceCaptionProps {
  children: ReactNode
}

export function DataTableSurface({
  children,
  fillWidth = true,
}: DataTableSurfaceProps) {
  return (
    <CoreTable
      className={fillWidth ? "w-full min-w-full" : "w-max"}
      containerClassName="overflow-visible"
    >
      {children}
    </CoreTable>
  )
}

export function DataTableSurfaceHeader({
  children,
}: DataTableSurfaceHeaderProps) {
  return <CoreTableHeader>{children}</CoreTableHeader>
}

export function DataTableSurfaceBody({ children }: DataTableSurfaceBodyProps) {
  return <CoreTableBody>{children}</CoreTableBody>
}

export function DataTableSurfaceRow({
  children,
  striped = false,
  compactRows = false,
}: DataTableSurfaceRowProps) {
  return (
    <CoreTableRow
      className={cn(
        striped ? "bg-muted/40" : undefined,
        compactRows ? "h-8" : undefined
      )}
    >
      {children}
    </CoreTableRow>
  )
}

export const DataTableSurfaceHead = forwardRef<
  HTMLTableCellElement,
  DataTableSurfaceHeadProps
>(function DataTableSurfaceHead(
  {
    children,
    minWidth,
    width,
    stickySide,
    stickyOffset,
    priority = "base",
    ariaSort,
    compactColumns = false,
    compactRows = false,
  },
  ref
) {
  const isSticky = stickySide !== undefined
  const zIndexClass =
    priority === "selection"
      ? "z-30"
      : priority === "sticky"
        ? "z-20"
        : "z-10"

  return (
    <CoreTableHead
      ref={ref}
      aria-sort={ariaSort}
      className={cn(
        compactColumns ? "px-1.5" : undefined,
        compactRows ? "h-8 py-1.5" : undefined,
        "sticky top-0",
        zIndexClass,
        isSticky
          ? cn(
              stickySide === "left" ? "left-0" : "right-0",
              getStickyInsetShadow(stickySide)
            )
          : "bg-[var(--surface)]"
      )}
      style={{
        ...getStickyOffsetStyle(stickySide, stickyOffset),
        ...(isSticky ? stickyHeaderSurfaceStyle : null),
        ...(minWidth ? { minWidth } : null),
        ...(width ? { width } : null),
      }}
    >
      {children}
    </CoreTableHead>
  )
})

export function DataTableSurfaceCell({
  children,
  colSpan,
  minWidth,
  width,
  stickySide,
  stickyOffset,
  striped = false,
  priority = "base",
  compactColumns = false,
  compactRows = false,
}: DataTableSurfaceCellProps) {
  const isSticky = stickySide !== undefined
  const zIndexClass =
    priority === "selection"
      ? "z-20"
      : priority === "sticky"
        ? "z-10"
        : undefined

  return (
    <CoreTableCell
      colSpan={colSpan}
      className={cn(
        isSticky
          ? cn(
              "sticky",
              zIndexClass,
              stickySide === "left" ? "left-0" : "right-0",
              getStickyInsetShadow(stickySide)
            )
          : undefined,
        compactColumns ? "px-1.5" : undefined,
        compactRows ? "py-1.5" : undefined
      )}
      style={{
        ...getStickyOffsetStyle(stickySide, stickyOffset),
        ...(isSticky ? getStickyBodySurfaceStyle(striped) : null),
        ...(minWidth ? { minWidth } : null),
        ...(width ? { width } : null),
      }}
    >
      {children}
    </CoreTableCell>
  )
}

export function DataTableSurfaceCaption({
  children,
}: DataTableSurfaceCaptionProps) {
  return <CoreTableCaption>{children}</CoreTableCaption>
}
