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

export function AuthLoginView({
  labels,
  brandLogo,
  footerLink,
  notice,
  variant = "split",
  onSubmit,
}: AuthLoginViewProps) {
  const { t } = useTranslation(["common"])
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
          <p className="m-0 text-[22px] font-bold tracking-normal text-[#111936] drop-shadow-[0_14px_36px_rgba(73,102,255,0.14)] dark:text-[#f7faff] dark:drop-shadow-[0_14px_36px_rgba(73,102,255,0.36)]">
            {labels.brand}
          </p>
        </div>

        <header className="mb-[34px] text-center">
          <p className="mx-auto my-0 max-w-[360px] text-[15px] leading-[1.8] text-[#65738d] dark:text-[#93a4c1]">
            {labels.subtitle}
          </p>
        </header>

        {notice ? (
          <div className="mb-5 rounded-2xl border border-[#dce6f7] bg-white/80 p-4 text-left shadow-[0_18px_42px_rgba(38,76,160,0.1)] backdrop-blur-xl dark:border-[rgba(159,179,217,0.24)] dark:bg-[rgba(11,18,38,0.78)] dark:shadow-[0_22px_54px_rgba(0,0,0,0.28)]">
            <p className="text-sm font-medium text-[#111936] dark:text-[#f7faff]">
              {notice.title}
            </p>
            <p className="mt-1 text-sm text-[#65738d] dark:text-[#93a4c1]">
              {notice.description}
            </p>
          </div>
        ) : null}

        <form onSubmit={handleLogin} className="px-0.5 pt-1">
          <div className="grid gap-[19px]">
            <label className="block">
              <span className="mb-[9px] block text-sm font-bold text-[#344054] dark:text-[#d8e4ff]">
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
                    className="h-12 w-full rounded-xl border border-[#dce6f7] bg-white/90 px-3.5 text-[15px] text-[#101828] shadow-[0_10px_26px_rgba(31,74,145,0.04),inset_0_1px_0_rgba(255,255,255,0.92)] transition outline-none focus:border-[rgba(53,92,255,0.72)] focus:shadow-[0_0_0_4px_rgba(53,92,255,0.08),inset_0_1px_0_rgba(255,255,255,0.9)] dark:border-[rgba(155,176,214,0.22)] dark:bg-[linear-gradient(180deg,rgba(16,25,50,0.9),rgba(11,18,38,0.74))] dark:text-[#eef4ff] dark:shadow-[0_16px_36px_rgba(0,0,0,0.2),inset_0_1px_0_rgba(255,255,255,0.08)] dark:focus:border-[rgba(118,162,255,0.72)] dark:focus:shadow-[0_0_0_4px_rgba(49,93,255,0.14),0_18px_38px_rgba(0,0,0,0.22),inset_0_1px_0_rgba(255,255,255,0.08)]"
                  />
                )}
              />
              <FieldError>{errors.identifier?.message}</FieldError>
            </label>

            <label className="block">
              <span className="mb-[9px] block text-sm font-bold text-[#344054] dark:text-[#d8e4ff]">
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
                      className="h-12 w-full rounded-xl border border-[#dce6f7] bg-white/90 px-3.5 pr-12 text-[15px] text-[#101828] shadow-[0_10px_26px_rgba(31,74,145,0.04),inset_0_1px_0_rgba(255,255,255,0.92)] transition outline-none focus:border-[rgba(53,92,255,0.72)] focus:shadow-[0_0_0_4px_rgba(53,92,255,0.08),inset_0_1px_0_rgba(255,255,255,0.9)] dark:border-[rgba(155,176,214,0.22)] dark:bg-[linear-gradient(180deg,rgba(16,25,50,0.9),rgba(11,18,38,0.74))] dark:text-[#eef4ff] dark:shadow-[0_16px_36px_rgba(0,0,0,0.2),inset_0_1px_0_rgba(255,255,255,0.08)] dark:focus:border-[rgba(118,162,255,0.72)] dark:focus:shadow-[0_0_0_4px_rgba(49,93,255,0.14),0_18px_38px_rgba(0,0,0,0.22),inset_0_1px_0_rgba(255,255,255,0.08)]"
                    />
                    <button
                      type="button"
                      className="absolute top-1/2 right-3 inline-flex size-8 -translate-y-1/2 items-center justify-center rounded-lg text-[#65738d] transition hover:bg-[#eef4ff] hover:text-[#2758ff] focus-visible:ring-2 focus-visible:ring-[rgba(53,92,255,0.24)] focus-visible:outline-none dark:text-[#93a4c1] dark:hover:bg-[rgba(118,162,255,0.12)] dark:hover:text-[#d8e4ff]"
                      aria-label={
                        isPasswordVisible ? "Hide password" : "Show password"
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
            className="mt-[26px] h-12 w-full rounded-xl border-0 bg-[linear-gradient(135deg,#8b3dff,#2758ff)] text-base font-bold text-white shadow-[0_16px_34px_rgba(67,77,255,0.28)] transition disabled:cursor-not-allowed disabled:opacity-65 dark:shadow-[0_18px_42px_rgba(61,88,255,0.36),inset_0_0_0_1px_rgba(255,255,255,0.08)]"
          >
            {isSubmitting ? labels.submitting : labels.submit}
          </button>
        </form>

        {footerLink ? (
          <p className="mt-6 flex justify-center gap-2 text-sm text-[#65738d] dark:text-[#93a4c1]">
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
