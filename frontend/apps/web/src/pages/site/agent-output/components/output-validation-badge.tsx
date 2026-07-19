import { StatusBadge } from "@/pages/site/commerce/components/commerce-ui"

export function OutputValidationBadge({
  label,
  quietOnDesktop = false,
  status,
}: {
  label: string
  quietOnDesktop?: boolean
  status?: string | null
}) {
  return (
    <span
      className={`inline-flex ${validationColor(status)} ${
        quietOnDesktop
          ? `md:items-center md:gap-1.5 md:before:size-1.5 md:before:shrink-0 md:before:rounded-full ${validationDotColor(status)} md:**:data-[slot=badge]:border-transparent md:**:data-[slot=badge]:bg-transparent md:**:data-[slot=badge]:px-0 md:**:data-[slot=badge]:text-muted-foreground/80`
          : ""
      }`}
    >
      <StatusBadge>{label}</StatusBadge>
    </span>
  )
}

function validationDotColor(status?: string | null) {
  switch (status?.trim().toLowerCase()) {
    case "passed":
      return "md:before:bg-emerald-500/60"
    case "unchecked":
      return "md:before:bg-amber-500/60"
    case "failed":
      return "md:before:bg-red-500/60"
    default:
      return "md:before:bg-muted-foreground/40"
  }
}

function validationColor(status?: string | null) {
  switch (status?.trim().toLowerCase()) {
    case "passed":
      return "[&_[data-slot=badge]]:border-emerald-200 [&_[data-slot=badge]]:bg-emerald-50 [&_[data-slot=badge]]:text-emerald-700 dark:[&_[data-slot=badge]]:border-emerald-800 dark:[&_[data-slot=badge]]:bg-emerald-950/55 dark:[&_[data-slot=badge]]:text-emerald-300"
    case "unchecked":
      return "[&_[data-slot=badge]]:border-amber-200 [&_[data-slot=badge]]:bg-amber-50 [&_[data-slot=badge]]:text-amber-700 dark:[&_[data-slot=badge]]:border-amber-800 dark:[&_[data-slot=badge]]:bg-amber-950/55 dark:[&_[data-slot=badge]]:text-amber-300"
    case "failed":
      return "[&_[data-slot=badge]]:border-red-200 [&_[data-slot=badge]]:bg-red-50 [&_[data-slot=badge]]:text-red-700 dark:[&_[data-slot=badge]]:border-red-800 dark:[&_[data-slot=badge]]:bg-red-950/55 dark:[&_[data-slot=badge]]:text-red-300"
    default:
      return "[&_[data-slot=badge]]:border-border [&_[data-slot=badge]]:bg-muted [&_[data-slot=badge]]:text-muted-foreground"
  }
}
