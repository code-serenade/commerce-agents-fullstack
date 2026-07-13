import { Link } from "react-router"
import { useTranslation } from "react-i18next"
import {
  AuthLoginView,
  type AuthLoginNotice,
  type AuthLoginSubmitValues,
  type AuthLoginViewLabels,
} from "@workspace/app-kit/login"
import { CommerceAgentLogo } from "@/components/brand"

export interface MinimalLoginFormProps {
  notice?: AuthLoginNotice
  onRegisterClick?: () => void
  onSubmit: (values: AuthLoginSubmitValues) => void | Promise<void>
}

const authLinkClassName =
  "font-bold text-[#2758ff] underline-offset-4 transition hover:underline dark:text-[#76a2ff]"

function createLoginLabels(
  t: ReturnType<typeof useTranslation>["t"]
): AuthLoginViewLabels {
  return {
    brand: t("login.brand"),
    heroTitleLine1: t("login.hero.titleLine1"),
    heroTitleLine2: t("login.hero.titleLine2"),
    heroDescriptions: [
      t("login.hero.desc1"),
      t("login.hero.desc2"),
      t("login.hero.desc3"),
    ],
    welcome: t("login.welcome"),
    title: t("login.title"),
    subtitle: t("login.subtitle"),
    identifierLabel: t("login.form.identifier.label"),
    identifierPlaceholder: t("login.form.identifier.placeholder"),
    passwordLabel: t("login.form.password.label"),
    passwordPlaceholder: t("login.form.password.placeholder"),
    submit: t("login.form.submit"),
    submitting: t("login.form.submitting"),
  }
}

export function MinimalLoginForm({
  notice,
  onRegisterClick,
  onSubmit,
}: MinimalLoginFormProps) {
  const { t } = useTranslation()

  return (
    <AuthLoginView
      labels={createLoginLabels(t)}
      brandLogo={<CommerceAgentLogo className="size-18" size="lg" />}
      variant="minimal"
      notice={notice}
      footerLink={{
        prefix: t("login.footer.toRegisterPrefix"),
        action: (
          <Link
            to="/register"
            className={authLinkClassName}
            onClick={onRegisterClick}
          >
            {t("login.footer.toRegisterAction")}
          </Link>
        ),
      }}
      onSubmit={onSubmit}
    />
  )
}
