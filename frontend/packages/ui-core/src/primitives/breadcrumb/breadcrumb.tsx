import { Slot } from "radix-ui"

import { DEFAULT_MODE } from "../../lib/component-mode"
import { cn } from "../../lib/utils"
import { ChevronRightIcon, MoreHorizontalIcon } from "../../lib/icon-slots"
import { breadcrumbClassNames } from "./breadcrumb.styles"
import type {
  BreadcrumbClassResolver,
  BreadcrumbEllipsisProps,
  BreadcrumbItemProps,
  BreadcrumbLinkProps,
  BreadcrumbListProps,
  BreadcrumbPageProps,
  BreadcrumbProps,
  BreadcrumbSeparatorProps,
} from "./breadcrumb.types"

function resolveStyledBreadcrumbClassName({
  className,
  defaultClassName,
  classNameMode,
  classResolver,
}: {
  className?: string
  defaultClassName: string
  classNameMode: "merge" | "replace"
  classResolver?: BreadcrumbClassResolver
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

function Breadcrumb({
  mode = DEFAULT_MODE,
  className,
  classNameMode = "merge",
  classResolver,
  ...props
}: BreadcrumbProps) {
  if (mode === "primitive") {
    return <nav aria-label="breadcrumb" className={className} {...props} />
  }

  return (
    <nav
      aria-label="breadcrumb"
      data-slot="breadcrumb"
      className={resolveStyledBreadcrumbClassName({
        className,
        defaultClassName: breadcrumbClassNames.slot0,
        classNameMode,
        classResolver,
      })}
      {...props}
    />
  )
}

function BreadcrumbList({
  mode = DEFAULT_MODE,
  className,
  classNameMode = "merge",
  classResolver,
  ...props
}: BreadcrumbListProps) {
  if (mode === "primitive") {
    return <ol className={className} {...props} />
  }

  return (
    <ol
      data-slot="breadcrumb-list"
      className={resolveStyledBreadcrumbClassName({
        className,
        defaultClassName: breadcrumbClassNames.slot2,
        classNameMode,
        classResolver,
      })}
      {...props}
    />
  )
}

function BreadcrumbItem({
  mode = DEFAULT_MODE,
  className,
  classNameMode = "merge",
  classResolver,
  ...props
}: BreadcrumbItemProps) {
  if (mode === "primitive") {
    return <li className={className} {...props} />
  }

  return (
    <li
      data-slot="breadcrumb-item"
      className={resolveStyledBreadcrumbClassName({
        className,
        defaultClassName: breadcrumbClassNames.slot3,
        classNameMode,
        classResolver,
      })}
      {...props}
    />
  )
}

function BreadcrumbLink({
  asChild,
  mode = DEFAULT_MODE,
  className,
  classNameMode = "merge",
  classResolver,
  ...props
}: BreadcrumbLinkProps) {
  const Comp = asChild ? Slot.Root : "a"

  if (mode === "primitive") {
    return <Comp className={className} {...props} />
  }

  return (
    <Comp
      data-slot="breadcrumb-link"
      className={resolveStyledBreadcrumbClassName({
        className,
        defaultClassName: breadcrumbClassNames.slot4,
        classNameMode,
        classResolver,
      })}
      {...props}
    />
  )
}

function BreadcrumbPage({
  mode = DEFAULT_MODE,
  className,
  classNameMode = "merge",
  classResolver,
  ...props
}: BreadcrumbPageProps) {
  if (mode === "primitive") {
    return (
      <span
        role="link"
        aria-disabled="true"
        aria-current="page"
        className={className}
        {...props}
      />
    )
  }

  return (
    <span
      data-slot="breadcrumb-page"
      role="link"
      aria-disabled="true"
      aria-current="page"
      className={resolveStyledBreadcrumbClassName({
        className,
        defaultClassName: breadcrumbClassNames.slot5,
        classNameMode,
        classResolver,
      })}
      {...props}
    />
  )
}

function BreadcrumbSeparator({
  children,
  mode = DEFAULT_MODE,
  className,
  classNameMode = "merge",
  classResolver,
  ...props
}: BreadcrumbSeparatorProps) {
  if (mode === "primitive") {
    return (
      <li
        role="presentation"
        aria-hidden="true"
        className={className}
        {...props}
      >
        {children ?? <ChevronRightIcon />}
      </li>
    )
  }

  return (
    <li
      data-slot="breadcrumb-separator"
      role="presentation"
      aria-hidden="true"
      className={resolveStyledBreadcrumbClassName({
        className,
        defaultClassName: breadcrumbClassNames.slot6,
        classNameMode,
        classResolver,
      })}
      {...props}
    >
      {children ?? <ChevronRightIcon />}
    </li>
  )
}

function BreadcrumbEllipsis({
  mode = DEFAULT_MODE,
  className,
  classNameMode = "merge",
  classResolver,
  ...props
}: BreadcrumbEllipsisProps) {
  if (mode === "primitive") {
    return (
      <span
        role="presentation"
        aria-hidden="true"
        className={className}
        {...props}
      >
        <MoreHorizontalIcon />
      </span>
    )
  }

  return (
    <span
      data-slot="breadcrumb-ellipsis"
      role="presentation"
      aria-hidden="true"
      className={resolveStyledBreadcrumbClassName({
        className,
        defaultClassName: breadcrumbClassNames.slot7,
        classNameMode,
        classResolver,
      })}
      {...props}
    >
      <MoreHorizontalIcon />
      <span className={breadcrumbClassNames.slot1}>More</span>
    </span>
  )
}

export {
  Breadcrumb,
  BreadcrumbList,
  BreadcrumbItem,
  BreadcrumbLink,
  BreadcrumbPage,
  BreadcrumbSeparator,
  BreadcrumbEllipsis,
}
