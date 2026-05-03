import { DEFAULT_MODE } from "../../lib/component-mode"
import { cn } from "../../lib/utils"
import {
  alertActionClassName,
  alertDescriptionClassName,
  alertTitleClassName,
  alertVariants,
} from "./alert.styles"
import type {
  AlertActionProps,
  AlertClassResolver,
  AlertDescriptionProps,
  AlertProps,
  AlertTitleProps,
  AlertVariant,
} from "./alert.types"

function resolveStyledClassName({
  className,
  defaultClassName,
  classNameMode,
  classResolver,
}: {
  className?: string
  defaultClassName: string
  classNameMode: "merge" | "replace"
  classResolver?: (params: {
    defaultClassName: string
    className?: string
  }) => string
}) {
  if (classResolver) {
    return classResolver({ defaultClassName, className })
  }

  if (classNameMode === "replace") {
    return className ?? defaultClassName
  }

  return cn(defaultClassName, className)
}

function resolveStyledAlertClassName({
  className,
  variant,
  classNameMode,
  classResolver,
}: {
  className?: string
  variant: AlertVariant
  classNameMode: "merge" | "replace"
  classResolver?: AlertClassResolver
}) {
  const defaultClassName = alertVariants({ variant })

  if (classResolver) {
    return classResolver({ variant, defaultClassName, className })
  }

  if (classNameMode === "replace") {
    return className ?? defaultClassName
  }

  return cn(defaultClassName, className)
}

function Alert({
  className,
  mode = DEFAULT_MODE,
  variant = "default",
  classNameMode = "merge",
  classResolver,
  ...props
}: AlertProps) {
  if (mode === "primitive") {
    const rest = { ...props }
    return <div role="alert" className={className} {...rest} />
  }

  const resolvedVariant = (variant ?? "default") as AlertVariant
  return (
    <div
      data-slot="alert"
      data-variant={resolvedVariant}
      role="alert"
      className={resolveStyledAlertClassName({
        className,
        variant: resolvedVariant,
        classNameMode,
        classResolver,
      })}
      {...props}
    />
  )
}

function AlertTitle({
  className,
  mode = DEFAULT_MODE,
  classNameMode = "merge",
  classResolver,
  ...props
}: AlertTitleProps) {
  if (mode === "primitive") {
    const rest = { ...props }
    return <div className={className} {...rest} />
  }

  return (
    <div
      data-slot="alert-title"
      className={resolveStyledClassName({
        className,
        defaultClassName: alertTitleClassName,
        classNameMode,
        classResolver,
      })}
      {...props}
    />
  )
}

function AlertDescription({
  className,
  mode = DEFAULT_MODE,
  classNameMode = "merge",
  classResolver,
  ...props
}: AlertDescriptionProps) {
  if (mode === "primitive") {
    const rest = { ...props }
    return <div className={className} {...rest} />
  }

  return (
    <div
      data-slot="alert-description"
      className={resolveStyledClassName({
        className,
        defaultClassName: alertDescriptionClassName,
        classNameMode,
        classResolver,
      })}
      {...props}
    />
  )
}

function AlertAction({
  className,
  mode = DEFAULT_MODE,
  classNameMode = "merge",
  classResolver,
  ...props
}: AlertActionProps) {
  if (mode === "primitive") {
    const rest = { ...props }
    return <div className={className} {...rest} />
  }

  return (
    <div
      data-slot="alert-action"
      className={resolveStyledClassName({
        className,
        defaultClassName: alertActionClassName,
        classNameMode,
        classResolver,
      })}
      {...props}
    />
  )
}

export { Alert, AlertTitle, AlertDescription, AlertAction }
