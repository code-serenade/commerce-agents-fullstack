export type DialogClassNameMode = "merge" | "replace"

export type DialogClassResolver = (params: {
  defaultClassName: string
  className?: string
}) => string
