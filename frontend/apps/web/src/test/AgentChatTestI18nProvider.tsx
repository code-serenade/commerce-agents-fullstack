import type { ReactNode } from "react"
import type { i18n as I18nInstance } from "i18next"
import { I18nextProvider } from "react-i18next"

export function AgentChatTestI18nProvider({
  children,
  i18n,
}: {
  children: ReactNode
  i18n: I18nInstance
}) {
  return <I18nextProvider i18n={i18n}>{children}</I18nextProvider>
}
