import { DEFAULT_MODE } from "../../lib/component-mode"
import { cn } from "../../lib/utils"
import { cardClassNames } from "./card.styles"
import type {
  CardActionProps,
  CardClassResolver,
  CardContentProps,
  CardDescriptionProps,
  CardFooterProps,
  CardHeaderProps,
  CardProps,
  CardSize,
  CardTitleProps,
} from "./card.types"

function resolveStyledCardClassName({
  className,
  defaultClassName,
  classNameMode,
  classResolver,
}: {
  className?: string
  defaultClassName: string
  classNameMode: "merge" | "replace"
  classResolver?: CardClassResolver
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

function Card({
  mode = DEFAULT_MODE,
  className,
  size = "default",
  classNameMode = "merge",
  classResolver,
  ...props
}: CardProps) {
  if (mode === "primitive") {
    return <div className={className} {...props} />
  }

  const resolvedSize = (size ?? "default") as CardSize

  return (
    <div
      data-slot="card"
      data-size={resolvedSize}
      className={resolveStyledCardClassName({
        className,
        defaultClassName: cardClassNames.slot1,
        classNameMode,
        classResolver,
      })}
      {...props}
    />
  )
}

function CardHeader({
  mode = DEFAULT_MODE,
  className,
  classNameMode = "merge",
  classResolver,
  ...props
}: CardHeaderProps) {
  if (mode === "primitive") {
    return <div className={className} {...props} />
  }

  return (
    <div
      data-slot="card-header"
      className={resolveStyledCardClassName({
        className,
        defaultClassName: cardClassNames.slot2,
        classNameMode,
        classResolver,
      })}
      {...props}
    />
  )
}

function CardTitle({
  mode = DEFAULT_MODE,
  className,
  classNameMode = "merge",
  classResolver,
  ...props
}: CardTitleProps) {
  if (mode === "primitive") {
    return <div className={className} {...props} />
  }

  return (
    <div
      data-slot="card-title"
      className={resolveStyledCardClassName({
        className,
        defaultClassName: cardClassNames.slot3,
        classNameMode,
        classResolver,
      })}
      {...props}
    />
  )
}

function CardDescription({
  mode = DEFAULT_MODE,
  className,
  classNameMode = "merge",
  classResolver,
  ...props
}: CardDescriptionProps) {
  if (mode === "primitive") {
    return <div className={className} {...props} />
  }

  return (
    <div
      data-slot="card-description"
      className={resolveStyledCardClassName({
        className,
        defaultClassName: cardClassNames.slot4,
        classNameMode,
        classResolver,
      })}
      {...props}
    />
  )
}

function CardAction({
  mode = DEFAULT_MODE,
  className,
  classNameMode = "merge",
  classResolver,
  ...props
}: CardActionProps) {
  if (mode === "primitive") {
    return <div className={className} {...props} />
  }

  return (
    <div
      data-slot="card-action"
      className={resolveStyledCardClassName({
        className,
        defaultClassName: cardClassNames.slot5,
        classNameMode,
        classResolver,
      })}
      {...props}
    />
  )
}

function CardContent({
  mode = DEFAULT_MODE,
  className,
  classNameMode = "merge",
  classResolver,
  ...props
}: CardContentProps) {
  if (mode === "primitive") {
    return <div className={className} {...props} />
  }

  return (
    <div
      data-slot="card-content"
      className={resolveStyledCardClassName({
        className,
        defaultClassName: cardClassNames.slot6,
        classNameMode,
        classResolver,
      })}
      {...props}
    />
  )
}

function CardFooter({
  mode = DEFAULT_MODE,
  className,
  classNameMode = "merge",
  classResolver,
  ...props
}: CardFooterProps) {
  if (mode === "primitive") {
    return <div className={className} {...props} />
  }

  return (
    <div
      data-slot="card-footer"
      className={resolveStyledCardClassName({
        className,
        defaultClassName: cardClassNames.slot7,
        classNameMode,
        classResolver,
      })}
      {...props}
    />
  )
}

export {
  Card,
  CardHeader,
  CardFooter,
  CardTitle,
  CardAction,
  CardDescription,
  CardContent,
}
