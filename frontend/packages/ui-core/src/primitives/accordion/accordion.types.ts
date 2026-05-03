import * as React from "react"
import { Accordion as AccordionPrimitive } from "radix-ui"

import type { BaseMode } from "../../lib/component-mode"

export type AccordionClassNameMode = "merge" | "replace"

export type AccordionClassResolver = (params: {
  defaultClassName: string
  className?: string
}) => string

export type AccordionProps = React.ComponentProps<
  typeof AccordionPrimitive.Root
> & {
  mode?: BaseMode
  classNameMode?: AccordionClassNameMode
  classResolver?: AccordionClassResolver
}

export type AccordionItemProps = React.ComponentProps<
  typeof AccordionPrimitive.Item
> & {
  mode?: BaseMode
  classNameMode?: AccordionClassNameMode
  classResolver?: AccordionClassResolver
}

export type AccordionTriggerProps = React.ComponentProps<
  typeof AccordionPrimitive.Trigger
> & {
  mode?: BaseMode
  classNameMode?: AccordionClassNameMode
  classResolver?: AccordionClassResolver
  headerClassName?: string
  headerClassNameMode?: AccordionClassNameMode
  headerClassResolver?: AccordionClassResolver
  collapsedIconClassName?: string
  collapsedIconClassNameMode?: AccordionClassNameMode
  collapsedIconClassResolver?: AccordionClassResolver
  expandedIconClassName?: string
  expandedIconClassNameMode?: AccordionClassNameMode
  expandedIconClassResolver?: AccordionClassResolver
}

export type AccordionContentProps = React.ComponentProps<
  typeof AccordionPrimitive.Content
> & {
  mode?: BaseMode
  classNameMode?: AccordionClassNameMode
  classResolver?: AccordionClassResolver
  innerClassName?: string
  innerClassNameMode?: AccordionClassNameMode
  innerClassResolver?: AccordionClassResolver
}
