import { useMemo, useState } from "react"
import { Link, useNavigate } from "react-router"
import { useTranslation } from "react-i18next"
import { Controller, useForm } from "react-hook-form"
import { zodResolver } from "@hookform/resolvers/zod"
import { z } from "zod"
import { CircleHelp, Eye, EyeOff } from "lucide-react"
import { registerApi } from "@workspace/services/api/auth"
import { Button } from "@workspace/ui-components/stable/button"
import {
  Field,
  FieldError,
  FieldGroup,
  FieldLabel,
} from "@workspace/ui-components/stable/field"
import { Input } from "@workspace/ui-components/stable/input"
import type {
  AuthRegisterExtraFieldDefinition,
  AuthRegisterPageProps,
} from "./types"

type Translate = (key: string, options?: Record<string, unknown>) => string

const authShellClassName =
  "relative mx-auto grid w-full max-w-5xl overflow-hidden rounded-[var(--ui-radius-xl)] border border-(--app-border) bg-(--app-surface) shadow-[var(--ui-shadow-soft)] sm:grid-cols-2"
const authAsideClassName =
  "relative hidden flex-col justify-between border-r border-(--app-border) bg-[linear-gradient(155deg,var(--app-active-bg)_0%,var(--app-surface)_65%)] p-10 text-(--app-text) sm:flex"
const authContentClassName = "p-8 sm:p-10"
const authFooterClassName = "mt-6 text-sm text-(--app-muted-text)"
const authLinkClassName =
  "font-medium text-(--app-text) underline-offset-4 transition hover:underline"
const minimalBrandClassName =
  "m-0 text-[22px] font-bold tracking-normal text-(--auth-minimal-heading-foreground) drop-shadow-[var(--auth-minimal-brand-shadow)]"
const minimalSubtitleClassName =
  "mx-auto my-0 max-w-[440px] text-[15px] leading-[1.8] text-(--auth-minimal-muted-foreground)"
const minimalInputClassName =
  "h-12 w-full rounded-xl border border-(--auth-minimal-field-border) [background:var(--auth-minimal-field-background)] px-3.5 text-[15px] text-(--auth-minimal-field-foreground) shadow-[var(--auth-minimal-field-shadow)] transition outline-none focus:border-(--auth-minimal-field-focus-border) focus:shadow-[var(--auth-minimal-field-focus-shadow)]"
const minimalLabelClassName =
  "mb-[9px] block text-sm font-bold text-(--auth-minimal-label-foreground)"
const minimalHelpIconClassName =
  "inline-flex size-4.5 items-center justify-center rounded-full text-(--auth-minimal-muted-foreground) transition hover:text-(--auth-minimal-action-foreground) focus-visible:ring-2 focus-visible:ring-ring/25 focus-visible:outline-none"
const minimalTooltipClassName =
  "pointer-events-none invisible absolute bottom-6 left-1/2 z-20 -translate-x-1/2 rounded-xl border border-(--auth-minimal-notice-border) bg-(--auth-minimal-popover-background) px-3 py-2 text-xs leading-relaxed font-medium text-(--auth-minimal-label-foreground) opacity-0 shadow-[var(--auth-minimal-popover-shadow)] backdrop-blur-xl transition group-focus-within/help:visible group-focus-within/help:opacity-100 group-hover/help:visible group-hover/help:opacity-100"
const minimalIconButtonClassName =
  "absolute top-1/2 right-3 inline-flex size-8 -translate-y-1/2 items-center justify-center rounded-lg text-(--auth-minimal-muted-foreground) transition hover:bg-(--auth-minimal-icon-hover-background) hover:text-(--auth-minimal-action-foreground) focus-visible:ring-2 focus-visible:ring-ring/25 focus-visible:outline-none"
const minimalSubmitClassName =
  "mt-[26px] h-12 w-full rounded-xl border-0 bg-[image:var(--auth-minimal-submit-background)] text-base font-bold text-(--auth-minimal-submit-foreground) shadow-[var(--auth-minimal-submit-shadow)] transition disabled:cursor-not-allowed disabled:opacity-65"
const minimalFooterClassName =
  "mt-6 flex justify-center gap-2 text-sm text-(--auth-minimal-muted-foreground)"
const minimalFooterLinkClassName =
  "font-bold text-(--auth-minimal-action-foreground) underline-offset-4 transition hover:underline"

function createAuthRegisterSchema(
  t: Translate,
  extraFields: AuthRegisterExtraFieldDefinition[]
) {
  const emailValidator = z.email()

  return z
    .object({
      username: z
        .string()
        .trim()
        .min(1, t("validation.required", { ns: "common" }))
        .min(3, t("-10007", { ns: "errors" }))
        .max(32, t("-10007", { ns: "errors" }))
        .regex(/^[A-Za-z0-9_-]+$/, t("-10008", { ns: "errors" })),
      displayName: z.string(),
      email: z
        .string()
        .trim()
        .refine(
          (value) => value === "" || emailValidator.safeParse(value).success,
          {
            message: t("-10012", { ns: "errors" }),
          }
        ),
      password: z
        .string()
        .trim()
        .min(1, t("validation.required", { ns: "common" }))
        .min(8, t("-10011", { ns: "errors" }))
        .max(128, t("-10011", { ns: "errors" })),
      confirmPassword: z
        .string()
        .trim()
        .min(1, t("validation.required", { ns: "common" })),
      extraFields: z
        .record(z.string(), z.string())
        .superRefine((values, ctx) => {
          extraFields.forEach((field) => {
            const value = values[field.name]?.trim() ?? ""
            if (field.required && !value) {
              ctx.addIssue({
                code: "custom",
                path: [field.name],
                message: t("validation.required", { ns: "common" }),
              })
            }
            if (field.maxLength && value.length > field.maxLength) {
              ctx.addIssue({
                code: "custom",
                path: [field.name],
                message: t("validation.maxLength", {
                  max: field.maxLength,
                  ns: "common",
                }),
              })
            }
          })
        }),
    })
    .refine((value) => value.password === value.confirmPassword, {
      path: ["confirmPassword"],
      message: t("authRegister.error.passwordMismatch", { ns: "pages" }),
    })
}

export function AuthRegisterPage({
  brandLogo,
  extraFields = [],
  loginPath = "/login",
  variant = "split",
  onRegistered,
}: AuthRegisterPageProps) {
  const { t } = useTranslation(["pages", "common", "errors", "components"])
  const navigate = useNavigate()
  const [isPasswordVisible, setIsPasswordVisible] = useState(false)
  const [isConfirmPasswordVisible, setIsConfirmPasswordVisible] =
    useState(false)
  const registerSchema = useMemo(
    () => createAuthRegisterSchema(t, extraFields),
    [extraFields, t]
  )
  type RegisterFormValues = z.infer<typeof registerSchema>
  const extraFieldDefaultValues = useMemo(
    () =>
      Object.fromEntries(
        extraFields.map((field) => [field.name, ""])
      ) as Record<string, string>,
    [extraFields]
  )
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
      extraFields: extraFieldDefaultValues,
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
      await onRegistered?.({
        username: payload.username,
        displayName: values.displayName.trim(),
        email: values.email.trim(),
        extraFields: normalizeExtraFieldValues(values.extraFields),
      })
      navigate(loginPath, { replace: true })
    } catch (err) {
      console.error("Register failed:", err)
    }
  })

  if (variant === "minimal") {
    return (
      <section className="relative z-10 mx-auto mt-[-4px] w-full max-w-[680px]">
        <div className="mb-[18px] flex items-center justify-center gap-3">
          {brandLogo ? <div className="shrink-0">{brandLogo}</div> : null}
          <p className={minimalBrandClassName}>
            {t("authRegister.brand", { ns: "pages" })}
          </p>
        </div>

        <header className="mb-[28px] text-center">
          <p className={minimalSubtitleClassName}>
            {t("authRegister.subtitle", { ns: "pages" })}
          </p>
        </header>

        <form onSubmit={handleRegister} className="px-0.5 pt-1">
          <div className="grid gap-[18px] sm:grid-cols-2">
            <label className="block">
              <span
                className={`${minimalLabelClassName} flex items-center gap-1.5`}
              >
                <span>
                  {t("authRegister.form.username.label", { ns: "pages" })}
                </span>
                <span className="group/help relative inline-flex">
                  <span
                    aria-label={t("authRegister.form.username.help", {
                      ns: "pages",
                    })}
                    className={minimalHelpIconClassName}
                    role="img"
                    tabIndex={0}
                  >
                    <CircleHelp aria-hidden="true" className="size-3.5" />
                  </span>
                  <span className={`${minimalTooltipClassName} w-[260px]`}>
                    {t("authRegister.form.username.help", { ns: "pages" })}
                  </span>
                </span>
              </span>
              <Controller
                control={control}
                name="username"
                render={({ field }) => (
                  <input
                    value={field.value}
                    onChange={(event) => field.onChange(event.target.value)}
                    type="text"
                    placeholder={t("authRegister.form.username.placeholder", {
                      ns: "pages",
                    })}
                    className={minimalInputClassName}
                  />
                )}
              />
              <FieldError>{errors.username?.message}</FieldError>
            </label>

            <label className="block">
              <span className={minimalLabelClassName}>
                {t("authRegister.form.displayName.label", { ns: "pages" })}
              </span>
              <Controller
                control={control}
                name="displayName"
                render={({ field }) => (
                  <input
                    value={field.value}
                    onChange={(event) => field.onChange(event.target.value)}
                    type="text"
                    placeholder={t(
                      "authRegister.form.displayName.placeholder",
                      { ns: "pages" }
                    )}
                    className={minimalInputClassName}
                  />
                )}
              />
              <FieldError>{errors.displayName?.message}</FieldError>
            </label>

            <label className="block">
              <span className={minimalLabelClassName}>
                {t("authRegister.form.email.label", { ns: "pages" })}
              </span>
              <Controller
                control={control}
                name="email"
                render={({ field }) => (
                  <input
                    value={field.value}
                    onChange={(event) => field.onChange(event.target.value)}
                    type="email"
                    placeholder={t("authRegister.form.email.placeholder", {
                      ns: "pages",
                    })}
                    className={minimalInputClassName}
                  />
                )}
              />
              <FieldError>{errors.email?.message}</FieldError>
            </label>

            {extraFields.map((extraField) => (
              <label key={extraField.name} className="block">
                <span
                  className={`${minimalLabelClassName} flex items-center gap-1.5`}
                >
                  <span>{extraField.label}</span>
                  {extraField.helpText ? (
                    <span className="group/help relative inline-flex">
                      <span
                        aria-label={extraField.helpText}
                        className={minimalHelpIconClassName}
                        role="img"
                        tabIndex={0}
                      >
                        <CircleHelp aria-hidden="true" className="size-3.5" />
                      </span>
                      <span className={`${minimalTooltipClassName} w-[280px]`}>
                        {extraField.helpText}
                      </span>
                    </span>
                  ) : null}
                </span>
                <Controller
                  control={control}
                  name={`extraFields.${extraField.name}`}
                  render={({ field }) => (
                    <input
                      value={field.value}
                      onChange={(event) => field.onChange(event.target.value)}
                      type="text"
                      placeholder={extraField.placeholder}
                      className={minimalInputClassName}
                    />
                  )}
                />
                <FieldError>
                  {errors.extraFields?.[extraField.name]?.message}
                </FieldError>
              </label>
            ))}

            <label className="block">
              <span className={minimalLabelClassName}>
                {t("authRegister.form.password.label", { ns: "pages" })}
              </span>
              <Controller
                control={control}
                name="password"
                render={({ field }) => (
                  <div className="relative">
                    <input
                      value={field.value}
                      onChange={(event) => field.onChange(event.target.value)}
                      type={isPasswordVisible ? "text" : "password"}
                      placeholder={t("authRegister.form.password.placeholder", {
                        ns: "pages",
                      })}
                      className={`${minimalInputClassName} pr-12`}
                    />
                    <button
                      type="button"
                      className={minimalIconButtonClassName}
                      aria-label={
                        isPasswordVisible
                          ? t("auth.passwordVisibility.hidePassword", {
                              ns: "components",
                            })
                          : t("auth.passwordVisibility.showPassword", {
                              ns: "components",
                            })
                      }
                      aria-pressed={isPasswordVisible}
                      onClick={() =>
                        setIsPasswordVisible((visible) => !visible)
                      }
                    >
                      {isPasswordVisible ? (
                        <EyeOff aria-hidden="true" className="size-4.5" />
                      ) : (
                        <Eye aria-hidden="true" className="size-4.5" />
                      )}
                    </button>
                  </div>
                )}
              />
              <FieldError>{errors.password?.message}</FieldError>
            </label>

            <label className="block">
              <span className={minimalLabelClassName}>
                {t("authRegister.form.confirmPassword.label", { ns: "pages" })}
              </span>
              <Controller
                control={control}
                name="confirmPassword"
                render={({ field }) => (
                  <div className="relative">
                    <input
                      value={field.value}
                      onChange={(event) => field.onChange(event.target.value)}
                      type={isConfirmPasswordVisible ? "text" : "password"}
                      placeholder={t(
                        "authRegister.form.confirmPassword.placeholder",
                        { ns: "pages" }
                      )}
                      className={`${minimalInputClassName} pr-12`}
                    />
                    <button
                      type="button"
                      className={minimalIconButtonClassName}
                      aria-label={
                        isConfirmPasswordVisible
                          ? t("auth.passwordVisibility.hideConfirmPassword", {
                              ns: "components",
                            })
                          : t("auth.passwordVisibility.showConfirmPassword", {
                              ns: "components",
                            })
                      }
                      aria-pressed={isConfirmPasswordVisible}
                      onClick={() =>
                        setIsConfirmPasswordVisible((visible) => !visible)
                      }
                    >
                      {isConfirmPasswordVisible ? (
                        <EyeOff aria-hidden="true" className="size-4.5" />
                      ) : (
                        <Eye aria-hidden="true" className="size-4.5" />
                      )}
                    </button>
                  </div>
                )}
              />
              <FieldError>{errors.confirmPassword?.message}</FieldError>
            </label>
          </div>

          <button
            type="submit"
            disabled={isSubmitting}
            className={minimalSubmitClassName}
          >
            {isSubmitting
              ? t("authRegister.form.submitting", { ns: "pages" })
              : t("authRegister.form.submit", { ns: "pages" })}
          </button>
        </form>

        <p className={minimalFooterClassName}>
          {t("authRegister.footer.toLoginPrefix", { ns: "pages" })}{" "}
          <Link
            to={loginPath}
            className={minimalFooterLinkClassName}
          >
            {t("authRegister.footer.toLoginAction", { ns: "pages" })}
          </Link>
        </p>
      </section>
    )
  }

  return (
    <div className={authShellClassName}>
      <section className={authAsideClassName}>
        <div>
          <div className="flex items-center gap-3">
            {brandLogo ? <div className="shrink-0">{brandLogo}</div> : null}
            <p className="text-xs tracking-[0.22em] text-(--app-muted-text)">
              {t("authRegister.brand", { ns: "pages" })}
            </p>
          </div>
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
          {brandLogo ? (
            <div className="mb-5 flex items-center gap-3 sm:hidden">
              {brandLogo}
              <span className="text-sm font-semibold text-(--app-text)">
                {t("authRegister.brand", { ns: "pages" })}
              </span>
            </div>
          ) : null}
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
              <p className="text-[11px] text-(--app-muted-text)">
                {t("authRegister.form.username.help", { ns: "pages" })}
              </p>
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
                    placeholder={t(
                      "authRegister.form.displayName.placeholder",
                      {
                        ns: "pages",
                      }
                    )}
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

            {extraFields.map((extraField) => (
              <Field key={extraField.name}>
                <FieldLabel>{extraField.label}</FieldLabel>
                <Controller
                  control={control}
                  name={`extraFields.${extraField.name}`}
                  render={({ field }) => (
                    <Input
                      value={field.value}
                      onValueChange={field.onChange}
                      type="text"
                      placeholder={extraField.placeholder}
                    />
                  )}
                />
                {extraField.helpText ? (
                  <p className="text-[11px] text-(--app-muted-text)">
                    {extraField.helpText}
                  </p>
                ) : null}
                <FieldError>
                  {errors.extraFields?.[extraField.name]?.message}
                </FieldError>
              </Field>
            ))}

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

function normalizeExtraFieldValues(values: Record<string, string>) {
  return Object.fromEntries(
    Object.entries(values).map(([key, value]) => [key, value.trim()])
  )
}
