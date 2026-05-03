import * as React from "react"

import { cn } from "../../lib/utils"
import { Button } from "../button"
import {
  ChevronLeftIcon,
  ChevronRightIcon,
  ChevronsLeftIcon,
  ChevronsRightIcon,
  MoreHorizontalIcon,
} from "../../lib/icon-slots"
import { paginationClassNames } from "./pagination.styles"

function Pagination({ className, ...props }: React.ComponentProps<"nav">) {
  return (
    <nav
      role="navigation"
      aria-label="pagination"
      data-slot="pagination"
      className={cn(paginationClassNames.slot1, className)}
      {...props}
    />
  )
}

function PaginationContent({
  className,
  ...props
}: React.ComponentProps<"ul">) {
  return (
    <ul
      data-slot="pagination-content"
      className={cn(paginationClassNames.slot2, className)}
      {...props}
    />
  )
}

function PaginationItem({ ...props }: React.ComponentProps<"li">) {
  return <li data-slot="pagination-item" {...props} />
}

type PaginationLinkProps = {
  isActive?: boolean
  disabled?: boolean
} & Pick<React.ComponentProps<typeof Button>, "size"> &
  React.ComponentProps<"a">

function PaginationLink({
  className,
  isActive,
  disabled = false,
  size = "icon",
  onClick,
  tabIndex,
  ...props
}: PaginationLinkProps) {
  return (
    <Button
      asChild
      variant={isActive ? "outline" : "ghost"}
      size={size}
      className={cn(paginationClassNames.slot3, className)}
    >
      <a
        aria-current={isActive ? "page" : undefined}
        aria-disabled={disabled || undefined}
        data-slot="pagination-link"
        data-active={isActive}
        onClick={(event) => {
          if (disabled) {
            event.preventDefault()
            return
          }

          onClick?.(event)
        }}
        tabIndex={disabled ? -1 : tabIndex}
        {...props}
      />
    </Button>
  )
}

function PaginationPrevious({
  className,
  text: _text = "Previous",
  ...props
}: React.ComponentProps<typeof PaginationLink> & { text?: string }) {
  return (
    <PaginationLink
      aria-label="Go to previous page"
      size="icon"
      className={cn(className)}
      {...props}
    >
      <ChevronLeftIcon data-icon="inline-start" />
    </PaginationLink>
  )
}

function PaginationPreviousMore({
  className,
  text: _text = "Previous pages",
  ...props
}: React.ComponentProps<typeof PaginationLink> & { text?: string }) {
  return (
    <PaginationLink
      aria-label="Go back several pages"
      size="icon"
      className={cn(className)}
      {...props}
    >
      <ChevronsLeftIcon data-icon="inline-start" />
    </PaginationLink>
  )
}

function PaginationNext({
  className,
  text: _text = "Next",
  ...props
}: React.ComponentProps<typeof PaginationLink> & { text?: string }) {
  return (
    <PaginationLink
      aria-label="Go to next page"
      size="icon"
      className={cn(className)}
      {...props}
    >
      <ChevronRightIcon data-icon="inline-end" />
    </PaginationLink>
  )
}

function PaginationNextMore({
  className,
  text: _text = "Next pages",
  ...props
}: React.ComponentProps<typeof PaginationLink> & { text?: string }) {
  return (
    <PaginationLink
      aria-label="Go forward several pages"
      size="icon"
      className={cn(className)}
      {...props}
    >
      <ChevronsRightIcon data-icon="inline-end" />
    </PaginationLink>
  )
}

function PaginationEllipsis({
  className,
  ...props
}: React.ComponentProps<"span">) {
  return (
    <span
      aria-hidden
      data-slot="pagination-ellipsis"
      className={cn(paginationClassNames.slot4, className)}
      {...props}
    >
      <MoreHorizontalIcon />
      <span className={paginationClassNames.slot5}>More pages</span>
    </span>
  )
}

export {
  Pagination,
  PaginationContent,
  PaginationEllipsis,
  PaginationItem,
  PaginationLink,
  PaginationNextMore,
  PaginationNext,
  PaginationPreviousMore,
  PaginationPrevious,
}
