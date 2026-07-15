import { AvatarPhotoEditor } from "../../components/file-upload"
import type { AuthProfileLabels, AuthProfileModel } from "./types"

export interface ProfileAvatarSectionProps {
  profile: AuthProfileModel
  labels: AuthProfileLabels
}

export function ProfileAvatarSection({
  profile,
  labels,
}: ProfileAvatarSectionProps) {
  return (
    <aside className="flex min-w-0 rounded-xl border border-(--document-card-border) bg-(image:--document-card-background) p-4 shadow-(--document-card-shadow) backdrop-blur-md lg:h-full">
      <div className="space-y-4">
        <h2 className="text-sm font-semibold">{labels.avatarTitle}</h2>
        <AvatarPhotoEditor
          value={profile.userAvatar}
          alt={profile.displayName}
          fallback={profile.avatarFallback}
          disabled={profile.saving}
          labels={{
            edit: labels.avatarEdit,
            upload: labels.avatarUpload,
            uploading: labels.avatarUploading,
            remove: labels.avatarRemove,
            removeConfirmTitle: labels.avatarRemoveConfirmTitle,
            removeConfirmDescription: labels.avatarRemoveConfirmDescription,
            removeConfirmCancel: labels.avatarRemoveConfirmCancel,
            removeConfirmAction: labels.avatarRemoveConfirmAction,
          }}
          onUploaded={profile.uploadAvatar}
          onRemove={profile.removeAvatar}
        />
      </div>
    </aside>
  )
}
