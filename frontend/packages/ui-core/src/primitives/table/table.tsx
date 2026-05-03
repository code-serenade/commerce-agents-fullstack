"use client"

import { DEFAULT_MODE } from "../../lib/component-mode"
import { cn } from "../../lib/utils"
import { tableClassNames } from "./table.styles"
import type {
  TableBodyProps,
  TableCaptionProps,
  TableCellProps,
  TableClassResolver,
  TableFooterProps,
  TableHeadProps,
  TableHeaderProps,
  TableProps,
  TableRowProps,
} from "./table.types"

function resolveStyledTableClassName({
  className,
  defaultClassName,
  classNameMode,
  classResolver,
}: {
  className?: string
  defaultClassName: string
  classNameMode: "merge" | "replace"
  classResolver?: TableClassResolver
}) {
  if (classResolver) {
    return classResolver({
      defaultClassName,
      className,
    })
  }

  if (classNameMode === "replace") {
    return className ?? defaultClassName
  }

  return cn(defaultClassName, className)
}

function Table({
  mode = DEFAULT_MODE,
  className,
  classNameMode = "merge",
  classResolver,
  containerClassName,
  containerClassNameMode = "merge",
  containerClassResolver,
  ...props
}: TableProps) {
  if (mode === "primitive") {
    return (
      <div className={containerClassName}>
        <table className={className} {...props} />
      </div>
    )
  }

  return (
    <div
      data-slot="table-container"
      className={resolveStyledTableClassName({
        className: containerClassName,
        defaultClassName: tableClassNames.slot1,
        classNameMode: containerClassNameMode,
        classResolver: containerClassResolver,
      })}
    >
      <table
        data-slot="table"
        className={resolveStyledTableClassName({
          className,
          defaultClassName: tableClassNames.slot2,
          classNameMode,
          classResolver,
        })}
        {...props}
      />
    </div>
  )
}

function TableHeader({
  mode = DEFAULT_MODE,
  className,
  classNameMode = "merge",
  classResolver,
  ...props
}: TableHeaderProps) {
  if (mode === "primitive") {
    return <thead className={className} {...props} />
  }

  return (
    <thead
      data-slot="table-header"
      className={resolveStyledTableClassName({
        className,
        defaultClassName: tableClassNames.slot3,
        classNameMode,
        classResolver,
      })}
      {...props}
    />
  )
}

function TableBody({
  mode = DEFAULT_MODE,
  className,
  classNameMode = "merge",
  classResolver,
  ...props
}: TableBodyProps) {
  if (mode === "primitive") {
    return <tbody className={className} {...props} />
  }

  return (
    <tbody
      data-slot="table-body"
      className={resolveStyledTableClassName({
        className,
        defaultClassName: tableClassNames.slot4,
        classNameMode,
        classResolver,
      })}
      {...props}
    />
  )
}

function TableFooter({
  mode = DEFAULT_MODE,
  className,
  classNameMode = "merge",
  classResolver,
  ...props
}: TableFooterProps) {
  if (mode === "primitive") {
    return <tfoot className={className} {...props} />
  }

  return (
    <tfoot
      data-slot="table-footer"
      className={resolveStyledTableClassName({
        className,
        defaultClassName: tableClassNames.slot5,
        classNameMode,
        classResolver,
      })}
      {...props}
    />
  )
}

function TableRow({
  mode = DEFAULT_MODE,
  className,
  classNameMode = "merge",
  classResolver,
  ...props
}: TableRowProps) {
  if (mode === "primitive") {
    return <tr className={className} {...props} />
  }

  return (
    <tr
      data-slot="table-row"
      className={resolveStyledTableClassName({
        className,
        defaultClassName: tableClassNames.slot6,
        classNameMode,
        classResolver,
      })}
      {...props}
    />
  )
}

function TableHead({
  mode = DEFAULT_MODE,
  className,
  classNameMode = "merge",
  classResolver,
  ...props
}: TableHeadProps) {
  if (mode === "primitive") {
    return <th className={className} {...props} />
  }

  return (
    <th
      data-slot="table-head"
      className={resolveStyledTableClassName({
        className,
        defaultClassName: tableClassNames.slot7,
        classNameMode,
        classResolver,
      })}
      {...props}
    />
  )
}

function TableCell({
  mode = DEFAULT_MODE,
  className,
  classNameMode = "merge",
  classResolver,
  ...props
}: TableCellProps) {
  if (mode === "primitive") {
    return <td className={className} {...props} />
  }

  return (
    <td
      data-slot="table-cell"
      className={resolveStyledTableClassName({
        className,
        defaultClassName: tableClassNames.slot8,
        classNameMode,
        classResolver,
      })}
      {...props}
    />
  )
}

function TableCaption({
  mode = DEFAULT_MODE,
  className,
  classNameMode = "merge",
  classResolver,
  ...props
}: TableCaptionProps) {
  if (mode === "primitive") {
    return <caption className={className} {...props} />
  }

  return (
    <caption
      data-slot="table-caption"
      className={resolveStyledTableClassName({
        className,
        defaultClassName: tableClassNames.slot9,
        classNameMode,
        classResolver,
      })}
      {...props}
    />
  )
}

export {
  Table,
  TableHeader,
  TableBody,
  TableFooter,
  TableHead,
  TableRow,
  TableCell,
  TableCaption,
}
