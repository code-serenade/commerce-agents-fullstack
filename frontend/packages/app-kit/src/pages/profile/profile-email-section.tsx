import { useState } from "react"
import { Mail } from "lucide-react"
import { Button } from "@workspace/ui-components/stable/button"
import {
  Field,
  FieldDescription,
  FieldGroup,
  FieldLabel,
} from "@workspace/ui-components/stable/field"
import { Input } from "@workspace/ui-components/stable/input"
import type { AuthProfileLabels, AuthProfileModel } from "./types"

export interface ProfileEmailSectionProps {
  profile: AuthProfileModel
  labels: AuthProfileLabels
}

export function ProfileEmailSection({
  profile,
  labels,
}: ProfileEmailSectionProps) {
  const [expanded, setExpanded] = useState(false)
  const canSaveEmail =
    profile.emailInput.trim().length > 0 &&
    profile.emailInput.trim() !== profile.email

  return (
    <section className="min-w-0 border-t border-(--app-border) py-5">
      <div className="flex items-start justify-between gap-4">
        <div className="flex min-w-0 items-start gap-4">
          <span className="mt-1 flex size-5 shrink-0 items-center justify-center text-(--app-muted-text)">
            <Mail className="size-5" />
          </span>
          <div className="min-w-0 space-y-1">
            <h2 className="text-sm font-semibold">{labels.emailSection}</h2>
            <p className="truncate text-sm text-(--app-muted-text)">
              {profile.hasEmail ? profile.email : labels.emailNotSet}
            </p>
          </div>
        </div>

        <Button
          type="button"
          variant="default"
          onClick={() => setExpanded((current) => !current)}
        >
          {expanded
            ? labels.hideEmail
            : profile.hasEmail
              ? labels.changeEmail
              : labels.setEmail}
        </Button>
      </div>

      {expanded ? (
        <div className="mt-6 max-w-xl pl-9">
          <FieldGroup>
            <Field>
              <FieldLabel>{labels.email}</FieldLabel>
              <Input
                type="email"
                value={profile.emailInput}
                onValueChange={profile.setEmailInput}
                placeholder={labels.emailPlaceholder}
                disabled={profile.emailSaving}
              />
              <FieldDescription>
                {profile.hasEmail ? profile.email : labels.emailNotSetHelp}
              </FieldDescription>
            </Field>
          </FieldGroup>

          <div className="mt-4 flex flex-wrap items-center gap-3">
            <Button
              type="button"
              variant="primary"
              onClick={() => void profile.saveEmail()}
              disabled={profile.emailSaving || !canSaveEmail}
            >
              {profile.emailSaving
                ? labels.savingEmail
                : profile.hasEmail
                  ? labels.updateEmail
                  : labels.setEmail}
            </Button>
            {profile.emailStatus ? (
              <span className="text-sm text-(--app-muted-text)">
                {profile.emailStatus}
              </span>
            ) : null}
          </div>
        </div>
      ) : null}
    </section>
  )
}
