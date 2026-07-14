import { useState } from "react"
import { RectangleEllipsis } from "lucide-react"
import { Button } from "@workspace/ui-components/stable/button"
import {
  Field,
  FieldDescription,
  FieldGroup,
  FieldLabel,
} from "@workspace/ui-components/stable/field"
import { Input } from "@workspace/ui-components/stable/input"
import type { AuthProfileLabels, AuthProfileModel } from "./types"

export interface ProfilePasswordSectionProps {
  profile: AuthProfileModel
  labels: AuthProfileLabels
}

export function ProfilePasswordSection({
  profile,
  labels,
}: ProfilePasswordSectionProps) {
  const [expanded, setExpanded] = useState(false)
  const canChangePassword =
    profile.currentPassword.trim().length > 0 &&
    profile.newPassword.trim().length >= 8 &&
    profile.confirmPassword.trim().length > 0

  return (
    <section className="min-w-0 rounded-xl border border-(--document-card-border) bg-(image:--document-card-background) p-4 shadow-(--document-card-shadow) backdrop-blur-md">
      <div className="flex flex-col gap-4 sm:flex-row sm:items-start sm:justify-between">
        <div className="flex min-w-0 items-start gap-4">
          <span className="flex size-9 shrink-0 items-center justify-center rounded-lg bg-primary/10 text-primary">
            <RectangleEllipsis className="size-5" />
          </span>
          <div className="min-w-0 space-y-1">
            <h2 className="text-sm font-semibold">{labels.passwordSection}</h2>
            <p className="text-sm text-(--app-muted-text)">
              {labels.passwordConfigured}
            </p>
          </div>
        </div>

        <Button
          type="button"
          variant="default"
          className="self-start"
          onClick={() => setExpanded((current) => !current)}
        >
          {expanded ? labels.hidePassword : labels.changePassword}
        </Button>
      </div>

      {expanded ? (
        <div className="mt-6 max-w-xl sm:pl-13">
          <FieldGroup>
            <Field>
              <FieldLabel>{labels.currentPassword}</FieldLabel>
              <Input
                type="password"
                value={profile.currentPassword}
                onValueChange={profile.setCurrentPassword}
                placeholder={labels.currentPasswordPlaceholder}
                disabled={profile.passwordSaving}
              />
            </Field>

            <Field>
              <FieldLabel>{labels.newPassword}</FieldLabel>
              <Input
                type="password"
                value={profile.newPassword}
                onValueChange={profile.setNewPassword}
                placeholder={labels.newPasswordPlaceholder}
                disabled={profile.passwordSaving}
              />
            </Field>

            <Field>
              <FieldLabel>{labels.confirmPassword}</FieldLabel>
              <Input
                type="password"
                value={profile.confirmPassword}
                onValueChange={profile.setConfirmPassword}
                placeholder={labels.confirmPasswordPlaceholder}
                disabled={profile.passwordSaving}
              />
              <FieldDescription>{labels.newPasswordHelp}</FieldDescription>
            </Field>
          </FieldGroup>

          <div className="mt-4 flex flex-wrap items-center gap-3">
            <Button
              type="button"
              variant="primary"
              onClick={() => void profile.changePassword()}
              disabled={profile.passwordSaving || !canChangePassword}
            >
              {profile.passwordSaving
                ? labels.changingPassword
                : labels.updatePassword}
            </Button>
            {profile.passwordStatus ? (
              <span className="text-sm text-(--app-muted-text)">
                {profile.passwordStatus}
              </span>
            ) : null}
          </div>
        </div>
      ) : null}
    </section>
  )
}
