import { useMutation, useQueryClient } from "@tanstack/react-query"
import {
  disableChannelConnectionApi,
  disablePlatformConnectionApi,
  saveChannelConnectionApi,
  savePlatformConnectionApi,
  saveStoreMetricApi,
} from "@/api"
import {
  notifySuccess,
  toNumberOrNull,
  toOptionalString,
  type SimpleFormValues,
} from "../components/commerce-ui"
import type { CommerceTranslate } from "../components/commerce-types"
import type { PlatformDialogName } from "./use-platform-connect-page"
import { platformQueryKey } from "./use-platform-queries"

export function usePlatformActions({
  setDialog,
  t,
}: {
  setDialog: (dialog: PlatformDialogName | null) => void
  t: CommerceTranslate
}) {
  const queryClient = useQueryClient()
  const invalidate = async () => {
    await queryClient.invalidateQueries({ queryKey: platformQueryKey })
  }
  const savePlatform = useMutation({
    mutationFn: (values: SimpleFormValues) =>
      savePlatformConnectionApi({
        accessToken: toOptionalString(values.accessToken),
        appKey: values.appKey,
        appSecret: values.appSecret,
        enabled: values.enabled !== "false",
        extra: {},
        lastSyncedAt: null,
        platform: values.platform,
        refreshToken: toOptionalString(values.refreshToken),
        shopId: toOptionalString(values.shopId),
        shopName: toOptionalString(values.shopName),
        syncMessage: toOptionalString(values.syncMessage),
        syncStatus: values.syncStatus || "idle",
        testResult: {},
        tokenExpiresAt: toOptionalString(values.tokenExpiresAt),
      }),
    onSuccess: async () => {
      setDialog(null)
      notifySuccess(t("commerce.platform.notify.platformSaved"))
      await invalidate()
    },
  })
  const saveChannel = useMutation({
    mutationFn: (values: SimpleFormValues) =>
      saveChannelConnectionApi({
        botToken: toOptionalString(values.botToken),
        channel: values.channel,
        chatId: toOptionalString(values.chatId),
        enabled: values.enabled !== "false",
        extraCredentials: {},
        lastError: null,
        secret: toOptionalString(values.secret),
        webhookUrl: toOptionalString(values.webhookUrl),
      }),
    onSuccess: async () => {
      setDialog(null)
      notifySuccess(t("commerce.platform.notify.channelSaved"))
      await invalidate()
    },
  })
  const saveMetric = useMutation({
    mutationFn: (values: SimpleFormValues) =>
      saveStoreMetricApi({
        dimensionKey: values.dimensionKey ?? "",
        dimensionValue: values.dimensionValue ?? "",
        metricDate: values.metricDate,
        metricKey: values.metricKey,
        metricValue: toNumberOrNull(values.metricValue) ?? 0,
        platform: values.platform,
        source: values.source || "import",
      }),
    onSuccess: async () => {
      setDialog(null)
      notifySuccess(t("commerce.platform.notify.metricSaved"))
      await invalidate()
    },
  })
  const disablePlatform = useMutation({
    mutationFn: disablePlatformConnectionApi,
    onSuccess: async () => {
      notifySuccess(t("commerce.platform.notify.platformDisabled"))
      await invalidate()
    },
  })
  const disableChannel = useMutation({
    mutationFn: disableChannelConnectionApi,
    onSuccess: async () => {
      notifySuccess(t("commerce.platform.notify.channelDisabled"))
      await invalidate()
    },
  })

  return {
    disableChannel: disableChannel.mutate,
    disablePlatform: disablePlatform.mutate,
    saveChannel: saveChannel.mutate,
    saveMetric: saveMetric.mutate,
    savePlatform: savePlatform.mutate,
  }
}
