export type PaginationClassNameMode = "merge" | "replace"

export type PaginationClassResolver = (params: {
  defaultClassName: string
  className?: string
}) => string
