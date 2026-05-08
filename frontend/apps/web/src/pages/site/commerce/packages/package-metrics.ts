import type { MetricCardsItem } from "@workspace/app-kit"
import type { PackageLockResponse, PackageResponse } from "@/api"
import type { CommerceTranslate } from "../components/commerce-types"

export function buildPackageMetricCards(
  t: CommerceTranslate,
  data: {
    locks: PackageLockResponse[]
    packages: PackageResponse[]
  }
): MetricCardsItem[] {
  return [
    {
      key: "packages",
      label: t("commerce.packages.metrics.packages"),
      value: data.packages.length,
      tail: t("commerce.common.countActive", {
        count: data.packages.filter((item) => item.status === "active").length,
      }),
      variant: "accent",
    },
    {
      key: "skills",
      label: t("commerce.packages.metrics.skills"),
      value: data.packages.filter((item) => item.packageType === "skill").length,
      tail: t("commerce.packages.metrics.skillsTail"),
      variant: "success",
    },
    {
      key: "locks",
      label: t("commerce.packages.metrics.locks"),
      value: data.locks.length,
      tail: t("commerce.packages.metrics.locksTail"),
      variant: "default",
    },
    {
      key: "disabled",
      label: t("commerce.packages.metrics.disabled"),
      value: data.packages.filter((item) =>
        ["disabled", "deprecated"].includes(item.status)
      ).length,
      tail: t("commerce.packages.metrics.disabledTail"),
      variant: "warning",
    },
  ]
}
