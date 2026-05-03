export type CarouselClassNameMode = "merge" | "replace"

export type CarouselClassResolver = (params: {
  defaultClassName: string
  className?: string
}) => string
