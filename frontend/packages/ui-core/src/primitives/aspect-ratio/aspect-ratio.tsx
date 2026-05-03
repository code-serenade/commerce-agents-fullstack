"use client"

import { AspectRatio as AspectRatioPrimitive } from "radix-ui"

import { DEFAULT_MODE } from "../../lib/component-mode"
import type { AspectRatioProps } from "./aspect-ratio.types"

function AspectRatio({ mode = DEFAULT_MODE, ...props }: AspectRatioProps) {
  if (mode === "primitive") {
    return <AspectRatioPrimitive.Root {...props} />
  }

  return <AspectRatioPrimitive.Root data-slot="aspect-ratio" {...props} />
}

export { AspectRatio }
