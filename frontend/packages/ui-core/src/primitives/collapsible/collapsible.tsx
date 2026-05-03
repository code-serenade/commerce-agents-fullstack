"use client"

import { Collapsible as CollapsiblePrimitive } from "radix-ui"
import { DEFAULT_MODE } from "../../lib/component-mode"
import type {
  CollapsibleContentProps,
  CollapsibleProps,
  CollapsibleTriggerProps,
} from "./collapsible.types"

function Collapsible({ mode = DEFAULT_MODE, ...props }: CollapsibleProps) {
  if (mode === "primitive") {
    return <CollapsiblePrimitive.Root {...props} />
  }

  return <CollapsiblePrimitive.Root data-slot="collapsible" {...props} />
}

function CollapsibleTrigger({
  mode = DEFAULT_MODE,
  ...props
}: CollapsibleTriggerProps) {
  if (mode === "primitive") {
    return <CollapsiblePrimitive.CollapsibleTrigger {...props} />
  }

  return (
    <CollapsiblePrimitive.CollapsibleTrigger
      data-slot="collapsible-trigger"
      {...props}
    />
  )
}

function CollapsibleContent({
  mode = DEFAULT_MODE,
  ...props
}: CollapsibleContentProps) {
  if (mode === "primitive") {
    return <CollapsiblePrimitive.CollapsibleContent {...props} />
  }

  return (
    <CollapsiblePrimitive.CollapsibleContent
      data-slot="collapsible-content"
      {...props}
    />
  )
}

export { Collapsible, CollapsibleTrigger, CollapsibleContent }
