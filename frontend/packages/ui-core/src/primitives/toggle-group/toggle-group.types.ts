export type ToggleGroupClassNameMode = "merge" | "replace"

export type ToggleGroupClassResolver = (params: {
  defaultClassName: string
  className?: string
}) => string
