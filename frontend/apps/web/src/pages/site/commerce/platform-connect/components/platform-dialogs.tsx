import { useState } from "react"
import { useTranslation } from "react-i18next"
import {
  SimpleFormDialog,
  todayDateString,
  type SimpleField,
  type SimpleFormValues,
} from "../../components/commerce-ui"
import type { CommerceTranslate } from "../../components/commerce-types"
import {
  commerceBooleanOptions,
  commerceStatusOptions,
} from "../../domain/enum-mappings"
import type { PlatformConnectPageState } from "../use-platform-connect-page"

export function PlatformDialogs({ page }: { page: PlatformConnectPageState }) {
  const { t } = useTranslation()

  return (
    <>
      {page.dialog === "platform" ? (
        <ControlledDialog
          title={t("commerce.platform.dialogs.platform")}
          fields={platformFields(page, t)}
          initialValues={{
            enabled: "true",
            platform: platformOptions(page)[0]?.value ?? "douyin",
            syncStatus: "idle",
          }}
          onClose={() => page.setDialog(null)}
          onSubmit={page.savePlatform}
        />
      ) : null}
      {page.dialog === "channel" ? (
        <ControlledDialog
          title={t("commerce.platform.dialogs.channel")}
          fields={channelFields(page, t)}
          initialValues={{
            channel: channelOptions(page)[0]?.value ?? "wecom",
            enabled: "true",
          }}
          onClose={() => page.setDialog(null)}
          onSubmit={page.saveChannel}
        />
      ) : null}
      {page.dialog === "metric" ? (
        <ControlledDialog
          title={t("commerce.platform.dialogs.metric")}
          fields={metricFields(page, t)}
          initialValues={{
            metricDate: todayDateString(),
            metricKey: "gmv",
            platform: platformOptions(page)[0]?.value ?? "douyin",
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

function platformFields(
  page: PlatformConnectPageState,
  t: CommerceTranslate
): SimpleField[] {
  return [
    {
      key: "platform",
      label: t("commerce.common.platform"),
      options: platformOptions(page),
      required: true,
      type: "select",
    },
    { key: "appKey", label: t("commerce.fields.appKey"), required: true },
    { key: "appSecret", label: t("commerce.fields.appSecret"), required: true },
    { key: "accessToken", label: t("commerce.fields.accessToken") },
    { key: "refreshToken", label: t("commerce.fields.refreshToken") },
    { key: "tokenExpiresAt", label: t("commerce.fields.tokenExpiresAt") },
    { key: "shopId", label: t("commerce.fields.shopId") },
    { key: "shopName", label: t("commerce.fields.shopName") },
    {
      key: "enabled",
      label: t("commerce.fields.enabled"),
      options: commerceBooleanOptions(t),
      type: "select",
    },
    {
      key: "syncStatus",
      label: t("commerce.fields.syncStatus"),
      options: commerceStatusOptions(t, ["idle", "syncing", "success", "failed"]),
      type: "select",
    },
    { key: "syncMessage", label: t("commerce.fields.syncMessage") },
  ]
}

function channelFields(
  page: PlatformConnectPageState,
  t: CommerceTranslate
): SimpleField[] {
  return [
    {
      key: "channel",
      label: t("commerce.common.channel"),
      options: channelOptions(page),
      required: true,
      type: "select",
    },
    { key: "webhookUrl", label: t("commerce.fields.webhookUrl") },
    { key: "botToken", label: t("commerce.fields.botToken") },
    { key: "chatId", label: t("commerce.fields.chatId") },
    { key: "secret", label: t("commerce.fields.secret") },
    {
      key: "enabled",
      label: t("commerce.fields.enabled"),
      options: commerceBooleanOptions(t),
      type: "select",
    },
  ]
}

function metricFields(
  page: PlatformConnectPageState,
  t: CommerceTranslate
): SimpleField[] {
  return [
    {
      key: "platform",
      label: t("commerce.common.platform"),
      options: platformOptions(page),
      required: true,
      type: "select",
    },
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

function platformOptions(page: PlatformConnectPageState) {
  const options = page.supportedPlatforms.map((item) => ({
    label: item.display || item.platform,
    value: item.platform,
  }))
  return options.length ? options : fallbackPlatformOptions
}

function channelOptions(page: PlatformConnectPageState) {
  const options = page.supportedChannels.map((item) => ({
    label: item.display || item.channel,
    value: item.channel,
  }))
  return options.length ? options : fallbackChannelOptions
}

const fallbackPlatformOptions = [
  { label: String("taobao"), value: "taobao" },
  { label: String("jd"), value: "jd" },
  { label: String("pdd"), value: "pdd" },
  { label: String("douyin"), value: "douyin" },
]

const fallbackChannelOptions = [
  { label: String("wecom"), value: "wecom" },
  { label: String("feishu"), value: "feishu" },
  { label: String("telegram"), value: "telegram" },
  { label: String("discord"), value: "discord" },
]
