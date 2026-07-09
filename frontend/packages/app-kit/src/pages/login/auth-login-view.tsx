import type { ReactNode } from "react"
import { useMemo, useState } from "react"
import { Controller, useForm } from "react-hook-form"
import { useTranslation } from "react-i18next"
import { zodResolver } from "@hookform/resolvers/zod"
import { z } from "zod"
import { Eye, EyeOff } from "lucide-react"
import { Button } from "@workspace/ui-components/stable/button"
import {
  Field,
  FieldError,
  FieldGroup,
  FieldLabel,
} from "@workspace/ui-components/stable/field"
import { Input } from "@workspace/ui-components/stable/input"
import { createAuthLoginSchema } from "./auth-login-schema"

export interface AuthLoginSubmitValues {
  identifier: string
  password: string
}

export interface AuthLoginViewLabels {
  brand: string
  heroTitleLine1: string
  heroTitleLine2: string
  heroDescriptions: string[]
  welcome: string
  title: string
  subtitle: string
  identifierLabel: string
  identifierPlaceholder: string
  passwordLabel: string
  passwordPlaceholder: string
  submit: string
  submitting: string
}

export interface AuthLoginFooterLink {
  prefix: string
  action: ReactNode
}

export interface AuthLoginNotice {
  title: string
  description: string
}

export interface AuthLoginViewProps {
  labels: AuthLoginViewLabels
  brandLogo?: ReactNode
  footerLink?: AuthLoginFooterLink
  notice?: AuthLoginNotice
  variant?: "split" | "minimal"
  onSubmit: (values: AuthLoginSubmitValues) => void | Promise<void>
}

const authShellClassName =
  "relative mx-auto grid w-full max-w-5xl overflow-hidden rounded-[var(--ui-radius-xl)] border border-(--app-border) bg-(--app-surface) shadow-[var(--ui-shadow-soft)] sm:grid-cols-2"
const authAsideClassName =
  "relative hidden flex-col justify-between border-r border-(--app-border) bg-[linear-gradient(155deg,var(--app-active-bg)_0%,var(--app-surface)_65%)] p-10 text-(--app-text) sm:flex"
const authContentClassName = "p-8 sm:p-10"
const authFooterClassName = "text-sm text-(--app-muted-text)"
const minimalBrandClassName =
  "m-0 text-[22px] font-bold tracking-normal text-(--auth-minimal-heading-foreground) drop-shadow-[var(--auth-minimal-brand-shadow)]"
const minimalSubtitleClassName =
  "mx-auto my-0 max-w-[360px] text-[15px] leading-[1.8] text-(--auth-minimal-muted-foreground)"
const minimalNoticeClassName =
  "mb-5 rounded-2xl border border-(--auth-minimal-notice-border) bg-(--auth-minimal-notice-background) p-4 text-left shadow-[var(--auth-minimal-notice-shadow)] backdrop-blur-xl"
const minimalLabelClassName =
  "mb-[9px] block text-sm font-bold text-(--auth-minimal-label-foreground)"
const minimalInputClassName =
  "h-12 w-full rounded-xl border border-(--auth-minimal-field-border) [background:var(--auth-minimal-field-background)] px-3.5 text-[15px] text-(--auth-minimal-field-foreground) shadow-[var(--auth-minimal-field-shadow)] transition outline-none focus:border-(--auth-minimal-field-focus-border) focus:shadow-[var(--auth-minimal-field-focus-shadow)]"
const minimalPasswordInputClassName = `${minimalInputClassName} pr-12`
const minimalIconButtonClassName =
  "absolute top-1/2 right-3 inline-flex size-8 -translate-y-1/2 items-center justify-center rounded-lg text-(--auth-minimal-muted-foreground) transition hover:bg-(--auth-minimal-icon-hover-background) hover:text-(--auth-minimal-action-foreground) focus-visible:ring-2 focus-visible:ring-ring/25 focus-visible:outline-none"
const minimalSubmitClassName =
  "mt-[26px] h-12 w-full rounded-xl border-0 bg-[image:var(--auth-minimal-submit-background)] text-base font-bold text-(--auth-minimal-submit-foreground) shadow-[var(--auth-minimal-submit-shadow)] transition disabled:cursor-not-allowed disabled:opacity-65"
const minimalFooterClassName =
  "mt-6 flex justify-center gap-2 text-sm text-(--auth-minimal-muted-foreground)"

export function AuthLoginView({
  labels,
  brandLogo,
  footerLink,
  notice,
  variant = "split",
  onSubmit,
}: AuthLoginViewProps) {
  const { t } = useTranslation(["common", "components"])
  const [isPasswordVisible, setIsPasswordVisible] = useState(false)
  const loginSchema = useMemo(() => createAuthLoginSchema(t), [t])
  type LoginFormValues = z.infer<typeof loginSchema>
  const {
    control,
    handleSubmit,
    formState: { errors, isSubmitting },
  } = useForm<LoginFormValues>({
    resolver: zodResolver(loginSchema),
    defaultValues: {
      identifier: "",
      password: "",
    },
  })

  const handleLogin = handleSubmit(onSubmit)

  if (variant === "minimal") {
    return (
      <section className="relative z-10 mx-auto mt-[-10px] w-full max-w-[420px]">
        <div className="mb-[18px] flex items-center justify-center gap-3">
          {brandLogo ? <div className="shrink-0">{brandLogo}</div> : null}
          <p className={minimalBrandClassName}>
            {labels.brand}
          </p>
        </div>

        <header className="mb-[34px] text-center">
          <p className={minimalSubtitleClassName}>
            {labels.subtitle}
          </p>
        </header>

        {notice ? (
          <div className={minimalNoticeClassName}>
            <p className="text-sm font-medium text-(--auth-minimal-heading-foreground)">
              {notice.title}
            </p>
            <p className="mt-1 text-sm text-(--auth-minimal-muted-foreground)">
              {notice.description}
            </p>
          </div>
        ) : null}

        <form onSubmit={handleLogin} className="px-0.5 pt-1">
          <div className="grid gap-[19px]">
            <label className="block">
              <span className={minimalLabelClassName}>
                {labels.identifierLabel}
              </span>
              <Controller
                control={control}
                name="identifier"
                render={({ field }) => (
                  <input
                    value={field.value}
                    onChange={(event) => field.onChange(event.target.value)}
                    type="text"
                    placeholder={labels.identifierPlaceholder}
                    className={minimalInputClassName}
                  />
                )}
              />
              <FieldError>{errors.identifier?.message}</FieldError>
            </label>

            <label className="block">
              <span className={minimalLabelClassName}>
                {labels.passwordLabel}
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
                      placeholder={labels.passwordPlaceholder}
                      className={minimalPasswordInputClassName}
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
          </div>

          <button
            type="submit"
            disabled={isSubmitting}
            className={minimalSubmitClassName}
          >
            {isSubmitting ? labels.submitting : labels.submit}
          </button>
        </form>

        {footerLink ? (
          <p className={minimalFooterClassName}>
            {footerLink.prefix} {footerLink.action}
          </p>
        ) : null}
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
              {labels.brand}
            </p>
          </div>
          <h1 className="mt-4 text-4xl/tight font-semibold">
            {labels.heroTitleLine1}
            <br />
            {labels.heroTitleLine2}
          </h1>
        </div>
        <div className="space-y-3 text-sm text-(--app-muted-text)">
          {labels.heroDescriptions.map((description) => (
            <p key={description}>{description}</p>
          ))}
        </div>
      </section>

      <section className={authContentClassName}>
        <div className="space-y-8">
          <div className="space-y-3">
            {brandLogo ? (
              <div className="mb-5 flex items-center gap-3 sm:hidden">
                {brandLogo}
                <span className="text-sm font-semibold text-(--app-text)">
                  {labels.brand}
                </span>
              </div>
            ) : null}
            <p className="text-xs font-medium tracking-[0.16em] text-(--app-muted-text)">
              {labels.welcome}
            </p>
            <h2 className="text-3xl font-semibold text-(--app-text)">
              {labels.title}
            </h2>
            <p className="text-sm text-(--app-muted-text)">{labels.subtitle}</p>
          </div>

          {notice ? (
            <div className="rounded-(--ui-radius-lg) border border-(--app-border) bg-(--app-panel) p-4">
              <p className="text-sm font-medium text-(--app-text)">
                {notice.title}
              </p>
              <p className="mt-1 text-sm text-(--app-muted-text)">
                {notice.description}
              </p>
            </div>
          ) : null}

          <form onSubmit={handleLogin}>
            <FieldGroup>
              <Field>
                <FieldLabel>{labels.identifierLabel}</FieldLabel>
                <Controller
                  control={control}
                  name="identifier"
                  render={({ field }) => (
                    <Input
                      value={field.value}
                      onValueChange={field.onChange}
                      type="text"
                      placeholder={labels.identifierPlaceholder}
                    />
                  )}
                />
                <FieldError>{errors.identifier?.message}</FieldError>
              </Field>

              <Field>
                <FieldLabel>{labels.passwordLabel}</FieldLabel>
                <Controller
                  control={control}
                  name="password"
                  render={({ field }) => (
                    <Input
                      value={field.value}
                      onValueChange={field.onChange}
                      type="password"
                      placeholder={labels.passwordPlaceholder}
                    />
                  )}
                />
                <FieldError>{errors.password?.message}</FieldError>
              </Field>

              <Button type="submit" size="lg" fullWidth disabled={isSubmitting}>
                {isSubmitting ? labels.submitting : labels.submit}
              </Button>
            </FieldGroup>
          </form>

          {footerLink ? (
            <p className={authFooterClassName}>
              {footerLink.prefix} {footerLink.action}
            </p>
          ) : null}
        </div>
      </section>
    </div>
  )
}
