export type DirectionClassNameMode = "merge" | "replace"

export type DirectionClassResolver = (params: {
  defaultClassName: string
  className?: string
}) => string
