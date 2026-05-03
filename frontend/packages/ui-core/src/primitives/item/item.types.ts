export type ItemClassNameMode = "merge" | "replace"

export type ItemClassResolver = (params: {
  defaultClassName: string
  className?: string
}) => string
