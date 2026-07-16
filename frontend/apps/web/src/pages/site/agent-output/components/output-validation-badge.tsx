import { StatusBadge } from "@/pages/site/commerce/components/commerce-ui"

export function OutputValidationBadge({
  label,
  status,
}: {
  label: string
  status?: string | null
}) {
  return (
    <span className={`inline-flex ${validationColor(status)}`}>
      <StatusBadge>{label}</StatusBadge>
    </span>
  )
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
