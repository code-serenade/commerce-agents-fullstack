import { registerLocaleMessages } from "@workspace/locales/i18n"
import enMessages from "./en/messages"
import enPermissions from "./en/permissions"
import zhCNMessages from "./zhCN/messages"
import zhCNPermissions from "./zhCN/permissions"

export function registerAdminLocaleMessages() {
  registerLocaleMessages({
    en: {
      ...enMessages,
      admin: {
        ...enMessages.admin,
        permissions: enPermissions,
      },
    },
    zhCN: {
      ...zhCNMessages,
      admin: {
        ...zhCNMessages.admin,
        permissions: zhCNPermissions,
      },
    },
  })
}
