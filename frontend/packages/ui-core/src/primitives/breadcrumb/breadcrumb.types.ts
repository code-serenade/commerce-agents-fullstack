import * as React from "react"

import type { BaseMode } from "../../lib/component-mode"

export type BreadcrumbClassNameMode = "merge" | "replace"

export type BreadcrumbClassResolver = (params: {
  defaultClassName: string
  className?: string
}) => string

export type BreadcrumbProps = React.ComponentProps<"nav"> & {
  mode?: BaseMode
  classNameMode?: BreadcrumbClassNameMode
  classResolver?: BreadcrumbClassResolver
}

export type BreadcrumbListProps = React.ComponentProps<"ol"> & {
  mode?: BaseMode
  classNameMode?: BreadcrumbClassNameMode
  classResolver?: BreadcrumbClassResolver
}

export type BreadcrumbItemProps = React.ComponentProps<"li"> & {
  mode?: BaseMode
  classNameMode?: BreadcrumbClassNameMode
  classResolver?: BreadcrumbClassResolver
}

export type BreadcrumbLinkProps = React.ComponentProps<"a"> & {
  asChild?: boolean
  mode?: BaseMode
  classNameMode?: BreadcrumbClassNameMode
  classResolver?: BreadcrumbClassResolver
}

export type BreadcrumbPageProps = React.ComponentProps<"span"> & {
  mode?: BaseMode
  classNameMode?: BreadcrumbClassNameMode
  classResolver?: BreadcrumbClassResolver
}

export type BreadcrumbSeparatorProps = React.ComponentProps<"li"> & {
  mode?: BaseMode
  classNameMode?: BreadcrumbClassNameMode
  classResolver?: BreadcrumbClassResolver
}

export type BreadcrumbEllipsisProps = React.ComponentProps<"span"> & {
  mode?: BaseMode
  classNameMode?: BreadcrumbClassNameMode
  classResolver?: BreadcrumbClassResolver
}
