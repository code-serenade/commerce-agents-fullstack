import * as React from "react"
import { getCalendarCopy, normalizeLanguage } from "@workspace/locales"
import {
  DayPicker,
  type DayButton,
  type ClassNames,
  type CustomComponents,
  type Locale,
  useDayPicker,
} from "react-day-picker"

import {
  ChevronDownIcon,
  ChevronLeftIcon,
  ChevronRightIcon,
  ChevronsLeftIcon,
  ChevronsRightIcon,
} from "../../lib/icon-slots"
import { cn } from "../../lib/utils"
import { Button } from "../button"
import {
  Select,
  SelectContent,
  SelectItem,
  SelectTrigger,
  SelectValue,
} from "../select"
import {
  calendarClassNames,
  getCalendarClassNames,
  getCalendarRootClassName,
} from "./calendar.styles"

function formatCalendarMonth(
  date: Date,
  localeCode: string | undefined,
  options?: { includeYear?: boolean }
) {
  const isEnglish = localeCode?.startsWith("en") ?? false

  return date.toLocaleString(localeCode, {
    month: isEnglish ? "short" : "long",
    ...(options?.includeYear ? { year: "numeric" } : {}),
  })
}

function Calendar({
  className,
  classNames,
  showOutsideDays = true,
  captionLayout = "label",
  buttonVariant = "ghost",
  locale,
  formatters,
  components,
  renderChevron,
  ...props
}: React.ComponentProps<typeof DayPicker> & {
  buttonVariant?: React.ComponentProps<typeof Button>["variant"]
  renderChevron?: (params: {
    orientation: "left" | "right" | "down"
    className?: string
    props: Record<string, unknown>
  }) => React.ReactElement
}) {
  const localeLanguage = normalizeLanguage(locale?.code)
  const copy = getCalendarCopy(localeLanguage)

  return (
    <DayPicker
      showOutsideDays={showOutsideDays}
      className={getCalendarRootClassName(className)}
      captionLayout={captionLayout}
      locale={locale}
      formatters={{
        formatMonthDropdown: (date) => formatCalendarMonth(date, locale?.code),
        formatCaption: (date) =>
          formatCalendarMonth(date, locale?.code, {
            includeYear: true,
          }),
        ...formatters,
      }}
      classNames={getCalendarClassNames({
        captionLayout,
        buttonVariant,
        showWeekNumber: props.showWeekNumber,
        classNames,
      })}
      components={{
        Root: ({ className, rootRef, ...props }) => {
          return (
            <div
              data-slot="calendar"
              ref={rootRef}
              className={cn(className)}
              {...props}
            />
          )
        },
        Chevron: ({ className, orientation, ...props }) => {
          const normalizedOrientation: "left" | "right" | "down" =
            orientation === "left"
              ? "left"
              : orientation === "right"
                ? "right"
                : "down"

          if (renderChevron) {
            return renderChevron({
              orientation: normalizedOrientation,
              className: cn("size-4", className),
              props: props as Record<string, unknown>,
            })
          }

          return (
            <DefaultCalendarChevron
              orientation={normalizedOrientation}
              className={className}
              props={props as Record<string, unknown>}
            />
          )
        },
        DayButton: ({ ...props }) => (
          <CalendarDayButton locale={locale} {...props} />
        ),
        Dropdown: (props) => <CalendarDropdown {...props} />,
        Nav: (props) => (
          <CalendarNav
            {...props}
            buttonVariant={buttonVariant}
            copy={copy}
            renderChevron={renderChevron}
          />
        ),
        WeekNumber: ({ children, ...props }) => {
          return (
            <td {...props}>
              <div className={calendarClassNames.weekNumberContent}>
                {children}
              </div>
            </td>
          )
        },
        ...components,
      }}
      {...props}
    />
  )
}

type CalendarDropdownOption = {
  value: number
  label: string
  disabled: boolean
}

type CalendarDropdownProps = Omit<
  React.SelectHTMLAttributes<HTMLSelectElement>,
  "onChange"
> & {
  classNames: ClassNames
  components: CustomComponents
  options?: CalendarDropdownOption[]
  onChange?: React.ChangeEventHandler<HTMLSelectElement>
}

function CalendarDropdown({
  className,
  classNames,
  options,
  value,
  disabled,
  onChange,
  "aria-label": ariaLabel,
}: CalendarDropdownProps) {
  const selectedValue = value === undefined ? undefined : String(value)

  const selectedOption = options?.find(
    (option) => String(option.value) === selectedValue
  )

  return (
    <span data-disabled={disabled} className={classNames.dropdown_root}>
      <Select
        value={selectedValue}
        disabled={disabled}
        onValueChange={(nextValue) => {
          onChange?.({
            target: { value: nextValue },
            currentTarget: { value: nextValue },
          } as React.ChangeEvent<HTMLSelectElement>)
        }}
      >
        <SelectTrigger
          aria-label={ariaLabel}
          className={cn(
            calendarClassNames.captionLabelDropdown,
            "h-auto min-w-0 gap-1 border-0 bg-transparent px-2 py-1 shadow-none hover:bg-[var(--surface-hover)] focus-visible:border-ring",
            className
          )}
          indicator={
            <DefaultCalendarChevron
              orientation="down"
              className={classNames.chevron}
              props={{}}
            />
          }
        >
          <SelectValue>{selectedOption?.label}</SelectValue>
        </SelectTrigger>
        <SelectContent>
          {options?.map((option) => (
            <SelectItem
              key={option.value}
              value={String(option.value)}
              disabled={option.disabled}
            >
              {option.label}
            </SelectItem>
          ))}
        </SelectContent>
      </Select>
    </span>
  )
}

function CalendarDayButton({
  className,
  day,
  modifiers,
  locale,
  color: _color,
  ...props
}: React.ComponentProps<typeof DayButton> & { locale?: Partial<Locale> }) {
  const ref = React.useRef<HTMLButtonElement>(null)
  React.useEffect(() => {
    if (modifiers.focused) ref.current?.focus()
  }, [modifiers.focused])

  return (
    <Button
      ref={ref}
      variant="ghost"
      size="icon"
      data-day={day.date.toLocaleDateString(locale?.code)}
      data-selected-single={
        modifiers.selected &&
        !modifiers.range_start &&
        !modifiers.range_end &&
        !modifiers.range_middle
      }
      data-outside={modifiers.outside}
      data-range-start={modifiers.range_start}
      data-range-end={modifiers.range_end}
      data-range-middle={modifiers.range_middle}
      className={cn(calendarClassNames.dayButton, className)}
      {...props}
    />
  )
}

function CalendarNav({
  className,
  previousMonth,
  nextMonth,
  onPreviousClick,
  onNextClick,
  buttonVariant,
  copy,
  renderChevron,
  ...props
}: React.ComponentProps<NonNullable<CustomComponents["Nav"]>> & {
  buttonVariant: React.ComponentProps<typeof Button>["variant"]
  copy: ReturnType<typeof getCalendarCopy>
  renderChevron?: (params: {
    orientation: "left" | "right" | "down"
    className?: string
    props: Record<string, unknown>
  }) => React.ReactElement
}) {
  const { classNames, dayPickerProps, goToMonth, months } = useDayPicker()
  const currentMonth = months[0]?.date ?? new Date()
  const previousYearMonth = shiftMonth(currentMonth, -12)
  const nextYearMonth = shiftMonth(currentMonth, 12)
  const startMonth = normalizeMonthBoundary(dayPickerProps.startMonth)
  const endMonth = normalizeMonthBoundary(dayPickerProps.endMonth)
  const canGoToPreviousYear =
    previousMonth !== undefined &&
    (startMonth === undefined || previousYearMonth >= startMonth)
  const canGoToNextYear =
    nextMonth !== undefined &&
    (endMonth === undefined || nextYearMonth <= endMonth)

  return (
    <nav {...props} className={cn(classNames.nav, className)}>
      <div className="flex items-center gap-1">
        <Button
          type="button"
          variant={buttonVariant}
          className={classNames.button_previous}
          disabled={!canGoToPreviousYear}
          aria-label={copy.previousYearLabel}
          onClick={() => {
            if (!canGoToPreviousYear) {
              return
            }

            goToMonth(previousYearMonth)
          }}
        >
          <DefaultCalendarChevron orientation="left-double" props={{}} />
        </Button>
        <Button
          type="button"
          variant={buttonVariant}
          className={classNames.button_previous}
          disabled={!previousMonth}
          aria-label={copy.previousMonthLabel}
          onClick={onPreviousClick}
        >
          {renderChevron ? (
            renderChevron({
              orientation: "left",
              className: classNames.chevron,
              props: {},
            })
          ) : (
            <DefaultCalendarChevron
              orientation="left"
              className={classNames.chevron}
              props={{}}
            />
          )}
        </Button>
      </div>

      <div className="flex items-center gap-1">
        <Button
          type="button"
          variant={buttonVariant}
          className={classNames.button_next}
          disabled={!nextMonth}
          aria-label={copy.nextMonthLabel}
          onClick={onNextClick}
        >
          {renderChevron ? (
            renderChevron({
              orientation: "right",
              className: classNames.chevron,
              props: {},
            })
          ) : (
            <DefaultCalendarChevron
              orientation="right"
              className={classNames.chevron}
              props={{}}
            />
          )}
        </Button>
        <Button
          type="button"
          variant={buttonVariant}
          className={classNames.button_next}
          disabled={!canGoToNextYear}
          aria-label={copy.nextYearLabel}
          onClick={() => {
            if (!canGoToNextYear) {
              return
            }

            goToMonth(nextYearMonth)
          }}
        >
          <DefaultCalendarChevron orientation="right-double" props={{}} />
        </Button>
      </div>
    </nav>
  )
}

function shiftMonth(month: Date, months: number) {
  return new Date(month.getFullYear(), month.getMonth() + months, 1)
}

function normalizeMonthBoundary(month: Date | undefined) {
  if (!month) {
    return undefined
  }

  return new Date(month.getFullYear(), month.getMonth(), 1)
}

function DefaultCalendarChevron({
  orientation,
  className,
  props,
}: {
  orientation: "left" | "right" | "down" | "left-double" | "right-double"
  className?: string
  props: Record<string, unknown>
}) {
  const iconProps = { className: cn("size-4", className), ...props }

  if (orientation === "left-double") {
    return <ChevronsLeftIcon {...iconProps} />
  }

  if (orientation === "right-double") {
    return <ChevronsRightIcon {...iconProps} />
  }

  if (orientation === "left") {
    return <ChevronLeftIcon {...iconProps} />
  }

  if (orientation === "right") {
    return <ChevronRightIcon {...iconProps} />
  }

  return <ChevronDownIcon {...iconProps} />
}

export { Calendar, CalendarDayButton }
