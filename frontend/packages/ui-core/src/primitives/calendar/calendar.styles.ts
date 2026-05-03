import type { ComponentProps } from "react"
import { getDefaultClassNames, type DayPicker } from "react-day-picker"

import { cn } from "../../lib/utils"
import { buttonVariants } from "../button"

type DayPickerProps = ComponentProps<typeof DayPicker>

export const calendarClassNames = {
  root: "w-fit",
  container:
    "group/calendar bg-background p-2 [--cell-radius:var(--radius-md)] [--cell-size:--spacing(7)] [--month-width:calc(var(--cell-size)*7+0.5rem)] in-data-[slot=card-content]:bg-transparent in-data-[slot=popover-content]:bg-transparent",
  months: "relative flex flex-col gap-4 md:flex-row",
  month: "flex w-(--month-width) min-w-(--month-width) flex-col gap-4",
  nav: "absolute inset-x-0 top-0 flex w-full items-center justify-between gap-1",
  navButton:
    "size-(--cell-size) bg-transparent p-0 select-none hover:bg-[var(--surface-hover)] aria-expanded:bg-transparent aria-disabled:opacity-50",
  monthCaption:
    "flex h-(--cell-size) w-full min-w-(--month-width) items-center justify-center px-[calc(var(--cell-size)*2+0.25rem)]",
  dropdowns:
    "flex h-(--cell-size) w-full items-center justify-center gap-1.5 text-sm font-medium",
  dropdownRoot: "relative rounded-(--cell-radius)",
  dropdown: "absolute inset-0 bg-popover opacity-0",
  captionLabelBase: "font-medium select-none",
  captionLabelDefault: "text-sm",
  captionLabelDropdown:
    "flex items-center gap-1 rounded-(--cell-radius) text-sm [&>svg]:size-3.5 [&>svg]:text-muted-foreground",
  table: "w-full border-collapse",
  weekdays: "flex",
  weekday:
    "flex-1 rounded-(--cell-radius) text-[0.8rem] font-normal text-muted-foreground select-none",
  week: "mt-2 flex w-full",
  weekNumberHeader: "w-(--cell-size) select-none",
  weekNumber: "text-[0.8rem] text-muted-foreground select-none",
  day: "group/day relative aspect-square h-full w-full rounded-(--cell-radius) p-0 text-center select-none [&:last-child[data-selected=true]_button]:rounded-r-(--cell-radius)",
  dayWeekNumber:
    "[&:nth-child(2)[data-selected=true]_button]:rounded-l-(--cell-radius)",
  dayDefault:
    "[&:first-child[data-selected=true]_button]:rounded-l-(--cell-radius)",
  rangeStart:
    "relative isolate z-0 rounded-l-(--cell-radius) bg-muted after:absolute after:inset-y-0 after:right-0 after:w-4 after:bg-muted",
  rangeMiddle: "rounded-none",
  rangeEnd:
    "relative isolate z-0 rounded-r-(--cell-radius) bg-muted after:absolute after:inset-y-0 after:left-0 after:w-4 after:bg-muted",
  today:
    "rounded-(--cell-radius) bg-[var(--surface-active)] text-foreground data-[selected=true]:rounded-none",
  outside:
    "text-[var(--muted-foreground)] aria-selected:text-[var(--muted-foreground)]",
  disabled: "text-muted-foreground opacity-50",
  hidden: "invisible",
  weekNumberContent:
    "flex size-(--cell-size) items-center justify-center text-center",
  dayButton:
    "relative isolate z-10 flex aspect-square size-auto w-full min-w-(--cell-size) flex-col gap-1 border-0 bg-transparent leading-none font-normal hover:bg-[var(--surface-hover)] group-data-[focused=true]/day:relative group-data-[focused=true]/day:z-10 group-data-[focused=true]/day:border-ring group-data-[focused=true]/day:ring-[3px] group-data-[focused=true]/day:ring-ring/50 data-[outside=true]:text-[var(--muted-foreground)] data-[range-end=true]:rounded-(--cell-radius) data-[range-end=true]:rounded-r-(--cell-radius) data-[range-end=true]:bg-primary data-[range-end=true]:text-primary-foreground data-[range-middle=true]:rounded-none data-[range-middle=true]:bg-muted data-[range-middle=true]:text-foreground data-[range-start=true]:rounded-(--cell-radius) data-[range-start=true]:rounded-l-(--cell-radius) data-[range-start=true]:bg-primary data-[range-start=true]:text-primary-foreground data-[selected-single=true]:bg-primary data-[selected-single=true]:text-primary-foreground dark:hover:text-foreground [&>span]:text-xs [&>span]:opacity-70",
} as const

export function getCalendarRootClassName(className?: string) {
  return cn(
    calendarClassNames.container,
    String.raw`rtl:**:[.rdp-button\_next>svg]:rotate-180`,
    String.raw`rtl:**:[.rdp-button\_previous>svg]:rotate-180`,
    className
  )
}

export function getCalendarClassNames({
  captionLayout,
  buttonVariant,
  showWeekNumber,
  classNames,
}: {
  captionLayout: DayPickerProps["captionLayout"]
  buttonVariant: ComponentProps<typeof import("../button").Button>["variant"]
  showWeekNumber?: boolean
  classNames?: DayPickerProps["classNames"]
}) {
  const defaults = getDefaultClassNames()

  return {
    root: cn(calendarClassNames.root, defaults.root),
    months: cn(calendarClassNames.months, defaults.months),
    month: cn(calendarClassNames.month, defaults.month),
    nav: cn(calendarClassNames.nav, defaults.nav),
    button_previous: cn(
      buttonVariants({ variant: buttonVariant }),
      calendarClassNames.navButton,
      defaults.button_previous
    ),
    button_next: cn(
      buttonVariants({ variant: buttonVariant }),
      calendarClassNames.navButton,
      defaults.button_next
    ),
    month_caption: cn(calendarClassNames.monthCaption, defaults.month_caption),
    dropdowns: cn(calendarClassNames.dropdowns, defaults.dropdowns),
    dropdown_root: cn(calendarClassNames.dropdownRoot, defaults.dropdown_root),
    dropdown: cn(calendarClassNames.dropdown, defaults.dropdown),
    caption_label: cn(
      calendarClassNames.captionLabelBase,
      captionLayout === "label"
        ? calendarClassNames.captionLabelDefault
        : calendarClassNames.captionLabelDropdown,
      defaults.caption_label
    ),
    table: calendarClassNames.table,
    weekdays: cn(calendarClassNames.weekdays, defaults.weekdays),
    weekday: cn(calendarClassNames.weekday, defaults.weekday),
    week: cn(calendarClassNames.week, defaults.week),
    week_number_header: cn(
      calendarClassNames.weekNumberHeader,
      defaults.week_number_header
    ),
    week_number: cn(calendarClassNames.weekNumber, defaults.week_number),
    day: cn(
      calendarClassNames.day,
      showWeekNumber
        ? calendarClassNames.dayWeekNumber
        : calendarClassNames.dayDefault,
      defaults.day
    ),
    range_start: cn(calendarClassNames.rangeStart, defaults.range_start),
    range_middle: cn(calendarClassNames.rangeMiddle, defaults.range_middle),
    range_end: cn(calendarClassNames.rangeEnd, defaults.range_end),
    today: cn(calendarClassNames.today, defaults.today),
    outside: cn(calendarClassNames.outside, defaults.outside),
    disabled: cn(calendarClassNames.disabled, defaults.disabled),
    hidden: cn(calendarClassNames.hidden, defaults.hidden),
    ...classNames,
  }
}
