import { useMemo } from "react"
import { Link, useNavigate } from "react-router"
import { useTranslation } from "react-i18next"
import { Controller, useForm } from "react-hook-form"
import { zodResolver } from "@hookform/resolvers/zod"
import { z } from "zod"
import { registerApi } from "@workspace/services/api/auth"
import { Button } from "@workspace/ui-components/stable/button"
import {
  Field,
  FieldError,
  FieldGroup,
  FieldLabel,
} from "@workspace/ui-components/stable/field"
import { Input } from "@workspace/ui-components/stable/input"
import type { AuthRegisterPageProps } from "./types"

type Translate = (key: string, options?: Record<string, unknown>) => string

const authShellClassName =
  "relative mx-auto grid w-full max-w-5xl overflow-hidden rounded-[var(--ui-radius-xl)] border border-(--app-border) bg-(--app-surface) shadow-[var(--ui-shadow-soft)] sm:grid-cols-2"
const authAsideClassName =
  "relative hidden flex-col justify-between border-r border-(--app-border) bg-[linear-gradient(155deg,var(--app-active-bg)_0%,var(--app-surface)_65%)] p-10 text-(--app-text) sm:flex"
const authContentClassName = "p-8 sm:p-10"
const authFooterClassName = "mt-6 text-sm text-(--app-muted-text)"
const authLinkClassName =
  "font-medium text-(--app-text) underline-offset-4 transition hover:underline"

function createAuthRegisterSchema(t: Translate) {
  const emailValidator = z.email()

  return z
    .object({
      username: z
        .string()
        .trim()
        .min(1, t("validation.required", { ns: "common" })),
      displayName: z.string(),
      email: z
        .string()
        .trim()
        .refine(
          (value) => value === "" || emailValidator.safeParse(value).success,
          {
            message: t("validation.emailInvalid", { ns: "common" }),
          }
        ),
      password: z
        .string()
        .trim()
        .min(1, t("validation.required", { ns: "common" })),
      confirmPassword: z
        .string()
        .trim()
        .min(1, t("validation.required", { ns: "common" })),
    })
    .refine((value) => value.password === value.confirmPassword, {
      path: ["confirmPassword"],
      message: t("authRegister.error.passwordMismatch", { ns: "pages" }),
    })
}

export function AuthRegisterPage({
  loginPath = "/login",
  onRegistered,
}: AuthRegisterPageProps) {
  const { t } = useTranslation(["pages", "common"])
  const navigate = useNavigate()
  const registerSchema = useMemo(() => createAuthRegisterSchema(t), [t])
  type RegisterFormValues = z.infer<typeof registerSchema>
  const {
    control,
    handleSubmit,
    formState: { errors, isSubmitting },
  } = useForm<RegisterFormValues>({
    resolver: zodResolver(registerSchema),
    defaultValues: {
      username: "",
      displayName: "",
      email: "",
      password: "",
      confirmPassword: "",
    },
  })

  const handleRegister = handleSubmit(async (values) => {
    const payload = {
      username: values.username.trim(),
      password: values.password,
      display_name: values.displayName.trim() || undefined,
      email: values.email.trim() || undefined,
    }

    try {
      await registerApi(payload)
      await onRegistered?.()
      navigate(loginPath, { replace: true })
    } catch (err) {
      console.error("Register failed:", err)
    }
  })

  return (
    <div className={authShellClassName}>
      <section className={authAsideClassName}>
        <div>
          <p className="text-xs tracking-[0.22em] text-(--app-muted-text)">
            {t("authRegister.brand", { ns: "pages" })}
          </p>
          <h1 className="mt-4 text-4xl/tight font-semibold">
            {t("authRegister.hero.titleLine1", { ns: "pages" })}
            <br />
            {t("authRegister.hero.titleLine2", { ns: "pages" })}
          </h1>
        </div>
        <div className="space-y-3 text-sm text-(--app-muted-text)">
          <p>{t("authRegister.hero.desc1", { ns: "pages" })}</p>
          <p>{t("authRegister.hero.desc2", { ns: "pages" })}</p>
        </div>
      </section>

      <section className={authContentClassName}>
        <div className="space-y-3">
          <p className="text-xs font-medium tracking-[0.16em] text-(--app-muted-text)">
            {t("authRegister.welcome", { ns: "pages" })}
          </p>
          <h2 className="text-3xl font-semibold text-(--app-text)">
            {t("authRegister.title", { ns: "pages" })}
          </h2>
          <p className="text-sm text-(--app-muted-text)">
            {t("authRegister.subtitle", { ns: "pages" })}
          </p>
        </div>

        <form onSubmit={handleRegister} className="mt-8">
          <FieldGroup>
            <Field>
              <FieldLabel>
              {t("authRegister.form.username.label", { ns: "pages" })}
              </FieldLabel>
              <Controller
                control={control}
                name="username"
                render={({ field }) => (
                  <Input
                    value={field.value}
                    onValueChange={field.onChange}
                    type="text"
                    placeholder={t("authRegister.form.username.placeholder", {
                      ns: "pages",
                    })}
                  />
                )}
              />
              <FieldError>{errors.username?.message}</FieldError>
            </Field>

            <Field>
              <FieldLabel>
              {t("authRegister.form.displayName.label", { ns: "pages" })}
              </FieldLabel>
              <Controller
                control={control}
                name="displayName"
                render={({ field }) => (
                  <Input
                    value={field.value}
                    onValueChange={field.onChange}
                    type="text"
                    placeholder={t("authRegister.form.displayName.placeholder", {
                      ns: "pages",
                    })}
                  />
                )}
              />
              <FieldError>{errors.displayName?.message}</FieldError>
            </Field>

            <Field>
              <FieldLabel>
              {t("authRegister.form.email.label", { ns: "pages" })}
              </FieldLabel>
              <Controller
                control={control}
                name="email"
                render={({ field }) => (
                  <Input
                    value={field.value}
                    onValueChange={field.onChange}
                    type="email"
                    placeholder={t("authRegister.form.email.placeholder", {
                      ns: "pages",
                    })}
                  />
                )}
              />
              <FieldError>{errors.email?.message}</FieldError>
            </Field>

            <Field>
              <FieldLabel>
              {t("authRegister.form.password.label", { ns: "pages" })}
              </FieldLabel>
              <Controller
                control={control}
                name="password"
                render={({ field }) => (
                  <Input
                    value={field.value}
                    onValueChange={field.onChange}
                    type="password"
                    placeholder={t("authRegister.form.password.placeholder", {
                      ns: "pages",
                    })}
                  />
                )}
              />
              <FieldError>{errors.password?.message}</FieldError>
            </Field>

            <Field>
              <FieldLabel>
              {t("authRegister.form.confirmPassword.label", { ns: "pages" })}
              </FieldLabel>
              <Controller
                control={control}
                name="confirmPassword"
                render={({ field }) => (
                  <Input
                    value={field.value}
                    onValueChange={field.onChange}
                    type="password"
                    placeholder={t(
                      "authRegister.form.confirmPassword.placeholder",
                      { ns: "pages" }
                    )}
                  />
                )}
              />
              <FieldError>{errors.confirmPassword?.message}</FieldError>
            </Field>

            <Button type="submit" size="lg" fullWidth disabled={isSubmitting}>
              {isSubmitting
                ? t("authRegister.form.submitting", { ns: "pages" })
                : t("authRegister.form.submit", { ns: "pages" })}
            </Button>
          </FieldGroup>
        </form>

        <p className={authFooterClassName}>
          {t("authRegister.footer.toLoginPrefix", { ns: "pages" })}{" "}
          <Link to={loginPath} className={authLinkClassName}>
            {t("authRegister.footer.toLoginAction", { ns: "pages" })}
          </Link>
        </p>
      </section>
    </div>
  )
}
