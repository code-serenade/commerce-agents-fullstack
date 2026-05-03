import * as React from "react"
import { Slider as SliderPrimitive } from "radix-ui"

import type { BaseMode } from "../../lib/component-mode"

export type SliderClassNameMode = "merge" | "replace"

export type SliderClassResolver = (params: {
  defaultClassName: string
  className?: string
}) => string

export type SliderProps = React.ComponentProps<typeof SliderPrimitive.Root> & {
  mode?: BaseMode
  classNameMode?: SliderClassNameMode
  classResolver?: SliderClassResolver
  trackClassName?: string
  trackClassNameMode?: SliderClassNameMode
  trackClassResolver?: SliderClassResolver
  rangeClassName?: string
  rangeClassNameMode?: SliderClassNameMode
  rangeClassResolver?: SliderClassResolver
  thumbClassName?: string
  thumbClassNameMode?: SliderClassNameMode
  thumbClassResolver?: SliderClassResolver
}
