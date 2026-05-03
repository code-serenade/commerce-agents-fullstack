import * as React from "react"
import { Slider as SliderPrimitive } from "radix-ui"

import { DEFAULT_MODE } from "../../lib/component-mode"
import { cn } from "../../lib/utils"
import { sliderClassNames } from "./slider.styles"
import type { SliderClassResolver, SliderProps } from "./slider.types"

function resolveStyledSliderClassName({
  className,
  defaultClassName,
  classNameMode,
  classResolver,
}: {
  className?: string
  defaultClassName: string
  classNameMode: "merge" | "replace"
  classResolver?: SliderClassResolver
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

function Slider({
  mode = DEFAULT_MODE,
  className,
  defaultValue,
  value,
  min = 0,
  max = 100,
  classNameMode = "merge",
  classResolver,
  trackClassName,
  trackClassNameMode = "merge",
  trackClassResolver,
  rangeClassName,
  rangeClassNameMode = "merge",
  rangeClassResolver,
  thumbClassName,
  thumbClassNameMode = "merge",
  thumbClassResolver,
  ...props
}: SliderProps) {
  const _values = React.useMemo(
    () =>
      Array.isArray(value)
        ? value
        : Array.isArray(defaultValue)
          ? defaultValue
          : [min, max],
    [value, defaultValue, min, max]
  )

  if (mode === "primitive") {
    return (
      <SliderPrimitive.Root
        defaultValue={defaultValue}
        value={value}
        min={min}
        max={max}
        className={className}
        {...props}
      >
        <SliderPrimitive.Track className={trackClassName}>
          <SliderPrimitive.Range className={rangeClassName} />
        </SliderPrimitive.Track>
        {Array.from({ length: _values.length }, (_, index) => (
          <SliderPrimitive.Thumb key={index} className={thumbClassName} />
        ))}
      </SliderPrimitive.Root>
    )
  }

  return (
    <SliderPrimitive.Root
      data-slot="slider"
      defaultValue={defaultValue}
      value={value}
      min={min}
      max={max}
      className={resolveStyledSliderClassName({
        className,
        defaultClassName: sliderClassNames.slot0,
        classNameMode,
        classResolver,
      })}
      {...props}
    >
      <span aria-hidden="true" className={sliderClassNames.slot4} />
      <SliderPrimitive.Track
        data-slot="slider-track"
        className={resolveStyledSliderClassName({
          className: trackClassName,
          defaultClassName: sliderClassNames.slot1,
          classNameMode: trackClassNameMode,
          classResolver: trackClassResolver,
        })}
      >
        <SliderPrimitive.Range
          data-slot="slider-range"
          className={resolveStyledSliderClassName({
            className: rangeClassName,
            defaultClassName: sliderClassNames.slot2,
            classNameMode: rangeClassNameMode,
            classResolver: rangeClassResolver,
          })}
        />
      </SliderPrimitive.Track>
      {Array.from({ length: _values.length }, (_, index) => (
        <SliderPrimitive.Thumb
          data-slot="slider-thumb"
          key={index}
          className={resolveStyledSliderClassName({
            className: thumbClassName,
            defaultClassName: sliderClassNames.slot3,
            classNameMode: thumbClassNameMode,
            classResolver: thumbClassResolver,
          })}
        />
      ))}
    </SliderPrimitive.Root>
  )
}

export { Slider }
