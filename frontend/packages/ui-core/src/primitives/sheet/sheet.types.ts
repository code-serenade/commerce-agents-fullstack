import * as React from "react"
import { Dialog as SheetPrimitive } from "radix-ui"

import type { BaseMode } from "../../lib/component-mode"

export type SheetClassNameMode = "merge" | "replace"

export type SheetClassResolver = (params: {
  defaultClassName: string
  className?: string
}) => string

export type SheetSide = "top" | "right" | "bottom" | "left"

export type SheetProps = React.ComponentProps<typeof SheetPrimitive.Root> & {
  mode?: BaseMode
}

export type SheetTriggerProps = React.ComponentProps<
  typeof SheetPrimitive.Trigger
> & {
  mode?: BaseMode
}

export type SheetCloseProps = React.ComponentProps<
  typeof SheetPrimitive.Close
> & {
  mode?: BaseMode
}

export type SheetPortalProps = React.ComponentProps<
  typeof SheetPrimitive.Portal
> & {
  mode?: BaseMode
}

export type SheetOverlayProps = React.ComponentProps<
  typeof SheetPrimitive.Overlay
> & {
  mode?: BaseMode
  classNameMode?: SheetClassNameMode
  classResolver?: SheetClassResolver
}

export type SheetContentProps = React.ComponentProps<
  typeof SheetPrimitive.Content
> & {
  mode?: BaseMode
  side?: SheetSide
  showCloseButton?: boolean
  classNameMode?: SheetClassNameMode
  classResolver?: SheetClassResolver
  overlayClassName?: string
  overlayClassNameMode?: SheetClassNameMode
  overlayClassResolver?: SheetClassResolver
  closeButtonClassName?: string
  closeButtonClassNameMode?: SheetClassNameMode
  closeButtonClassResolver?: SheetClassResolver
  closeTextClassName?: string
  closeTextClassNameMode?: SheetClassNameMode
  closeTextClassResolver?: SheetClassResolver
}

export type SheetHeaderProps = React.ComponentProps<"div"> & {
  mode?: BaseMode
  classNameMode?: SheetClassNameMode
  classResolver?: SheetClassResolver
}

export type SheetFooterProps = React.ComponentProps<"div"> & {
  mode?: BaseMode
  classNameMode?: SheetClassNameMode
  classResolver?: SheetClassResolver
}

export type SheetTitleProps = React.ComponentProps<
  typeof SheetPrimitive.Title
> & {
  mode?: BaseMode
  classNameMode?: SheetClassNameMode
  classResolver?: SheetClassResolver
}

export type SheetDescriptionProps = React.ComponentProps<
  typeof SheetPrimitive.Description
> & {
  mode?: BaseMode
  classNameMode?: SheetClassNameMode
  classResolver?: SheetClassResolver
}
