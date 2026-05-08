import { useState } from "react"
import { useTranslation } from "react-i18next"
import {
  SimpleFormDialog,
  type SimpleField,
  type SimpleFormValues,
} from "../../components/commerce-ui"
import type { CommerceTranslate } from "../../components/commerce-types"
import type { PackagesPageState } from "../use-packages-page"

export function PackageDialogs({ page }: { page: PackagesPageState }) {
  const { t } = useTranslation()

  return (
    <>
      {page.dialog === "package" ? (
        <ControlledDialog
          title={t("commerce.packages.dialogs.package")}
          fields={packageFields(t)}
          initialValues={{
            packageType: "skill",
            status: "active",
            version: "0.1.0",
          }}
          onClose={() => page.setDialog(null)}
          onSubmit={page.savePackage}
        />
      ) : null}
      {page.dialog === "lock" ? (
        <ControlledDialog
          title={t("commerce.packages.dialogs.lock")}
          fields={lockFields(t)}
          initialValues={{
            lockedStatus: page.selectedPackage?.status ?? "active",
            lockedVersion: page.selectedPackage?.version ?? "0.1.0",
            lockSource: "manual",
            manifestPath: page.selectedPackage?.manifestPath ?? "",
            packageId: page.selectedPackage?.packageId ?? "",
            packageName: page.selectedPackage?.packageName ?? "",
            packageType: page.selectedPackage?.packageType ?? "skill",
            required: "true",
          }}
          onClose={() => page.setDialog(null)}
          onSubmit={page.saveLock}
        />
      ) : null}
    </>
  )
}

function ControlledDialog({
  fields,
  initialValues,
  onClose,
  onSubmit,
  title,
}: {
  fields: SimpleField[]
  initialValues: SimpleFormValues
  onClose: () => void
  onSubmit: (values: SimpleFormValues) => void
  title: string
}) {
  const [values, setValues] = useState<SimpleFormValues>(initialValues)

  return (
    <SimpleFormDialog
      open
      title={title}
      fields={fields}
      values={values}
      setValues={setValues}
      onOpenChange={(nextOpen) => {
        if (!nextOpen) onClose()
      }}
      onSubmit={() => onSubmit(values)}
    />
  )
}

function packageFields(t: CommerceTranslate): SimpleField[] {
  return [
    { key: "packageType", label: t("commerce.fields.packageType"), required: true },
    { key: "packageId", label: t("commerce.fields.packageId"), required: true },
    { key: "packageName", label: t("commerce.fields.packageName"), required: true },
    { key: "version", label: t("commerce.common.version") },
    { key: "status", label: t("commerce.common.status") },
    { key: "manifestPath", label: t("commerce.fields.manifestPath") },
    { key: "reason", label: t("commerce.common.reason"), type: "textarea" },
  ]
}

function lockFields(t: CommerceTranslate): SimpleField[] {
  return [
    { key: "packageType", label: t("commerce.fields.packageType"), required: true },
    { key: "packageId", label: t("commerce.fields.packageId"), required: true },
    { key: "packageName", label: t("commerce.fields.packageName") },
    {
      key: "lockedVersion",
      label: t("commerce.fields.lockedVersion"),
      required: true,
    },
    { key: "lockedStatus", label: t("commerce.fields.lockedStatus") },
    { key: "required", label: t("commerce.fields.required") },
    { key: "lockSource", label: t("commerce.fields.lockSource") },
    { key: "manifestPath", label: t("commerce.fields.manifestPath") },
    { key: "notes", label: t("commerce.fields.notes"), type: "textarea" },
  ]
}
