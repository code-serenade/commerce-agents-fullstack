export type CommandClassNameMode = "merge" | "replace"

export type CommandClassResolver = (params: {
  defaultClassName: string
  className?: string
}) => string
