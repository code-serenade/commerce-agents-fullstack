export type AlertDialogClassNameMode = "merge" | "replace"

export type AlertDialogClassResolver = (params: {
  defaultClassName: string
  className?: string
}) => string
