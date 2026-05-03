export type CalendarClassNameMode = "merge" | "replace"

export type CalendarClassResolver = (params: {
  defaultClassName: string
  className?: string
}) => string
