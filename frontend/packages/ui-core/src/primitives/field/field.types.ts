export type FieldClassNameMode = "merge" | "replace"

export type FieldClassResolver = (params: {
  defaultClassName: string
  className?: string
}) => string
