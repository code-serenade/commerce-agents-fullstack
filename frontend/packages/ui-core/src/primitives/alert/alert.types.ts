import * as React from "react"
import type { VariantProps } from "class-variance-authority"

import type { BaseMode } from "../../lib/component-mode"
import { alertVariants } from "./alert.styles"

export type AlertVariantProps = VariantProps<typeof alertVariants>
export type AlertVariant = NonNullable<AlertVariantProps["variant"]>

export type AlertClassResolver = (params: {
  variant: AlertVariant
  defaultClassName: string
  className?: string
}) => string

export type AlertSlotClassResolver = (params: {
  defaultClassName: string
  className?: string
}) => string

export type AlertProps = React.ComponentProps<"div"> &
  AlertVariantProps & {
    mode?: BaseMode
    classNameMode?: "merge" | "replace"
    classResolver?: AlertClassResolver
  }

export type AlertTitleProps = React.ComponentProps<"div"> & {
  mode?: BaseMode
  classNameMode?: "merge" | "replace"
  classResolver?: AlertSlotClassResolver
}

export type AlertDescriptionProps = React.ComponentProps<"div"> & {
  mode?: BaseMode
  classNameMode?: "merge" | "replace"
  classResolver?: AlertSlotClassResolver
}

export type AlertActionProps = React.ComponentProps<"div"> & {
  mode?: BaseMode
  classNameMode?: "merge" | "replace"
  classResolver?: AlertSlotClassResolver
}
