import { Tabs as TabsPrimitive } from "radix-ui"

import { DEFAULT_MODE } from "../../lib/component-mode"
import { cn } from "../../lib/utils"
import { tabsClassNames, tabsListVariants } from "./tabs.styles"
import type {
  TabsClassResolver,
  TabsContentProps,
  TabsListClassResolver,
  TabsListProps,
  TabsListVariant,
  TabsProps,
  TabsTriggerProps,
} from "./tabs.types"

function resolveStyledTabsClassName({
  className,
  defaultClassName,
  classNameMode,
  classResolver,
}: {
  className?: string
  defaultClassName: string
  classNameMode: "merge" | "replace"
  classResolver?: TabsClassResolver
}) {
  if (classResolver) {
    return classResolver({
      defaultClassName,
      className,
    })
  }

  if (classNameMode === "replace") {
    return className ?? defaultClassName
  }

  return cn(defaultClassName, className)
}

function resolveStyledTabsListClassName({
  className,
  variant,
  classNameMode,
  classResolver,
}: {
  className?: string
  variant: TabsListVariant
  classNameMode: "merge" | "replace"
  classResolver?: TabsListClassResolver
}) {
  const defaultClassName = tabsListVariants({ variant })

  if (classResolver) {
    return classResolver({
      variant,
      defaultClassName,
      className,
    })
  }

  if (classNameMode === "replace") {
    return className ?? defaultClassName
  }

  return cn(defaultClassName, className)
}

function Tabs({
  mode = DEFAULT_MODE,
  className,
  orientation = "horizontal",
  classNameMode = "merge",
  classResolver,
  ...props
}: TabsProps) {
  if (mode === "primitive") {
    return (
      <TabsPrimitive.Root
        orientation={orientation}
        className={className}
        {...props}
      />
    )
  }

  return (
    <TabsPrimitive.Root
      data-slot="tabs"
      data-orientation={orientation}
      className={resolveStyledTabsClassName({
        className,
        defaultClassName: tabsClassNames.slot4,
        classNameMode,
        classResolver,
      })}
      {...props}
    />
  )
}

function TabsList({
  mode = DEFAULT_MODE,
  className,
  variant = "default",
  classNameMode = "merge",
  classResolver,
  ...props
}: TabsListProps) {
  if (mode === "primitive") {
    return <TabsPrimitive.List className={className} {...props} />
  }

  const resolvedVariant = (variant ?? "default") as TabsListVariant

  return (
    <TabsPrimitive.List
      data-slot="tabs-list"
      data-variant={resolvedVariant}
      className={resolveStyledTabsListClassName({
        className,
        variant: resolvedVariant,
        classNameMode,
        classResolver,
      })}
      {...props}
    />
  )
}

function TabsTrigger({
  mode = DEFAULT_MODE,
  className,
  classNameMode = "merge",
  classResolver,
  ...props
}: TabsTriggerProps) {
  if (mode === "primitive") {
    return <TabsPrimitive.Trigger className={className} {...props} />
  }

  return (
    <TabsPrimitive.Trigger
      data-slot="tabs-trigger"
      className={resolveStyledTabsClassName({
        className,
        defaultClassName: cn(
          tabsClassNames.slot5,
          tabsClassNames.slot6,
          tabsClassNames.slot7,
          tabsClassNames.slot8
        ),
        classNameMode,
        classResolver,
      })}
      {...props}
    />
  )
}

function TabsContent({
  mode = DEFAULT_MODE,
  className,
  classNameMode = "merge",
  classResolver,
  ...props
}: TabsContentProps) {
  if (mode === "primitive") {
    return <TabsPrimitive.Content className={className} {...props} />
  }

  return (
    <TabsPrimitive.Content
      data-slot="tabs-content"
      className={resolveStyledTabsClassName({
        className,
        defaultClassName: tabsClassNames.slot9,
        classNameMode,
        classResolver,
      })}
      {...props}
    />
  )
}

export { Tabs, TabsList, TabsTrigger, TabsContent }
