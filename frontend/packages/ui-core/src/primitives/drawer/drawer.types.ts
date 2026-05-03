import * as React from "react"
import { Drawer as DrawerPrimitive } from "vaul"

import type { BaseMode } from "../../lib/component-mode"

export type DrawerClassNameMode = "merge" | "replace"

export type DrawerClassResolver = (params: {
  defaultClassName: string
  className?: string
}) => string

export type DrawerProps = React.ComponentProps<typeof DrawerPrimitive.Root> & {
  mode?: BaseMode
}

export type DrawerTriggerProps = React.ComponentProps<
  typeof DrawerPrimitive.Trigger
> & {
  mode?: BaseMode
}

export type DrawerPortalProps = React.ComponentProps<
  typeof DrawerPrimitive.Portal
> & {
  mode?: BaseMode
}

export type DrawerCloseProps = React.ComponentProps<
  typeof DrawerPrimitive.Close
> & {
  mode?: BaseMode
}

export type DrawerOverlayProps = React.ComponentProps<
  typeof DrawerPrimitive.Overlay
> & {
  mode?: BaseMode
  classNameMode?: DrawerClassNameMode
  classResolver?: DrawerClassResolver
}

export type DrawerContentProps = React.ComponentProps<
  typeof DrawerPrimitive.Content
> & {
  mode?: BaseMode
  showHandle?: boolean
  classNameMode?: DrawerClassNameMode
  classResolver?: DrawerClassResolver
  overlayClassName?: string
  overlayClassNameMode?: DrawerClassNameMode
  overlayClassResolver?: DrawerClassResolver
  handleClassName?: string
  handleClassNameMode?: DrawerClassNameMode
  handleClassResolver?: DrawerClassResolver
}

export type DrawerHeaderProps = React.ComponentProps<"div"> & {
  mode?: BaseMode
  classNameMode?: DrawerClassNameMode
  classResolver?: DrawerClassResolver
}

export type DrawerFooterProps = React.ComponentProps<"div"> & {
  mode?: BaseMode
  classNameMode?: DrawerClassNameMode
  classResolver?: DrawerClassResolver
}

export type DrawerTitleProps = React.ComponentProps<
  typeof DrawerPrimitive.Title
> & {
  mode?: BaseMode
  classNameMode?: DrawerClassNameMode
  classResolver?: DrawerClassResolver
}

export type DrawerDescriptionProps = React.ComponentProps<
  typeof DrawerPrimitive.Description
> & {
  mode?: BaseMode
  classNameMode?: DrawerClassNameMode
  classResolver?: DrawerClassResolver
}
