import * as React from "react"

import type { BaseMode } from "../../lib/component-mode"

export type TableClassNameMode = "merge" | "replace"

export type TableClassResolver = (params: {
  defaultClassName: string
  className?: string
}) => string

export type TableProps = React.ComponentProps<"table"> & {
  mode?: BaseMode
  classNameMode?: TableClassNameMode
  classResolver?: TableClassResolver
  containerClassName?: string
  containerClassNameMode?: TableClassNameMode
  containerClassResolver?: TableClassResolver
}

export type TableHeaderProps = React.ComponentProps<"thead"> & {
  mode?: BaseMode
  classNameMode?: TableClassNameMode
  classResolver?: TableClassResolver
}

export type TableBodyProps = React.ComponentProps<"tbody"> & {
  mode?: BaseMode
  classNameMode?: TableClassNameMode
  classResolver?: TableClassResolver
}

export type TableFooterProps = React.ComponentProps<"tfoot"> & {
  mode?: BaseMode
  classNameMode?: TableClassNameMode
  classResolver?: TableClassResolver
}

export type TableRowProps = React.ComponentProps<"tr"> & {
  mode?: BaseMode
  classNameMode?: TableClassNameMode
  classResolver?: TableClassResolver
}

export type TableHeadProps = React.ComponentProps<"th"> & {
  mode?: BaseMode
  classNameMode?: TableClassNameMode
  classResolver?: TableClassResolver
}

export type TableCellProps = React.ComponentProps<"td"> & {
  mode?: BaseMode
  classNameMode?: TableClassNameMode
  classResolver?: TableClassResolver
}

export type TableCaptionProps = React.ComponentProps<"caption"> & {
  mode?: BaseMode
  classNameMode?: TableClassNameMode
  classResolver?: TableClassResolver
}
