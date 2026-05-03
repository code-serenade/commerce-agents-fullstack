import {
  Pagination as CorePagination,
  PaginationContent as CorePaginationContent,
  PaginationEllipsis as CorePaginationEllipsis,
  PaginationItem as CorePaginationItem,
  PaginationLink as CorePaginationLink,
  PaginationNextMore as CorePaginationNextMore,
  PaginationNext as CorePaginationNext,
  PaginationPreviousMore as CorePaginationPreviousMore,
  PaginationPrevious as CorePaginationPrevious,
} from "@workspace/ui-core/components/pagination"

export interface PaginationProps {
  page: number
  totalPages: number
  onPageChange: (page: number) => void
  jumpPageCount?: number
  maxVisiblePages?: number
}

type PaginationPart = number | "ellipsis-left" | "ellipsis-right"

function createRange(start: number, end: number) {
  return Array.from({ length: end - start + 1 }, (_, index) => start + index)
}

function normalizeMaxVisiblePages(maxVisiblePages: number) {
  const normalized = Math.max(5, Math.floor(maxVisiblePages))
  return normalized % 2 === 0 ? normalized + 1 : normalized
}

function buildPaginationParts(
  page: number,
  totalPages: number,
  maxVisiblePages: number
): PaginationPart[] {
  const visibleItems = normalizeMaxVisiblePages(maxVisiblePages)

  if (totalPages <= visibleItems) {
    return Array.from({ length: totalPages }, (_, index) => index + 1)
  }

  const siblingCount = Math.max(1, Math.floor((visibleItems - 5) / 2))
  const leftSibling = Math.max(page - siblingCount, 1)
  const rightSibling = Math.min(page + siblingCount, totalPages)
  const showLeftEllipsis = leftSibling > 2
  const showRightEllipsis = rightSibling < totalPages - 1

  if (!showLeftEllipsis && showRightEllipsis) {
    const leftItemCount = 3 + siblingCount * 2
    return [...createRange(1, leftItemCount), "ellipsis-right", totalPages]
  }

  if (showLeftEllipsis && !showRightEllipsis) {
    const rightItemCount = 3 + siblingCount * 2
    return [
      1,
      "ellipsis-left",
      ...createRange(totalPages - rightItemCount + 1, totalPages),
    ]
  }

  return [
    1,
    "ellipsis-left",
    ...createRange(leftSibling, rightSibling),
    "ellipsis-right",
    totalPages,
  ]
}

export function Pagination({
  page,
  totalPages,
  onPageChange,
  jumpPageCount = 5,
  maxVisiblePages = 7,
}: PaginationProps) {
  const safeTotalPages = Math.max(1, totalPages)
  const safePage = Math.min(Math.max(1, page), safeTotalPages)
  const safeJumpPageCount = Math.max(1, Math.floor(jumpPageCount))
  const parts = buildPaginationParts(safePage, safeTotalPages, maxVisiblePages)
  const previousMorePage = Math.max(1, safePage - safeJumpPageCount)
  const nextMorePage = Math.min(safeTotalPages, safePage + safeJumpPageCount)
  const canGoPrevious = safePage > 1
  const canGoPreviousMore = safePage - previousMorePage > 1
  const canGoNext = safePage < safeTotalPages
  const canGoNextMore = nextMorePage - safePage > 1

  return (
    <CorePagination>
      <CorePaginationContent>
        <CorePaginationItem>
          <CorePaginationPreviousMore
            href="#"
            disabled={!canGoPreviousMore}
            onClick={(event) => {
              event.preventDefault()
              onPageChange(previousMorePage)
            }}
          />
        </CorePaginationItem>

        <CorePaginationItem>
          <CorePaginationPrevious
            href="#"
            disabled={!canGoPrevious}
            onClick={(event) => {
              event.preventDefault()
              onPageChange(Math.max(1, safePage - 1))
            }}
          />
        </CorePaginationItem>

        {parts.map((part) =>
          typeof part === "number" ? (
            <CorePaginationItem key={part}>
              <CorePaginationLink
                href="#"
                isActive={part === safePage}
                onClick={(event) => {
                  event.preventDefault()
                  onPageChange(part)
                }}
              >
                {part}
              </CorePaginationLink>
            </CorePaginationItem>
          ) : (
            <CorePaginationItem key={part}>
              <CorePaginationEllipsis />
            </CorePaginationItem>
          )
        )}

        <CorePaginationItem>
          <CorePaginationNext
            href="#"
            disabled={!canGoNext}
            onClick={(event) => {
              event.preventDefault()
              onPageChange(Math.min(safeTotalPages, safePage + 1))
            }}
          />
        </CorePaginationItem>

        <CorePaginationItem>
          <CorePaginationNextMore
            href="#"
            disabled={!canGoNextMore}
            onClick={(event) => {
              event.preventDefault()
              onPageChange(nextMorePage)
            }}
          />
        </CorePaginationItem>
      </CorePaginationContent>
    </CorePagination>
  )
}
