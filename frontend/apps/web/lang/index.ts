import { registerLocaleMessages } from "@workspace/locales/i18n"
import enMessages from "./en/messages"
import zhCNMessages from "./zhCN/messages"

export function registerWebLocaleMessages() {
  registerLocaleMessages({
    en: enMessages,
    zhCN: zhCNMessages,
  })
}
