import { useId, useState, type ChangeEventHandler } from "react"
import { Clock3, X } from "lucide-react"
import { useTranslation } from "react-i18next"
import {
  getTimePickerCopy,
  normalizeLanguage,
} from "@workspace/locales"
import { Button } from "@workspace/ui-core/components/button"
import {
  Popover,
  PopoverContent,
  PopoverTrigger,
} from "@workspace/ui-core/components/popover"
import { cn } from "@workspace/ui-core/lib/utils.js"
import { Time, type TimeValue } from "@workspace/ui-components/stable/time"

export interface TimePickerProps {
  label?: string
  value: string | null
  placeholder?: string
  disabled?: boolean
  className?: string
  onValueChange?: (value: string | null) => void
  onChange?: ChangeEventHandler<HTMLInputElement>
}

export function TimePicker({
  label,
  value,
  placeholder,
  disabled = false,
  className,
  onValueChange,
  onChange,
}: TimePickerProps) {
  const { i18n } = useTranslation()
  const language = normalizeLanguage(i18n.language)
  const copy = getTimePickerCopy(language)
  const [open, setOpen] = useState(false)
  const triggerId = useId()
  const hasValue = Boolean(value)
  const resolvedPlaceholder = placeholder ?? copy.placeholder

  const emitValueChange = (nextValue: string | null) => {
    onValueChange?.(nextValue)
    onChange?.({
      target: { value: nextValue ?? "" },
      currentTarget: { value: nextValue ?? "" },
    } as React.ChangeEvent<HTMLInputElement>)
  }

  const handleValueChange = (nextValue: TimeValue) => {
    emitValueChange(formatTimeValue(nextValue))
  }

  const picker = (
    <Popover
      open={open}
      onOpenChange={setOpen}
    >
      <PopoverTrigger asChild>
        <Button
          type="button"
          variant="outline"
          className={cn(
            "min-w-[120px] w-fit justify-between gap-2 text-left font-normal",
            !hasValue && "text-muted-foreground"
          )}
          aria-haspopup="dialog"
          aria-expanded={open}
          aria-controls={triggerId}
          disabled={disabled}
        >
          <span>{value || resolvedPlaceholder}</span>
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
                emitValueChange(null)
                setOpen(false)
              }}
            >
              <X className="size-4" />
            </span>
          ) : (
            <Clock3 className="size-4 shrink-0 text-muted-foreground" />
          )}
        </Button>
      </PopoverTrigger>
      <PopoverContent
        id={triggerId}
        align="start"
        className="w-auto border-0 bg-transparent p-0 shadow-none"
      >
        <div className="overflow-hidden rounded-[10px] border bg-background">
          <Time
            value={parseTimeValue(value ?? "00:00:00")}
            onValueChange={handleValueChange}
            showSeconds={true}
            ariaLabel={label ?? copy.ariaLabel}
            size="md"
            disabled={disabled}
          />
        </div>
      </PopoverContent>
    </Popover>
  )

  if (!label) {
    return <div className={className}>{picker}</div>
  }

  return (
    <div className={cn("grid gap-1.5", className)}>
      <span className="text-sm font-medium">{label}</span>
      {picker}
    </div>
  )
}

function parseTimeValue(value: string): TimeValue {
  const [hour = "00", minute = "00", second = "00"] = value.split(":")

  return {
    hour,
    minute,
    second,
  }
}

function formatTimeValue(value: TimeValue) {
  return [value.hour, value.minute, value.second ?? "00"].join(":")
}
