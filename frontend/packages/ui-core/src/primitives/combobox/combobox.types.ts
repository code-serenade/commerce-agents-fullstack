export type ComboboxClassNameMode = "merge" | "replace"

export type ComboboxClassResolver = (params: {
  defaultClassName: string
  className?: string
}) => string
