export type InputGroupClassNameMode = "merge" | "replace"

export type InputGroupClassResolver = (params: {
  defaultClassName: string
  className?: string
}) => string
