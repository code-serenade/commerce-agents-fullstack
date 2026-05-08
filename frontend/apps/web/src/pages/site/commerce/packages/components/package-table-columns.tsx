import type {
  PackageHistoryResponse,
  PackageLockResponse,
  PackageResponse,
} from "@/api"
import {
  IdText,
  MutedText,
  StatusBadge,
  toneForStatus,
} from "../../components/commerce-ui"
import type { CommerceColumn } from "../../components/commerce-ui"
import type { CommerceTranslate } from "../../components/commerce-types"
import { packageKey, type PackagesPageState } from "../use-packages-page"

export function packageColumns(
  page: PackagesPageState,
  t: CommerceTranslate
): CommerceColumn<PackageResponse>[] {
  return [
    {
      key: "name",
      header: t("commerce.packages.columns.package"),
      render: (item) => (
        <button
          type="button"
          className="block min-w-0 text-left"
          onClick={() => page.setSelectedPackageKey(packageKey(item))}
        >
          <span className="block truncate font-medium">{item.packageName}</span>
          <IdText>{item.packageType}:{item.packageId}</IdText>
        </button>
      ),
    },
    {
      key: "version",
      header: t("commerce.common.version"),
      render: (item) => item.version,
    },
    {
      key: "status",
      header: t("commerce.common.status"),
      render: (item) => (
        <StatusBadge tone={toneForStatus(item.status)}>{item.status}</StatusBadge>
      ),
    },
  ]
}

export function historyColumns(
  t: CommerceTranslate
): CommerceColumn<PackageHistoryResponse>[] {
  return [
    {
      key: "change",
      header: t("commerce.packages.columns.stateChange"),
      render: (item) => (
        <div className="min-w-0">
          <div className="truncate font-medium">
            {item.fromStatus || "-"} {"->"} {item.toStatus || "-"}
          </div>
          <MutedText>{item.reason || "-"}</MutedText>
        </div>
      ),
    },
    {
      key: "time",
      header: t("commerce.common.time"),
      render: (item) => item.createdAt || "-",
    },
  ]
}

export function lockColumns(
  t: CommerceTranslate
): CommerceColumn<PackageLockResponse>[] {
  return [
    {
      key: "package",
      header: t("commerce.packages.columns.lockedPackage"),
      render: (item) => (
        <div className="min-w-0">
          <div className="truncate font-medium">
            {item.packageName || item.packageId}
          </div>
          <MutedText>{item.packageType}:{item.packageId}</MutedText>
        </div>
      ),
    },
    {
      key: "version",
      header: t("commerce.common.version"),
      render: (item) => item.lockedVersion,
    },
    {
      key: "status",
      header: t("commerce.common.status"),
      render: (item) => (
        <StatusBadge tone={toneForStatus(item.lockedStatus)}>
          {item.lockedStatus}
        </StatusBadge>
      ),
    },
  ]
}
