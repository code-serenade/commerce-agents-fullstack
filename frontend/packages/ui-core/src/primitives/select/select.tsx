import { Select as SelectPrimitive } from "radix-ui"

import { DEFAULT_MODE } from "../../lib/component-mode"
import { CheckIcon, ChevronDownIcon, ChevronUpIcon } from "../../lib/icon-slots"
import { cn } from "../../lib/utils"
import {
  selectContentBaseClassName,
  selectContentPopperOffsetClassName,
  selectGroupClassName,
  selectItemClassName,
  selectItemIndicatorContainerClassName,
  selectItemStateClassName,
  selectLabelClassName,
  selectScrollButtonClassName,
  selectSeparatorClassName,
  selectTriggerClassName,
  selectTriggerIconClassName,
  selectTriggerSurfaceClassName,
  selectViewportClassName,
} from "./select.styles"
import type {
  SelectContentProps,
  SelectClassNameMode,
  SelectClassResolver,
  SelectGroupProps,
  SelectItemProps,
  SelectLabelProps,
  SelectProps,
  SelectScrollDownButtonProps,
  SelectScrollUpButtonProps,
  SelectSeparatorProps,
  SelectTriggerProps,
  SelectValueProps,
} from "./select.types"

function resolveStyledClassName({
  className,
  defaultClassName,
  classNameMode,
  classResolver,
}: {
  className?: string
  defaultClassName: string
  classNameMode?: SelectClassNameMode
  classResolver?: SelectClassResolver
}) {
  if (classResolver) {
    return classResolver({ defaultClassName, className })
  }

  if (classNameMode === "replace") {
    return className ?? defaultClassName
  }

  return cn(defaultClassName, className)
}

function DefaultSelectIndicator({ className }: { className?: string }) {
  return (
    <span className={className}>
      <span className="flex flex-col items-center justify-center leading-none">
        <ChevronUpIcon className="size-2.5" />
        <ChevronDownIcon className="-mt-1.25 size-2.5" />
      </span>
    </span>
  )
}

export function Select({ mode = DEFAULT_MODE, ...props }: SelectProps) {
  if (mode === "primitive") {
    return <SelectPrimitive.Root {...props} />
  }

  return <SelectPrimitive.Root data-slot="select" {...props} />
}

export function SelectGroup({
  className,
  mode = DEFAULT_MODE,
  classNameMode = "merge",
  classResolver,
  ...props
}: SelectGroupProps) {
  if (mode === "primitive") {
    const rest = { ...props }
    return <SelectPrimitive.Group className={className} {...rest} />
  }

  return (
    <SelectPrimitive.Group
      data-slot="select-group"
      className={resolveStyledClassName({
        className,
        defaultClassName: selectGroupClassName,
        classNameMode,
        classResolver,
      })}
      {...props}
    />
  )
}

export function SelectValue({
  mode = DEFAULT_MODE,
  ...props
}: SelectValueProps) {
  if (mode === "primitive") {
    return <SelectPrimitive.Value {...props} />
  }

  return <SelectPrimitive.Value data-slot="select-value" {...props} />
}

export function SelectTrigger({
  className,
  mode = DEFAULT_MODE,
  size = "default",
  children,
  hideIndicator = false,
  indicator,
  indicatorClassName,
  classNameMode = "merge",
  classResolver,
  indicatorClassResolver,
  ...props
}: SelectTriggerProps) {
  if (mode === "primitive") {
    const rest = { ...props }
    return (
      <SelectPrimitive.Trigger className={className} {...rest}>
        {children}
        {!hideIndicator && (
          <SelectPrimitive.Icon asChild>
            {indicator ?? (
              <DefaultSelectIndicator className={indicatorClassName} />
            )}
          </SelectPrimitive.Icon>
        )}
      </SelectPrimitive.Trigger>
    )
  }

  const defaultClassName = cn(
    selectTriggerClassName,
    selectTriggerSurfaceClassName
  )

  const resolvedClassName = classResolver
    ? classResolver({
        defaultClassName,
        className,
        size,
      })
    : resolveStyledClassName({
        className,
        defaultClassName,
        classNameMode,
        classResolver: undefined,
      })

  const resolvedIndicatorClassName = resolveStyledClassName({
    className: indicatorClassName,
    defaultClassName: selectTriggerIconClassName,
    classNameMode,
    classResolver: indicatorClassResolver,
  })

  return (
    <SelectPrimitive.Trigger
      data-slot="select-trigger"
      data-size={size}
      className={resolvedClassName}
      {...props}
    >
      {children}
      {!hideIndicator && (
        <SelectPrimitive.Icon asChild>
          {indicator ?? (
            <DefaultSelectIndicator className={resolvedIndicatorClassName} />
          )}
        </SelectPrimitive.Icon>
      )}
    </SelectPrimitive.Trigger>
  )
}

export function SelectContent({
  className,
  mode = DEFAULT_MODE,
  children,
  position = "item-aligned",
  align = "center",
  showScrollButtons = true,
  classNameMode = "merge",
  classResolver,
  ...props
}: SelectContentProps) {
  if (mode === "primitive") {
    const rest = { ...props }
    return (
      <SelectPrimitive.Portal>
        <SelectPrimitive.Content
          className={className}
          position={position}
          align={align}
          {...rest}
        >
          {showScrollButtons ? <SelectScrollUpButton mode="primitive" /> : null}
          <SelectPrimitive.Viewport>{children}</SelectPrimitive.Viewport>
          {showScrollButtons ? (
            <SelectScrollDownButton mode="primitive" />
          ) : null}
        </SelectPrimitive.Content>
      </SelectPrimitive.Portal>
    )
  }

  const defaultClassName = cn(
    selectContentBaseClassName,
    position === "popper" && selectContentPopperOffsetClassName
  )

  const resolvedClassName = classResolver
    ? classResolver({
        defaultClassName,
        className,
        position,
      })
    : resolveStyledClassName({
        className,
        defaultClassName,
        classNameMode,
        classResolver: undefined,
      })

  return (
    <SelectPrimitive.Portal>
      <SelectPrimitive.Content
        data-slot="select-content"
        data-align-trigger={position === "item-aligned"}
        className={resolvedClassName}
        position={position}
        align={align}
        {...props}
      >
        {showScrollButtons ? <SelectScrollUpButton /> : null}
        <SelectPrimitive.Viewport
          data-position={position}
          className={cn(selectViewportClassName)}
        >
          {children}
        </SelectPrimitive.Viewport>
        {showScrollButtons ? <SelectScrollDownButton /> : null}
      </SelectPrimitive.Content>
    </SelectPrimitive.Portal>
  )
}

export function SelectLabel({
  className,
  mode = DEFAULT_MODE,
  classNameMode = "merge",
  classResolver,
  ...props
}: SelectLabelProps) {
  if (mode === "primitive") {
    const rest = { ...props }
    return <SelectPrimitive.Label className={className} {...rest} />
  }

  return (
    <SelectPrimitive.Label
      data-slot="select-label"
      className={resolveStyledClassName({
        className,
        defaultClassName: selectLabelClassName,
        classNameMode,
        classResolver,
      })}
      {...props}
    />
  )
}

export function SelectItem({
  className,
  mode = DEFAULT_MODE,
  children,
  hideIndicator = false,
  indicator,
  indicatorContainerClassName,
  classNameMode = "merge",
  classResolver,
  indicatorContainerClassResolver,
  ...props
}: SelectItemProps) {
  if (mode === "primitive") {
    const rest = { ...props }
    return (
      <SelectPrimitive.Item className={className} {...rest}>
        {!hideIndicator && (
          <span className={indicatorContainerClassName}>
            <SelectPrimitive.ItemIndicator>
              {indicator ?? (
                <CheckIcon className="pointer-events-none size-3.5" />
              )}
            </SelectPrimitive.ItemIndicator>
          </span>
        )}
        <SelectPrimitive.ItemText>{children}</SelectPrimitive.ItemText>
      </SelectPrimitive.Item>
    )
  }

  const defaultClassName = cn(selectItemClassName, selectItemStateClassName)

  return (
    <SelectPrimitive.Item
      data-slot="select-item"
      className={resolveStyledClassName({
        className,
        defaultClassName,
        classNameMode,
        classResolver,
      })}
      {...props}
    >
      {!hideIndicator && (
        <span
          className={resolveStyledClassName({
            className: indicatorContainerClassName,
            defaultClassName: selectItemIndicatorContainerClassName,
            classNameMode,
            classResolver: indicatorContainerClassResolver,
          })}
        >
          <SelectPrimitive.ItemIndicator>
            {indicator ?? (
              <CheckIcon className="pointer-events-none size-3.5" />
            )}
          </SelectPrimitive.ItemIndicator>
        </span>
      )}
      <SelectPrimitive.ItemText>{children}</SelectPrimitive.ItemText>
    </SelectPrimitive.Item>
  )
}

export function SelectSeparator({
  className,
  mode = DEFAULT_MODE,
  classNameMode = "merge",
  classResolver,
  ...props
}: SelectSeparatorProps) {
  if (mode === "primitive") {
    const rest = { ...props }
    return <SelectPrimitive.Separator className={className} {...rest} />
  }

  return (
    <SelectPrimitive.Separator
      data-slot="select-separator"
      className={resolveStyledClassName({
        className,
        defaultClassName: selectSeparatorClassName,
        classNameMode,
        classResolver,
      })}
      {...props}
    />
  )
}

export function SelectScrollUpButton({
  className,
  children,
  mode = DEFAULT_MODE,
  classNameMode = "merge",
  classResolver,
  ...props
}: SelectScrollUpButtonProps) {
  if (mode === "primitive") {
    const rest = { ...props }
    return (
      <SelectPrimitive.ScrollUpButton className={className} {...rest}>
        {children ?? <ChevronUpIcon className="size-4" />}
      </SelectPrimitive.ScrollUpButton>
    )
  }

  return (
    <SelectPrimitive.ScrollUpButton
      data-slot="select-scroll-up-button"
      className={resolveStyledClassName({
        className,
        defaultClassName: selectScrollButtonClassName,
        classNameMode,
        classResolver,
      })}
      {...props}
    >
      {children ?? <ChevronUpIcon className="size-4" />}
    </SelectPrimitive.ScrollUpButton>
  )
}

export function SelectScrollDownButton({
  className,
  children,
  mode = DEFAULT_MODE,
  classNameMode = "merge",
  classResolver,
  ...props
}: SelectScrollDownButtonProps) {
  if (mode === "primitive") {
    const rest = { ...props }
    return (
      <SelectPrimitive.ScrollDownButton className={className} {...rest}>
        {children ?? <ChevronDownIcon className="size-4" />}
      </SelectPrimitive.ScrollDownButton>
    )
  }

  return (
    <SelectPrimitive.ScrollDownButton
      data-slot="select-scroll-down-button"
      className={resolveStyledClassName({
        className,
        defaultClassName: selectScrollButtonClassName,
        classNameMode,
        classResolver,
      })}
      {...props}
    >
      {children ?? <ChevronDownIcon className="size-4" />}
    </SelectPrimitive.ScrollDownButton>
  )
}
