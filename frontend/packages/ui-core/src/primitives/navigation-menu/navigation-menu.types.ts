import * as React from "react"
import { NavigationMenu as NavigationMenuPrimitive } from "radix-ui"

import type { BaseMode } from "../../lib/component-mode"

export type NavigationMenuClassNameMode = "merge" | "replace"

export type NavigationMenuClassResolver = (params: {
  defaultClassName: string
  className?: string
}) => string

export type NavigationMenuProps = React.ComponentProps<
  typeof NavigationMenuPrimitive.Root
> & {
  mode?: BaseMode
  viewport?: boolean
  classNameMode?: NavigationMenuClassNameMode
  classResolver?: NavigationMenuClassResolver
}

export type NavigationMenuListProps = React.ComponentProps<
  typeof NavigationMenuPrimitive.List
> & {
  mode?: BaseMode
  classNameMode?: NavigationMenuClassNameMode
  classResolver?: NavigationMenuClassResolver
}

export type NavigationMenuItemProps = React.ComponentProps<
  typeof NavigationMenuPrimitive.Item
> & {
  mode?: BaseMode
  classNameMode?: NavigationMenuClassNameMode
  classResolver?: NavigationMenuClassResolver
}

export type NavigationMenuTriggerProps = React.ComponentProps<
  typeof NavigationMenuPrimitive.Trigger
> & {
  mode?: BaseMode
  classNameMode?: NavigationMenuClassNameMode
  classResolver?: NavigationMenuClassResolver
  iconClassName?: string
  iconClassNameMode?: NavigationMenuClassNameMode
  iconClassResolver?: NavigationMenuClassResolver
}

export type NavigationMenuContentProps = React.ComponentProps<
  typeof NavigationMenuPrimitive.Content
> & {
  mode?: BaseMode
  classNameMode?: NavigationMenuClassNameMode
  classResolver?: NavigationMenuClassResolver
}

export type NavigationMenuViewportProps = React.ComponentProps<
  typeof NavigationMenuPrimitive.Viewport
> & {
  mode?: BaseMode
  classNameMode?: NavigationMenuClassNameMode
  classResolver?: NavigationMenuClassResolver
  containerClassName?: string
  containerClassNameMode?: NavigationMenuClassNameMode
  containerClassResolver?: NavigationMenuClassResolver
}

export type NavigationMenuLinkProps = React.ComponentProps<
  typeof NavigationMenuPrimitive.Link
> & {
  mode?: BaseMode
  classNameMode?: NavigationMenuClassNameMode
  classResolver?: NavigationMenuClassResolver
}

export type NavigationMenuIndicatorProps = React.ComponentProps<
  typeof NavigationMenuPrimitive.Indicator
> & {
  mode?: BaseMode
  classNameMode?: NavigationMenuClassNameMode
  classResolver?: NavigationMenuClassResolver
  arrowClassName?: string
  arrowClassNameMode?: NavigationMenuClassNameMode
  arrowClassResolver?: NavigationMenuClassResolver
}
