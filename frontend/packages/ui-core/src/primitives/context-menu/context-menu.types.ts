import * as React from "react"
import { ContextMenu as ContextMenuPrimitive } from "radix-ui"

import type { BaseMode } from "../../lib/component-mode"

export type ContextMenuClassNameMode = "merge" | "replace"

export type ContextMenuClassResolver = (params: {
  defaultClassName: string
  className?: string
}) => string

export type ContextMenuProps = React.ComponentProps<
  typeof ContextMenuPrimitive.Root
> & {
  mode?: BaseMode
}

export type ContextMenuTriggerProps = React.ComponentProps<
  typeof ContextMenuPrimitive.Trigger
> & {
  mode?: BaseMode
  classNameMode?: ContextMenuClassNameMode
  classResolver?: ContextMenuClassResolver
}

export type ContextMenuGroupProps = React.ComponentProps<
  typeof ContextMenuPrimitive.Group
> & {
  mode?: BaseMode
}

export type ContextMenuPortalProps = React.ComponentProps<
  typeof ContextMenuPrimitive.Portal
> & {
  mode?: BaseMode
}

export type ContextMenuSubProps = React.ComponentProps<
  typeof ContextMenuPrimitive.Sub
> & {
  mode?: BaseMode
}

export type ContextMenuRadioGroupProps = React.ComponentProps<
  typeof ContextMenuPrimitive.RadioGroup
> & {
  mode?: BaseMode
}

export type ContextMenuContentProps = React.ComponentProps<
  typeof ContextMenuPrimitive.Content
> & {
  mode?: BaseMode
  classNameMode?: ContextMenuClassNameMode
  classResolver?: ContextMenuClassResolver
}

export type ContextMenuItemVariant = "default" | "destructive"

export type ContextMenuItemProps = React.ComponentProps<
  typeof ContextMenuPrimitive.Item
> & {
  mode?: BaseMode
  inset?: boolean
  variant?: ContextMenuItemVariant
  classNameMode?: ContextMenuClassNameMode
  classResolver?: ContextMenuClassResolver
}

export type ContextMenuSubTriggerProps = React.ComponentProps<
  typeof ContextMenuPrimitive.SubTrigger
> & {
  mode?: BaseMode
  inset?: boolean
  classNameMode?: ContextMenuClassNameMode
  classResolver?: ContextMenuClassResolver
  iconClassName?: string
  iconClassNameMode?: ContextMenuClassNameMode
  iconClassResolver?: ContextMenuClassResolver
}

export type ContextMenuSubContentProps = React.ComponentProps<
  typeof ContextMenuPrimitive.SubContent
> & {
  mode?: BaseMode
  classNameMode?: ContextMenuClassNameMode
  classResolver?: ContextMenuClassResolver
}

export type ContextMenuCheckboxItemProps = React.ComponentProps<
  typeof ContextMenuPrimitive.CheckboxItem
> & {
  mode?: BaseMode
  inset?: boolean
  classNameMode?: ContextMenuClassNameMode
  classResolver?: ContextMenuClassResolver
  indicatorClassName?: string
  indicatorClassNameMode?: ContextMenuClassNameMode
  indicatorClassResolver?: ContextMenuClassResolver
}

export type ContextMenuRadioItemProps = React.ComponentProps<
  typeof ContextMenuPrimitive.RadioItem
> & {
  mode?: BaseMode
  inset?: boolean
  classNameMode?: ContextMenuClassNameMode
  classResolver?: ContextMenuClassResolver
  indicatorClassName?: string
  indicatorClassNameMode?: ContextMenuClassNameMode
  indicatorClassResolver?: ContextMenuClassResolver
}

export type ContextMenuLabelProps = React.ComponentProps<
  typeof ContextMenuPrimitive.Label
> & {
  mode?: BaseMode
  inset?: boolean
  classNameMode?: ContextMenuClassNameMode
  classResolver?: ContextMenuClassResolver
}

export type ContextMenuSeparatorProps = React.ComponentProps<
  typeof ContextMenuPrimitive.Separator
> & {
  mode?: BaseMode
  classNameMode?: ContextMenuClassNameMode
  classResolver?: ContextMenuClassResolver
}

export type ContextMenuShortcutProps = React.ComponentProps<"span"> & {
  mode?: BaseMode
  classNameMode?: ContextMenuClassNameMode
  classResolver?: ContextMenuClassResolver
}
