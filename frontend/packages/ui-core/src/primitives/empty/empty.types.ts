import * as React from "react"
import type { VariantProps } from "class-variance-authority"

import type { BaseMode } from "../../lib/component-mode"
import { emptyMediaVariants } from "./empty.styles"

export type EmptyClassNameMode = "merge" | "replace"

export type EmptyClassResolver = (params: {
  defaultClassName: string
  className?: string
}) => string

export type EmptyMediaVariantProps = VariantProps<typeof emptyMediaVariants>
export type EmptyMediaVariant = NonNullable<EmptyMediaVariantProps["variant"]>

export type EmptyMediaClassResolver = (params: {
  variant: EmptyMediaVariant
  defaultClassName: string
  className?: string
}) => string

export type EmptyProps = React.ComponentProps<"div"> & {
  mode?: BaseMode
  classNameMode?: EmptyClassNameMode
  classResolver?: EmptyClassResolver
}

export type EmptyHeaderProps = React.ComponentProps<"div"> & {
  mode?: BaseMode
  classNameMode?: EmptyClassNameMode
  classResolver?: EmptyClassResolver
}

export type EmptyMediaProps = React.ComponentProps<"div"> &
  EmptyMediaVariantProps & {
    mode?: BaseMode
    classNameMode?: EmptyClassNameMode
    classResolver?: EmptyMediaClassResolver
  }

export type EmptyTitleProps = React.ComponentProps<"div"> & {
  mode?: BaseMode
  classNameMode?: EmptyClassNameMode
  classResolver?: EmptyClassResolver
}

export type EmptyDescriptionProps = React.ComponentProps<"p"> & {
  mode?: BaseMode
  classNameMode?: EmptyClassNameMode
  classResolver?: EmptyClassResolver
}

export type EmptyContentProps = React.ComponentProps<"div"> & {
  mode?: BaseMode
  classNameMode?: EmptyClassNameMode
  classResolver?: EmptyClassResolver
}
