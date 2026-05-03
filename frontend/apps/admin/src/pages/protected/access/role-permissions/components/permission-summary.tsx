import { useTranslation } from "react-i18next"
import { CheckCheck, FolderTree, SquareMousePointer } from "lucide-react"
import type { PermissionSummaryType } from "../types"

export function PermissionSummary({
  selectedCount,
  summary,
}: {
  selectedCount: number
  summary: Record<PermissionSummaryType, number>
}) {
  const { t } = useTranslation()
  const items = [
    {
      icon: <FolderTree className="size-4" />,
      label: t("admin.access.rolePermissions.summary.group"),
      value: summary.group,
    },
    {
      icon: <SquareMousePointer className="size-4" />,
      label: t("admin.access.rolePermissions.summary.action"),
      value: summary.action,
    },
    {
      icon: <CheckCheck className="size-4" />,
      label: t("admin.access.rolePermissions.summary.selected"),
      value: selectedCount,
    },
  ]

  return (
    <div className="w-full min-w-0">
      <div className="grid w-full min-w-0 grid-cols-3 gap-1.5 sm:flex sm:w-max sm:gap-3">
        {items.map((item) => (
          <div
            key={item.label}
            className="flex min-h-14 min-w-0 items-center gap-1.5 rounded-lg bg-(--surface) px-2 sm:min-h-18 sm:w-33 sm:shrink-0 sm:gap-2.5 sm:rounded-xl sm:px-4"
          >
            <span className="flex size-3.5 shrink-0 items-center justify-center text-(--foreground) sm:size-4">
              {item.icon}
            </span>
            <div className="min-w-0">
              <p className="truncate text-xs/4 font-medium text-(--muted-foreground) sm:text-sm">
                {item.label}
              </p>
              <p className="truncate text-lg leading-none font-semibold text-(--foreground) sm:mt-0.5 sm:text-xl">
                {item.value}
              </p>
            </div>
          </div>
        ))}
      </div>
    </div>
  )
}
