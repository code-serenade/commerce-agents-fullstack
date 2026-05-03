import { useId, useState } from "react"
import { CalendarDays, X } from "lucide-react"
import { useTranslation } from "react-i18next"
import {
  getDatePickerCopy,
  normalizeLanguage,
} from "@workspace/locales"
import { Button } from "@workspace/ui-core/components/button"
import {
  Popover,
  PopoverContent,
  PopoverTrigger,
} from "@workspace/ui-core/components/popover"
import { cn } from "@workspace/ui-core/lib/utils.js"
import { Calendar, type CalendarProps } from "@workspace/ui-components/stable/calendar"
import { toIntlLocale } from "./shared"

type CalendarSingleValue = Date | undefined

export interface DatePickerProps {
  value?: CalendarSingleValue
  onValueChange?: (value: CalendarSingleValue) => void
  placeholder?: string
  disabled?: boolean
  className?: string
  calendarProps?: Omit<CalendarProps, "mode" | "value" | "onValueChange">
}

function formatDate(value: Date, language: ReturnType<typeof normalizeLanguage>) {
  return new Intl.DateTimeFormat(toIntlLocale(language), {
    year: "numeric",
    month: "2-digit",
    day: "2-digit",
  }).format(value)
}

export function DatePicker({
  value,
  onValueChange,
  placeholder,
  disabled = false,
  className,
  calendarProps,
}: DatePickerProps) {
  const { i18n } = useTranslation()
  const language = normalizeLanguage(i18n.language)
  const copy = getDatePickerCopy(language)
  const [open, setOpen] = useState(false)
  const triggerId = useId()
  const resolvedPlaceholder = placeholder ?? copy.placeholder
  const displayValue = value ? formatDate(value, language) : resolvedPlaceholder
  const hasValue = Boolean(value)

  return (
    <Popover open={open} onOpenChange={setOpen}>
      <PopoverTrigger asChild>
        <Button
          type="button"
          variant="outline"
          className={cn(
            "min-w-[140px] w-fit justify-between gap-2 text-left font-normal",
            !hasValue && "text-muted-foreground",
            className
          )}
          aria-haspopup="dialog"
          aria-expanded={open}
          aria-controls={triggerId}
          disabled={disabled}
        >
          <span>{displayValue}</span>
          {hasValue ? (
            <span
              role="button"
              aria-label={copy.clearLabel}
              tabIndex={-1}
              className="inline-flex size-4 shrink-0 items-center justify-center text-muted-foreground"
              onMouseDown={(event) => {
                event.preventDefault()
                event.stopPropagation()
              }}
              onClick={(event) => {
                event.preventDefault()
                event.stopPropagation()
                onValueChange?.(undefined)
                setOpen(false)
              }}
            >
              <X className="size-4" />
            </span>
          ) : (
            <CalendarDays className="size-4 shrink-0 text-muted-foreground" />
          )}
        </Button>
      </PopoverTrigger>
      <PopoverContent
        id={triggerId}
        align="start"
        className="w-auto p-0"
      >
        <Calendar
          mode="single"
          value={value}
          locale={language}
          onValueChange={(nextValue) => {
            onValueChange?.(nextValue instanceof Date ? nextValue : undefined)
            setOpen(false)
          }}
          captionMode={calendarProps?.captionMode ?? "label"}
          {...calendarProps}
        />
      </PopoverContent>
    </Popover>
  )
}
