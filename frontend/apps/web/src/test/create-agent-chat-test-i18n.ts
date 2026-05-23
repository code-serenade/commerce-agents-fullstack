import i18next from "i18next"
import enMessages from "../../lang/en/messages"

export async function createAgentChatTestI18n() {
  const i18n = i18next.createInstance()
  await i18n.init({
    lng: "en",
    resources: {
      en: {
        translation: enMessages,
      },
    },
  })
  return i18n
}
