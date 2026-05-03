import * as React from "react"
import type { VariantProps } from "class-variance-authority"
import { Tabs as TabsPrimitive } from "radix-ui"

import type { BaseMode } from "../../lib/component-mode"
import { tabsListVariants } from "./tabs.styles"

export type TabsClassNameMode = "merge" | "replace"

export type TabsClassResolver = (params: {
  defaultClassName: string
  className?: string
}) => string

export type TabsListVariantProps = VariantProps<typeof tabsListVariants>
export type TabsListVariant = NonNullable<TabsListVariantProps["variant"]>

export type TabsListClassResolver = (params: {
  variant: TabsListVariant
  defaultClassName: string
  className?: string
}) => string

export type TabsProps = React.ComponentProps<typeof TabsPrimitive.Root> & {
  mode?: BaseMode
  classNameMode?: TabsClassNameMode
  classResolver?: TabsClassResolver
}

export type TabsListProps = React.ComponentProps<typeof TabsPrimitive.List> &
  TabsListVariantProps & {
    mode?: BaseMode
    classNameMode?: TabsClassNameMode
    classResolver?: TabsListClassResolver
  }

export type TabsTriggerProps = React.ComponentProps<
  typeof TabsPrimitive.Trigger
> & {
  mode?: BaseMode
  classNameMode?: TabsClassNameMode
  classResolver?: TabsClassResolver
}

export type TabsContentProps = React.ComponentProps<
  typeof TabsPrimitive.Content
> & {
  mode?: BaseMode
  classNameMode?: TabsClassNameMode
  classResolver?: TabsClassResolver
}
