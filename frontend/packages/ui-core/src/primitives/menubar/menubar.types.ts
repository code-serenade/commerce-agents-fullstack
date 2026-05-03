import * as React from "react"
import { Menubar as MenubarPrimitive } from "radix-ui"

import type { BaseMode } from "../../lib/component-mode"

export type MenubarClassNameMode = "merge" | "replace"

export type MenubarClassResolver = (params: {
  defaultClassName: string
  className?: string
}) => string

export type MenubarProps = React.ComponentProps<
  typeof MenubarPrimitive.Root
> & {
  mode?: BaseMode
  classNameMode?: MenubarClassNameMode
  classResolver?: MenubarClassResolver
}

export type MenubarMenuProps = React.ComponentProps<
  typeof MenubarPrimitive.Menu
> & {
  mode?: BaseMode
}

export type MenubarGroupProps = React.ComponentProps<
  typeof MenubarPrimitive.Group
> & {
  mode?: BaseMode
}

export type MenubarPortalProps = React.ComponentProps<
  typeof MenubarPrimitive.Portal
> & {
  mode?: BaseMode
}

export type MenubarRadioGroupProps = React.ComponentProps<
  typeof MenubarPrimitive.RadioGroup
> & {
  mode?: BaseMode
}

export type MenubarTriggerProps = React.ComponentProps<
  typeof MenubarPrimitive.Trigger
> & {
  mode?: BaseMode
  classNameMode?: MenubarClassNameMode
  classResolver?: MenubarClassResolver
}

export type MenubarContentProps = React.ComponentProps<
  typeof MenubarPrimitive.Content
> & {
  mode?: BaseMode
  classNameMode?: MenubarClassNameMode
  classResolver?: MenubarClassResolver
}

export type MenubarItemVariant = "default" | "destructive"

export type MenubarItemProps = React.ComponentProps<
  typeof MenubarPrimitive.Item
> & {
  mode?: BaseMode
  inset?: boolean
  variant?: MenubarItemVariant
  classNameMode?: MenubarClassNameMode
  classResolver?: MenubarClassResolver
}

export type MenubarCheckboxItemProps = React.ComponentProps<
  typeof MenubarPrimitive.CheckboxItem
> & {
  mode?: BaseMode
  inset?: boolean
  classNameMode?: MenubarClassNameMode
  classResolver?: MenubarClassResolver
  indicatorClassName?: string
  indicatorClassNameMode?: MenubarClassNameMode
  indicatorClassResolver?: MenubarClassResolver
}

export type MenubarRadioItemProps = React.ComponentProps<
  typeof MenubarPrimitive.RadioItem
> & {
  mode?: BaseMode
  inset?: boolean
  classNameMode?: MenubarClassNameMode
  classResolver?: MenubarClassResolver
  indicatorClassName?: string
  indicatorClassNameMode?: MenubarClassNameMode
  indicatorClassResolver?: MenubarClassResolver
}

export type MenubarLabelProps = React.ComponentProps<
  typeof MenubarPrimitive.Label
> & {
  mode?: BaseMode
  inset?: boolean
  classNameMode?: MenubarClassNameMode
  classResolver?: MenubarClassResolver
}

export type MenubarSeparatorProps = React.ComponentProps<
  typeof MenubarPrimitive.Separator
> & {
  mode?: BaseMode
  classNameMode?: MenubarClassNameMode
  classResolver?: MenubarClassResolver
}

export type MenubarShortcutProps = React.ComponentProps<"span"> & {
  mode?: BaseMode
  classNameMode?: MenubarClassNameMode
  classResolver?: MenubarClassResolver
}

export type MenubarSubProps = React.ComponentProps<
  typeof MenubarPrimitive.Sub
> & {
  mode?: BaseMode
}

export type MenubarSubTriggerProps = React.ComponentProps<
  typeof MenubarPrimitive.SubTrigger
> & {
  mode?: BaseMode
  inset?: boolean
  classNameMode?: MenubarClassNameMode
  classResolver?: MenubarClassResolver
  iconClassName?: string
  iconClassNameMode?: MenubarClassNameMode
  iconClassResolver?: MenubarClassResolver
}

export type MenubarSubContentProps = React.ComponentProps<
  typeof MenubarPrimitive.SubContent
> & {
  mode?: BaseMode
  classNameMode?: MenubarClassNameMode
  classResolver?: MenubarClassResolver
}
