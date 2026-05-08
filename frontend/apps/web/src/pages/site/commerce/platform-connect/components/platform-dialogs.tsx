import { useState } from "react"
import { useTranslation } from "react-i18next"
import {
  SimpleFormDialog,
  todayDateString,
  type SimpleField,
  type SimpleFormValues,
} from "../../components/commerce-ui"
import type { CommerceTranslate } from "../../components/commerce-types"
import type { PlatformConnectPageState } from "../use-platform-connect-page"

export function PlatformDialogs({ page }: { page: PlatformConnectPageState }) {
  const { t } = useTranslation()

  return (
    <>
      {page.dialog === "platform" ? (
        <ControlledDialog
          title={t("commerce.platform.dialogs.platform")}
          fields={platformFields(t)}
          initialValues={{ enabled: "true", platform: "douyin", syncStatus: "idle" }}
          onClose={() => page.setDialog(null)}
          onSubmit={page.savePlatform}
        />
      ) : null}
      {page.dialog === "channel" ? (
        <ControlledDialog
          title={t("commerce.platform.dialogs.channel")}
          fields={channelFields(t)}
          initialValues={{ channel: "wecom", enabled: "true" }}
          onClose={() => page.setDialog(null)}
          onSubmit={page.saveChannel}
        />
      ) : null}
      {page.dialog === "metric" ? (
        <ControlledDialog
          title={t("commerce.platform.dialogs.metric")}
          fields={metricFields(t)}
          initialValues={{
            metricDate: todayDateString(),
            metricKey: "gmv",
            platform: "douyin",
            source: "import",
          }}
          onClose={() => page.setDialog(null)}
          onSubmit={page.saveMetric}
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

function platformFields(t: CommerceTranslate): SimpleField[] {
  return [
    { key: "platform", label: t("commerce.common.platform"), required: true },
    { key: "appKey", label: t("commerce.fields.appKey"), required: true },
    { key: "appSecret", label: t("commerce.fields.appSecret"), required: true },
    { key: "accessToken", label: t("commerce.fields.accessToken") },
    { key: "refreshToken", label: t("commerce.fields.refreshToken") },
    { key: "tokenExpiresAt", label: t("commerce.fields.tokenExpiresAt") },
    { key: "shopId", label: t("commerce.fields.shopId") },
    { key: "shopName", label: t("commerce.fields.shopName") },
    { key: "enabled", label: t("commerce.fields.enabled") },
    { key: "syncStatus", label: t("commerce.fields.syncStatus") },
    { key: "syncMessage", label: t("commerce.fields.syncMessage") },
  ]
}

function channelFields(t: CommerceTranslate): SimpleField[] {
  return [
    { key: "channel", label: t("commerce.common.channel"), required: true },
    { key: "webhookUrl", label: t("commerce.fields.webhookUrl") },
    { key: "botToken", label: t("commerce.fields.botToken") },
    { key: "chatId", label: t("commerce.fields.chatId") },
    { key: "secret", label: t("commerce.fields.secret") },
    { key: "enabled", label: t("commerce.fields.enabled") },
  ]
}

function metricFields(t: CommerceTranslate): SimpleField[] {
  return [
    { key: "platform", label: t("commerce.common.platform"), required: true },
    { key: "metricDate", label: t("commerce.fields.metricDate"), required: true },
    { key: "metricKey", label: t("commerce.fields.metricKey"), required: true },
    {
      key: "metricValue",
      label: t("commerce.fields.metricValue"),
      required: true,
      type: "number",
    },
    { key: "dimensionKey", label: t("commerce.fields.dimensionKey") },
    { key: "dimensionValue", label: t("commerce.fields.dimensionValue") },
    { key: "source", label: t("commerce.common.source") },
  ]
}
