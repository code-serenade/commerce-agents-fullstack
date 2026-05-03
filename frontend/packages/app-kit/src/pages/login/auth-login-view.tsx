import type { ReactNode } from "react"
import { useMemo } from "react"
import { Controller, useForm } from "react-hook-form"
import { useTranslation } from "react-i18next"
import { zodResolver } from "@hookform/resolvers/zod"
import { z } from "zod"
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
  footerLink?: AuthLoginFooterLink
  notice?: AuthLoginNotice
  onSubmit: (values: AuthLoginSubmitValues) => void | Promise<void>
}

const authShellClassName =
  "relative mx-auto grid w-full max-w-5xl overflow-hidden rounded-[var(--ui-radius-xl)] border border-(--app-border) bg-(--app-surface) shadow-[var(--ui-shadow-soft)] sm:grid-cols-2"
const authAsideClassName =
  "relative hidden flex-col justify-between border-r border-(--app-border) bg-[linear-gradient(155deg,var(--app-active-bg)_0%,var(--app-surface)_65%)] p-10 text-(--app-text) sm:flex"
const authContentClassName = "p-8 sm:p-10"
const authFooterClassName = "text-sm text-(--app-muted-text)"

export function AuthLoginView({
  labels,
  footerLink,
  notice,
  onSubmit,
}: AuthLoginViewProps) {
  const { t } = useTranslation(["common"])
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

  return (
    <div className={authShellClassName}>
      <section className={authAsideClassName}>
        <div>
          <p className="text-xs tracking-[0.22em] text-(--app-muted-text)">
            {labels.brand}
          </p>
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
