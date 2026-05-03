import * as React from "react"

export interface TimeValue {
  hour: string
  minute: string
  second?: string
}

export type TimeClassResolver = (params: {
  defaultClassName: string
  className?: string
}) => string

interface TimeBaseProps {
  value?: TimeValue
  defaultValue?: TimeValue
  onValueChange?: (value: TimeValue) => void
  disabled?: boolean
  showSeconds?: boolean
  visibleRows?: 3 | 5 | 7
  itemHeight?: number
  "aria-label"?: string
  hourLabel?: string
  minuteLabel?: string
  secondLabel?: string
  className?: string
  columnClassName?: string
  itemClassName?: string
  selectionClassName?: string
}

export type StyledTimeProps = TimeBaseProps & {
  mode?: "styled"
  classNameMode?: "merge" | "replace"
  classResolver?: TimeClassResolver
  columnClassNameMode?: "merge" | "replace"
  columnClassResolver?: TimeClassResolver
  itemClassNameMode?: "merge" | "replace"
  itemClassResolver?: TimeClassResolver
  selectionClassNameMode?: "merge" | "replace"
  selectionClassResolver?: TimeClassResolver
}

export type HeadlessTimeProps = TimeBaseProps & {
  mode: "primitive"
  classNameMode?: never
  classResolver?: never
  columnClassNameMode?: never
  columnClassResolver?: never
  itemClassNameMode?: never
  itemClassResolver?: never
  selectionClassNameMode?: never
  selectionClassResolver?: never
}

export type TimeProps = StyledTimeProps | HeadlessTimeProps

export type TimeSegment = "hour" | "minute" | "second"

export interface TimeColumnProps extends React.HTMLAttributes<HTMLDivElement> {
  segment: TimeSegment
}
