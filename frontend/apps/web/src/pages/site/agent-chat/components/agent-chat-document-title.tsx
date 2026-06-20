import { useEffect } from "react"
import { useTranslation } from "react-i18next"

export function AgentChatDocumentTitle() {
  const { t } = useTranslation()

  useEffect(() => {
    const previousTitle = document.title
    document.title = t("agentChat.thread.agentName")
    return () => {
      document.title = previousTitle
    }
  }, [t])

  return null
}
