import enCommon from "../lang/en/common"
import enComponents from "../lang/en/components"
import enErrors from "../lang/en/errors"
import enPages from "../lang/en/pages"
import zhCNCommon from "../lang/zhCN/common"
import zhCNComponents from "../lang/zhCN/components"
import zhCNErrors from "../lang/zhCN/errors"
import zhCNPages from "../lang/zhCN/pages"

export type LocaleNamespace =
  | "common"
  | "messages"
  | "pages"
  | "components"
  | "errors"

export type LocaleResources = {
  en: {
    common: typeof enCommon
    pages: typeof enPages
    components: typeof enComponents
    errors: typeof enErrors
  }
  zhCN: {
    common: typeof zhCNCommon
    pages: typeof zhCNPages
    components: typeof zhCNComponents
    errors: typeof zhCNErrors
  }
}

export const localeResources: LocaleResources = {
  en: {
    common: enCommon,
    pages: enPages,
    components: enComponents,
    errors: enErrors,
  },
  zhCN: {
    common: zhCNCommon,
    pages: zhCNPages,
    components: zhCNComponents,
    errors: zhCNErrors,
  },
}
