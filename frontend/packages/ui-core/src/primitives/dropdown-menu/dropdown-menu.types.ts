import * as React from "react"
import { DropdownMenu as DropdownMenuPrimitive } from "radix-ui"

import type { BaseMode } from "../../lib/component-mode"

export type DropdownMenuClassNameMode = "merge" | "replace"

export type DropdownMenuClassResolver = (params: {
  defaultClassName: string
  className?: string
}) => string

export type DropdownMenuProps = React.ComponentProps<
  typeof DropdownMenuPrimitive.Root
> & {
  mode?: BaseMode
}

export type DropdownMenuPortalProps = React.ComponentProps<
  typeof DropdownMenuPrimitive.Portal
> & {
  mode?: BaseMode
}

export type DropdownMenuTriggerProps = React.ComponentProps<
  typeof DropdownMenuPrimitive.Trigger
> & {
  mode?: BaseMode
  classNameMode?: DropdownMenuClassNameMode
  classResolver?: DropdownMenuClassResolver
}

export type DropdownMenuContentProps = React.ComponentProps<
  typeof DropdownMenuPrimitive.Content
> & {
  mode?: BaseMode
  classNameMode?: DropdownMenuClassNameMode
  classResolver?: DropdownMenuClassResolver
}

export type DropdownMenuGroupProps = React.ComponentProps<
  typeof DropdownMenuPrimitive.Group
> & {
  mode?: BaseMode
}

export type DropdownMenuItemVariant = "default" | "destructive"

export type DropdownMenuItemProps = React.ComponentProps<
  typeof DropdownMenuPrimitive.Item
> & {
  mode?: BaseMode
  inset?: boolean
  variant?: DropdownMenuItemVariant
  classNameMode?: DropdownMenuClassNameMode
  classResolver?: DropdownMenuClassResolver
}

export type DropdownMenuCheckboxItemProps = React.ComponentProps<
  typeof DropdownMenuPrimitive.CheckboxItem
> & {
  mode?: BaseMode
  inset?: boolean
  classNameMode?: DropdownMenuClassNameMode
  classResolver?: DropdownMenuClassResolver
  indicatorClassName?: string
  indicatorClassNameMode?: DropdownMenuClassNameMode
  indicatorClassResolver?: DropdownMenuClassResolver
}

export type DropdownMenuRadioGroupProps = React.ComponentProps<
  typeof DropdownMenuPrimitive.RadioGroup
> & {
  mode?: BaseMode
}

export type DropdownMenuRadioItemProps = React.ComponentProps<
  typeof DropdownMenuPrimitive.RadioItem
> & {
  mode?: BaseMode
  inset?: boolean
  classNameMode?: DropdownMenuClassNameMode
  classResolver?: DropdownMenuClassResolver
  indicatorClassName?: string
  indicatorClassNameMode?: DropdownMenuClassNameMode
  indicatorClassResolver?: DropdownMenuClassResolver
}

export type DropdownMenuLabelProps = React.ComponentProps<
  typeof DropdownMenuPrimitive.Label
> & {
  mode?: BaseMode
  inset?: boolean
  classNameMode?: DropdownMenuClassNameMode
  classResolver?: DropdownMenuClassResolver
}

export type DropdownMenuSeparatorProps = React.ComponentProps<
  typeof DropdownMenuPrimitive.Separator
> & {
  mode?: BaseMode
  classNameMode?: DropdownMenuClassNameMode
  classResolver?: DropdownMenuClassResolver
}

export type DropdownMenuShortcutProps = React.ComponentProps<"span"> & {
  mode?: BaseMode
  classNameMode?: DropdownMenuClassNameMode
  classResolver?: DropdownMenuClassResolver
}

export type DropdownMenuSubProps = React.ComponentProps<
  typeof DropdownMenuPrimitive.Sub
> & {
  mode?: BaseMode
}

export type DropdownMenuSubTriggerProps = React.ComponentProps<
  typeof DropdownMenuPrimitive.SubTrigger
> & {
  mode?: BaseMode
  inset?: boolean
  classNameMode?: DropdownMenuClassNameMode
  classResolver?: DropdownMenuClassResolver
  iconClassName?: string
  iconClassNameMode?: DropdownMenuClassNameMode
  iconClassResolver?: DropdownMenuClassResolver
}

export type DropdownMenuSubContentProps = React.ComponentProps<
  typeof DropdownMenuPrimitive.SubContent
> & {
  mode?: BaseMode
  classNameMode?: DropdownMenuClassNameMode
  classResolver?: DropdownMenuClassResolver
}
