import { useState } from "react"
import { useMutation, useQueryClient } from "@tanstack/react-query"
import {
  activatePackageApi,
  checkPackageLockDriftApi,
  deactivatePackageApi,
  rollbackPackageApi,
  savePackageApi,
  savePackageLocksApi,
  validatePackageApi,
  type PackageLockDriftResponse,
  type PackageResponse,
  type PackageValidationResponse,
} from "@/api"
import {
  notifySuccess,
  toOptionalString,
  type SimpleFormValues,
} from "../components/commerce-ui"
import type { CommerceTranslate } from "../components/commerce-types"
import { packageKey } from "./use-packages-page"
import { packagesQueryKey } from "./use-package-queries"

export function usePackageActions({
  runId,
  selectedPackage,
  setDialog,
  setSelectedPackageKey,
  t,
  workspaceId,
}: {
  runId: string
  selectedPackage: PackageResponse | null
  setDialog: (dialog: "package" | "lock" | null) => void
  setSelectedPackageKey: (key: string | null) => void
  t: CommerceTranslate
  workspaceId: string
}) {
  const queryClient = useQueryClient()
  const [validation, setValidation] = useState<PackageValidationResponse | null>(null)
  const [drift, setDrift] = useState<PackageLockDriftResponse | null>(null)
  const invalidate = async () => {
    await queryClient.invalidateQueries({ queryKey: packagesQueryKey })
  }
  const savePackage = useMutation({
    mutationFn: (values: SimpleFormValues) =>
      savePackageApi({
        compatibility: {},
        dependencies: {},
        manifestPath: toOptionalString(values.manifestPath),
        metadata: {},
        packageId: values.packageId,
        packageName: values.packageName,
        packageType: values.packageType || "skill",
        reason: toOptionalString(values.reason),
        status: values.status || "active",
        version: values.version || "0.1.0",
      }),
    onSuccess: async (result) => {
      setSelectedPackageKey(packageKey(result))
      setDialog(null)
      notifySuccess(t("commerce.packages.notify.saved"))
      await invalidate()
    },
  })
  const validatePackage = useMutation({
    mutationFn: (target: PackageResponse) =>
      validatePackageApi(target.packageType, target.packageId),
    onSuccess: (result) => {
      setValidation(result)
      notifySuccess(
        result.valid
          ? t("commerce.packages.notify.valid")
          : t("commerce.packages.notify.invalid")
      )
    },
  })
  const activatePackage = useMutation({
    mutationFn: (target: PackageResponse) =>
      activatePackageApi(target.packageType, target.packageId, {
        reason: t("commerce.common.manualEnable"),
      }),
    onSuccess: async () => {
      notifySuccess(t("commerce.packages.notify.activated"))
      await invalidate()
    },
  })
  const deactivatePackage = useMutation({
    mutationFn: (target: PackageResponse) =>
      deactivatePackageApi(target.packageType, target.packageId, {
        reason: t("commerce.common.manualDisable"),
      }),
    onSuccess: async () => {
      notifySuccess(t("commerce.packages.notify.deactivated"))
      await invalidate()
    },
  })
  const rollbackPackage = useMutation({
    mutationFn: (target: PackageResponse) =>
      rollbackPackageApi(target.packageType, target.packageId, {
        reason: t("commerce.common.manualRollback"),
      }),
    onSuccess: async () => {
      notifySuccess(t("commerce.packages.notify.rolledBack"))
      await invalidate()
    },
  })
  const saveLock = useMutation({
    mutationFn: (values: SimpleFormValues) =>
      savePackageLocksApi(workspaceId, runId, {
        entries: [lockEntry(values, selectedPackage)],
      }),
    onSuccess: async () => {
      setDialog(null)
      notifySuccess(t("commerce.packages.notify.lockSaved"))
      await invalidate()
    },
  })
  const checkDrift = useMutation({
    mutationFn: () => checkPackageLockDriftApi(workspaceId, runId),
    onSuccess: (result) => {
      setDrift(result)
      notifySuccess(t("commerce.packages.notify.driftChecked"))
    },
  })

  return {
    activatePackage: activatePackage.mutate,
    checkDrift: checkDrift.mutate,
    deactivatePackage: deactivatePackage.mutate,
    drift,
    rollbackPackage: rollbackPackage.mutate,
    saveLock: saveLock.mutate,
    savePackage: savePackage.mutate,
    validatePackage: validatePackage.mutate,
    validation,
  }
}

function lockEntry(values: SimpleFormValues, selectedPackage: PackageResponse | null) {
  return {
    lockedStatus: values.lockedStatus || selectedPackage?.status || "active",
    lockedVersion: values.lockedVersion || selectedPackage?.version || "0.1.0",
    lockSource: values.lockSource || "manual",
    manifestPath:
      toOptionalString(values.manifestPath) ?? selectedPackage?.manifestPath ?? null,
    metadata: {},
    notes: toOptionalString(values.notes),
    packageId: values.packageId || selectedPackage?.packageId || "",
    packageName: values.packageName || selectedPackage?.packageName || "",
    packageType: values.packageType || selectedPackage?.packageType || "skill",
    required: values.required !== "false",
  }
}
