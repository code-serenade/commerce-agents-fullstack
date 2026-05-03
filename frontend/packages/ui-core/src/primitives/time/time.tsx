import * as React from "react"

import { DEFAULT_MODE } from "../../lib/component-mode"
import { cn } from "../../lib/utils"
import { timeClassNames } from "./time.styles"
import type {
  TimeClassResolver,
  TimeProps,
  TimeSegment,
  TimeValue,
} from "./time.types"

const HOUR_OPTIONS = Array.from({ length: 24 }, (_, index) =>
  String(index).padStart(2, "0")
)
const MINUTE_SECOND_OPTIONS = Array.from({ length: 60 }, (_, index) =>
  String(index).padStart(2, "0")
)

function resolveStyledClassName({
  className,
  defaultClassName,
  classNameMode,
  classResolver,
}: {
  className?: string
  defaultClassName: string
  classNameMode: "merge" | "replace"
  classResolver?: TimeClassResolver
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

function normalizeUnit(value: string | undefined, max: number) {
  const numeric = Number.parseInt(value ?? "", 10)

  if (Number.isNaN(numeric)) {
    return "00"
  }

  return String(Math.min(Math.max(numeric, 0), max)).padStart(2, "0")
}

function normalizeValue(
  value: TimeValue | undefined,
  showSeconds: boolean
): TimeValue {
  return {
    hour: normalizeUnit(value?.hour, 23),
    minute: normalizeUnit(value?.minute, 59),
    second: showSeconds ? normalizeUnit(value?.second, 59) : undefined,
  }
}

function getOptions(segment: TimeSegment) {
  return segment === "hour" ? HOUR_OPTIONS : MINUTE_SECOND_OPTIONS
}

function getValueBySegment(value: TimeValue, segment: TimeSegment) {
  if (segment === "hour") {
    return value.hour
  }

  if (segment === "minute") {
    return value.minute
  }

  return value.second ?? "00"
}

function setValueBySegment(
  value: TimeValue,
  segment: TimeSegment,
  nextSegmentValue: string
): TimeValue {
  if (segment === "hour") {
    return { ...value, hour: nextSegmentValue }
  }

  if (segment === "minute") {
    return { ...value, minute: nextSegmentValue }
  }

  return { ...value, second: nextSegmentValue }
}

function getSegmentLabel(
  segment: TimeSegment,
  props: Pick<TimeProps, "hourLabel" | "minuteLabel" | "secondLabel">
) {
  if (segment === "hour") {
    return props.hourLabel ?? "Hours"
  }

  if (segment === "minute") {
    return props.minuteLabel ?? "Minutes"
  }

  return props.secondLabel ?? "Seconds"
}

function Time({
  mode = DEFAULT_MODE,
  value,
  defaultValue,
  onValueChange,
  disabled = false,
  showSeconds = true,
  visibleRows = 5,
  itemHeight = 44,
  className,
  columnClassName,
  itemClassName,
  selectionClassName,
  classNameMode = "merge",
  classResolver,
  columnClassNameMode = "merge",
  columnClassResolver,
  itemClassNameMode = "merge",
  itemClassResolver,
  selectionClassNameMode = "merge",
  selectionClassResolver,
  hourLabel,
  minuteLabel,
  secondLabel,
  ...props
}: TimeProps) {
  const segments = React.useMemo(
    () =>
      (showSeconds
        ? ["hour", "minute", "second"]
        : ["hour", "minute"]) as TimeSegment[],
    [showSeconds]
  )

  const isControlled = value !== undefined
  const [internalValue, setInternalValue] = React.useState<TimeValue>(() =>
    normalizeValue(defaultValue, showSeconds)
  )
  const resolvedValue = isControlled
    ? normalizeValue(value, showSeconds)
    : normalizeValue(internalValue, showSeconds)

  const columnRefs = React.useRef<Record<string, HTMLDivElement | null>>({})
  const syncLockRef = React.useRef(false)

  React.useEffect(() => {
    if (!showSeconds) {
      setInternalValue((currentValue) => ({
        ...currentValue,
        second: undefined,
      }))
    }
  }, [showSeconds])

  const emitValueChange = React.useCallback(
    (nextValue: TimeValue) => {
      const normalizedNextValue = normalizeValue(nextValue, showSeconds)

      if (!isControlled) {
        setInternalValue(normalizedNextValue)
      }

      onValueChange?.(normalizedNextValue)
    },
    [isControlled, onValueChange, showSeconds]
  )

  const scrollToValue = React.useCallback(
    (segment: TimeSegment, segmentValue: string, behavior: ScrollBehavior) => {
      const node = columnRefs.current[segment]
      if (!node) {
        return
      }

      const index = getOptions(segment).indexOf(segmentValue)
      if (index < 0) {
        return
      }

      syncLockRef.current = true
      node.scrollTo({
        top: index * itemHeight,
        behavior,
      })

      window.setTimeout(
        () => {
          syncLockRef.current = false
        },
        behavior === "smooth" ? 180 : 0
      )
    },
    [itemHeight]
  )

  React.useEffect(() => {
    segments.forEach((segment) => {
      scrollToValue(segment, getValueBySegment(resolvedValue, segment), "auto")
    })
  }, [resolvedValue, scrollToValue, segments])

  const handleSegmentChange = React.useCallback(
    (segment: TimeSegment, nextSegmentValue: string) => {
      const currentSegmentValue = getValueBySegment(resolvedValue, segment)
      if (currentSegmentValue === nextSegmentValue) {
        return
      }

      emitValueChange(
        setValueBySegment(resolvedValue, segment, nextSegmentValue)
      )
    },
    [emitValueChange, resolvedValue]
  )

  const handleScroll = React.useCallback(
    (segment: TimeSegment) => (event: React.UIEvent<HTMLDivElement>) => {
      if (disabled || syncLockRef.current) {
        return
      }

      const options = getOptions(segment)
      const nextIndex = Math.min(
        Math.max(Math.round(event.currentTarget.scrollTop / itemHeight), 0),
        options.length - 1
      )

      handleSegmentChange(segment, options[nextIndex] ?? options[0] ?? "00")
    },
    [disabled, handleSegmentChange, itemHeight]
  )

  const handleKeyDown = React.useCallback(
    (segment: TimeSegment) => (event: React.KeyboardEvent<HTMLDivElement>) => {
      if (disabled) {
        return
      }

      if (event.key !== "ArrowUp" && event.key !== "ArrowDown") {
        return
      }

      event.preventDefault()

      const options = getOptions(segment)
      const currentIndex = options.indexOf(
        getValueBySegment(resolvedValue, segment)
      )
      const delta = event.key === "ArrowUp" ? -1 : 1
      const nextIndex = Math.min(
        Math.max(currentIndex + delta, 0),
        options.length - 1
      )
      const nextValue = options[nextIndex]

      if (!nextValue) {
        return
      }

      handleSegmentChange(segment, nextValue)
      scrollToValue(segment, nextValue, "smooth")
    },
    [disabled, handleSegmentChange, resolvedValue, scrollToValue]
  )

  const rootStyle = {
    "--time-item-height": `${itemHeight}px`,
    "--time-visible-rows": String(visibleRows),
  } as React.CSSProperties

  const resolvedRootClassName =
    mode === "primitive"
      ? className
      : resolveStyledClassName({
          className,
          defaultClassName: timeClassNames.root,
          classNameMode,
          classResolver,
        })

  const resolvedColumnClassName =
    mode === "primitive"
      ? columnClassName
      : resolveStyledClassName({
          className: columnClassName,
          defaultClassName: timeClassNames.column,
          classNameMode: columnClassNameMode,
          classResolver: columnClassResolver,
        })

  const resolvedItemClassName =
    mode === "primitive"
      ? itemClassName
      : resolveStyledClassName({
          className: itemClassName,
          defaultClassName: timeClassNames.item,
          classNameMode: itemClassNameMode,
          classResolver: itemClassResolver,
        })

  const resolvedSelectionClassName =
    mode === "primitive"
      ? selectionClassName
      : resolveStyledClassName({
          className: selectionClassName,
          defaultClassName: timeClassNames.selection,
          classNameMode: selectionClassNameMode,
          classResolver: selectionClassResolver,
        })

  return (
    <div
      className={cn("relative", resolvedRootClassName)}
      style={rootStyle}
      role="group"
      aria-label={props["aria-label"]}
      aria-disabled={disabled || undefined}
    >
      <div
        aria-hidden="true"
        className={cn(
          "pointer-events-none absolute top-1/2 z-10 -translate-y-1/2",
          resolvedSelectionClassName
        )}
      />

      {segments.map((segment) => {
        const options = getOptions(segment)
        const currentValue = getValueBySegment(resolvedValue, segment)

        return (
          <div key={segment} className="relative">
            <div
              ref={(node) => {
                columnRefs.current[segment] = node
              }}
              className={resolvedColumnClassName}
              onScroll={handleScroll(segment)}
              onKeyDown={handleKeyDown(segment)}
              tabIndex={disabled ? -1 : 0}
              role="listbox"
              aria-label={getSegmentLabel(segment, {
                hourLabel,
                minuteLabel,
                secondLabel,
              })}
              aria-disabled={disabled || undefined}
            >
              <div
                className="relative z-20"
                style={{
                  paddingBlock: `${((visibleRows - 1) * itemHeight) / 2}px`,
                }}
              >
                {options.map((option) => (
                  <div
                    key={`${segment}-${option}`}
                    role="option"
                    aria-selected={currentValue === option}
                    className={resolvedItemClassName}
                  >
                    {option}
                  </div>
                ))}
              </div>
            </div>
          </div>
        )
      })}
    </div>
  )
}

export { Time }
