import { useEffect } from "react"
import {
  Field,
  FieldDescription,
  FieldGroup,
  FieldLabel,
} from "@workspace/ui-components/stable/field"
import { Input } from "@workspace/ui-components/stable/input"
import type { AuthProfileLabels, AuthProfileModel } from "./types"

export interface ProfileNameSectionProps {
  profile: AuthProfileModel
  labels: AuthProfileLabels
}

export function ProfileNameSection({
  profile,
  labels,
}: ProfileNameSectionProps) {
  const hasNameChange = profile.name.trim() !== profile.displayName

  useEffect(() => {
    if (!hasNameChange || profile.saving) return

    const timer = window.setTimeout(() => {
      void profile.saveProfile()
    }, 800)

    return () => window.clearTimeout(timer)
  }, [hasNameChange, profile])

  return (
    <section className="min-w-0 space-y-5 rounded-xl border border-(--document-card-border) bg-(image:--document-card-background) p-4 shadow-(--document-card-shadow) backdrop-blur-md">
      <FieldGroup>
        <Field>
          <FieldLabel>{labels.name}</FieldLabel>
          <Input
            value={profile.name}
            onValueChange={profile.setName}
            placeholder={labels.namePlaceholder}
            disabled={profile.saving}
          />
          <FieldDescription>{labels.nameHelp}</FieldDescription>
        </Field>
      </FieldGroup>

      <section className="rounded-lg bg-(--agent-chat-content-surface) p-3">
        <h2 className="text-sm font-semibold">{labels.accountSection}</h2>
        <dl className="mt-3 grid gap-x-6 gap-y-3 text-sm sm:grid-cols-[max-content_minmax(0,1fr)]">
          <dt className="font-medium text-(--app-muted-text)">
            {labels.account}
          </dt>
          <dd className="min-w-0 truncate font-medium">{profile.account}</dd>
          <dt className="font-medium text-(--app-muted-text)">{labels.id}</dt>
          <dd className="min-w-0 truncate font-medium">{profile.displayId}</dd>
        </dl>
      </section>

      {profile.saving || profile.status ? (
        <div className="flex flex-wrap items-center gap-3">
          <span className="text-sm text-(--app-muted-text)">
            {profile.saving ? labels.savingProfile : profile.status}
          </span>
        </div>
      ) : null}
    </section>
  )
}
