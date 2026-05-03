import * as React from "react"

import type { BaseMode } from "../../lib/component-mode"

export type SkeletonClassNameMode = "merge" | "replace"

export type SkeletonClassResolver = (params: {
  defaultClassName: string
  className?: string
}) => string

export type SkeletonProps = React.ComponentProps<"div"> & {
  mode?: BaseMode
  classNameMode?: SkeletonClassNameMode
  classResolver?: SkeletonClassResolver
}
