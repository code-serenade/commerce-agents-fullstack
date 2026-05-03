import * as React from "react"
import { Select as SelectPrimitive } from "radix-ui"
import type { BaseMode } from "../../lib/component-mode"

export type SelectProps = React.ComponentProps<typeof SelectPrimitive.Root> & {
  mode?: BaseMode
}
export type SelectClassNameMode = "merge" | "replace"

export type SelectClassResolver = (params: {
  defaultClassName: string
  className?: string
}) => string

export type SelectGroupProps = React.ComponentProps<
  typeof SelectPrimitive.Group
> & {
  mode?: BaseMode
  classNameMode?: SelectClassNameMode
  classResolver?: SelectClassResolver
}

export type SelectValueProps = React.ComponentProps<
  typeof SelectPrimitive.Value
> & {
  mode?: BaseMode
}

export type SelectTriggerProps = React.ComponentProps<
  typeof SelectPrimitive.Trigger
> & {
  size?: "sm" | "default"
  hideIndicator?: boolean
  indicator?: React.ReactNode
  indicatorClassName?: string
  mode?: BaseMode
  classNameMode?: SelectClassNameMode
  classResolver?: (params: {
    defaultClassName: string
    className?: string
    size: "sm" | "default"
  }) => string
  indicatorClassResolver?: SelectClassResolver
}

export type SelectContentProps = React.ComponentProps<
  typeof SelectPrimitive.Content
> & {
  showScrollButtons?: boolean
  mode?: BaseMode
  classNameMode?: SelectClassNameMode
  classResolver?: (params: {
    defaultClassName: string
    className?: string
    position: React.ComponentProps<typeof SelectPrimitive.Content>["position"]
  }) => string
}

export type SelectLabelProps = React.ComponentProps<
  typeof SelectPrimitive.Label
> & {
  mode?: BaseMode
  classNameMode?: SelectClassNameMode
  classResolver?: SelectClassResolver
}

export type SelectItemProps = React.ComponentProps<
  typeof SelectPrimitive.Item
> & {
  hideIndicator?: boolean
  indicator?: React.ReactNode
  indicatorContainerClassName?: string
  mode?: BaseMode
  classNameMode?: SelectClassNameMode
  classResolver?: SelectClassResolver
  indicatorContainerClassResolver?: SelectClassResolver
}

export type SelectSeparatorProps = React.ComponentProps<
  typeof SelectPrimitive.Separator
> & {
  mode?: BaseMode
  classNameMode?: SelectClassNameMode
  classResolver?: SelectClassResolver
}

export type SelectScrollUpButtonProps = React.ComponentProps<
  typeof SelectPrimitive.ScrollUpButton
> & {
  mode?: BaseMode
  classNameMode?: SelectClassNameMode
  classResolver?: SelectClassResolver
}

export type SelectScrollDownButtonProps = React.ComponentProps<
  typeof SelectPrimitive.ScrollDownButton
> & {
  mode?: BaseMode
  classNameMode?: SelectClassNameMode
  classResolver?: SelectClassResolver
}
