import { Check, Languages } from "lucide-react"
import { useTranslation } from "react-i18next"
import { normalizeLanguage } from "@workspace/locales"
import {
  DropdownMenu,
  DropdownMenuContent,
  DropdownMenuLabel,
  DropdownMenuRadioGroup,
  DropdownMenuRadioItem,
  DropdownMenuSeparator,
  DropdownMenuTrigger,
} from "@workspace/ui-core/components/dropdown-menu"
import { cn } from "@workspace/ui-core/lib/utils.js"

const LANGUAGE_OPTIONS = [
  { value: "en", nativeLabel: "English", uiLabel: "Language" },
  { value: "zhCN", nativeLabel: "简体中文", uiLabel: "语言" },
] as const

export interface LanguageSwitchProps {
  className?: string
}

export function LanguageSwitch({ className }: LanguageSwitchProps) {
  const { i18n } = useTranslation()
  const currentLanguage = normalizeLanguage(i18n.language)
  const currentValue =
    LANGUAGE_OPTIONS.find((item) => item.value === currentLanguage) ??
    LANGUAGE_OPTIONS[0]
  const label = currentValue.uiLabel

  return (
    <DropdownMenu>
      <DropdownMenuTrigger
        className={cn(
          "inline-flex size-9 items-center justify-center rounded-full border border-transparent bg-transparent text-[var(--info)] shadow-none transition-colors hover:bg-[var(--surface-hover)] hover:text-[var(--info)] data-[state=open]:bg-[var(--surface-hover)]",
          className
        )}
        aria-label={label}
      >
        <Languages aria-hidden="true" className="size-5" strokeWidth={2.4} />
        <span className="sr-only">{label}</span>
      </DropdownMenuTrigger>

      <DropdownMenuContent
        mode="primitive"
        align="end"
        sideOffset={8}
        className="z-[2000] min-w-40 overflow-hidden rounded-lg border border-[var(--select-border)] bg-[var(--select)] p-1 text-[var(--select-foreground)] shadow-lg outline-hidden"
      >
        <DropdownMenuLabel
          mode="primitive"
          className="px-2 py-1.5 text-xs text-muted-foreground"
        >
          {label}
        </DropdownMenuLabel>
        <DropdownMenuSeparator
          mode="primitive"
          className="my-1 bg-[var(--select-border)]"
        />
        <DropdownMenuRadioGroup
          className="space-y-1"
          value={currentValue.value}
          onValueChange={(value) => {
            void i18n.changeLanguage(value)
          }}
        >
          {LANGUAGE_OPTIONS.map((option) => (
            <DropdownMenuRadioItem
              key={option.value}
              mode="primitive"
              value={option.value}
              className={cn(
                "flex min-h-7 cursor-default items-center rounded-md px-2 py-0.5 pr-8 text-left text-sm outline-hidden transition hover:bg-[var(--select-item-hover)] hover:text-[var(--select-item-hover-foreground)] focus:bg-[var(--select-item-hover)] focus:text-[var(--select-item-hover-foreground)] data-[highlighted]:bg-[var(--select-item-hover)] data-[highlighted]:text-[var(--select-item-hover-foreground)]",
                currentValue.value === option.value &&
                  "bg-[var(--select-item-hover)] text-[var(--select-item-hover-foreground)]"
              )}
            >
              <span className="block flex-1 text-left">{option.nativeLabel}</span>
              {currentValue.value === option.value ? (
                <Check
                  aria-hidden="true"
                  className="absolute right-2 size-4 text-[var(--select-item-hover-foreground)]"
                />
              ) : null}
            </DropdownMenuRadioItem>
          ))}
        </DropdownMenuRadioGroup>
      </DropdownMenuContent>
    </DropdownMenu>
  )
}
