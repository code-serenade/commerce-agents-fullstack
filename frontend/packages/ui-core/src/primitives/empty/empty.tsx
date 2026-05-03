import { DEFAULT_MODE } from "../../lib/component-mode"
import { cn } from "../../lib/utils"
import { emptyClassNames, emptyMediaVariants } from "./empty.styles"
import type {
  EmptyClassResolver,
  EmptyContentProps,
  EmptyDescriptionProps,
  EmptyHeaderProps,
  EmptyMediaClassResolver,
  EmptyMediaProps,
  EmptyMediaVariant,
  EmptyProps,
  EmptyTitleProps,
} from "./empty.types"

function resolveStyledEmptyClassName({
  className,
  defaultClassName,
  classNameMode,
  classResolver,
}: {
  className?: string
  defaultClassName: string
  classNameMode: "merge" | "replace"
  classResolver?: EmptyClassResolver
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

function resolveStyledEmptyMediaClassName({
  className,
  variant,
  classNameMode,
  classResolver,
}: {
  className?: string
  variant: EmptyMediaVariant
  classNameMode: "merge" | "replace"
  classResolver?: EmptyMediaClassResolver
}) {
  const defaultClassName = emptyMediaVariants({ variant })

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

function Empty({
  mode = DEFAULT_MODE,
  className,
  classNameMode = "merge",
  classResolver,
  ...props
}: EmptyProps) {
  if (mode === "primitive") {
    return <div className={className} {...props} />
  }

  return (
    <div
      data-slot="empty"
      className={resolveStyledEmptyClassName({
        className,
        defaultClassName: emptyClassNames.slot0,
        classNameMode,
        classResolver,
      })}
      {...props}
    />
  )
}

function EmptyHeader({
  mode = DEFAULT_MODE,
  className,
  classNameMode = "merge",
  classResolver,
  ...props
}: EmptyHeaderProps) {
  if (mode === "primitive") {
    return <div className={className} {...props} />
  }

  return (
    <div
      data-slot="empty-header"
      className={resolveStyledEmptyClassName({
        className,
        defaultClassName: emptyClassNames.slot4,
        classNameMode,
        classResolver,
      })}
      {...props}
    />
  )
}

function EmptyMedia({
  mode = DEFAULT_MODE,
  className,
  variant = "default",
  classNameMode = "merge",
  classResolver,
  ...props
}: EmptyMediaProps) {
  if (mode === "primitive") {
    return <div className={className} {...props} />
  }

  const resolvedVariant = (variant ?? "default") as EmptyMediaVariant

  return (
    <div
      data-slot="empty-icon"
      data-variant={resolvedVariant}
      className={resolveStyledEmptyMediaClassName({
        className,
        variant: resolvedVariant,
        classNameMode,
        classResolver,
      })}
      {...props}
    />
  )
}

function EmptyTitle({
  mode = DEFAULT_MODE,
  className,
  classNameMode = "merge",
  classResolver,
  ...props
}: EmptyTitleProps) {
  if (mode === "primitive") {
    return <div className={className} {...props} />
  }

  return (
    <div
      data-slot="empty-title"
      className={resolveStyledEmptyClassName({
        className,
        defaultClassName: emptyClassNames.slot5,
        classNameMode,
        classResolver,
      })}
      {...props}
    />
  )
}

function EmptyDescription({
  mode = DEFAULT_MODE,
  className,
  classNameMode = "merge",
  classResolver,
  ...props
}: EmptyDescriptionProps) {
  if (mode === "primitive") {
    return <p className={className} {...props} />
  }

  return (
    <p
      data-slot="empty-description"
      className={resolveStyledEmptyClassName({
        className,
        defaultClassName: emptyClassNames.slot6,
        classNameMode,
        classResolver,
      })}
      {...props}
    />
  )
}

function EmptyContent({
  mode = DEFAULT_MODE,
  className,
  classNameMode = "merge",
  classResolver,
  ...props
}: EmptyContentProps) {
  if (mode === "primitive") {
    return <div className={className} {...props} />
  }

  return (
    <div
      data-slot="empty-content"
      className={resolveStyledEmptyClassName({
        className,
        defaultClassName: emptyClassNames.slot7,
        classNameMode,
        classResolver,
      })}
      {...props}
    />
  )
}

export {
  Empty,
  EmptyHeader,
  EmptyTitle,
  EmptyDescription,
  EmptyContent,
  EmptyMedia,
}
