import type { ReactNode } from "react"
import type { UserInfo } from "@workspace/services/api/auth"
import type { AvatarUploadResult } from "../../components/file-upload"

export interface AuthProfileModel {
  userAvatar?: string
  displayName: string
  displayId: string
  email: string
  hasEmail: boolean
  avatarFallback: string
  name: string
  emailInput: string
  currentPassword: string
  newPassword: string
  confirmPassword: string
  saving: boolean
  emailSaving: boolean
  passwordSaving: boolean
  status: string
  emailStatus: string
  passwordStatus: string
  setName: (value: string) => void
  setEmailInput: (value: string) => void
  setCurrentPassword: (value: string) => void
  setNewPassword: (value: string) => void
  setConfirmPassword: (value: string) => void
  saveProfile: () => Promise<void>
  saveEmail: () => Promise<void>
  changePassword: () => Promise<void>
  uploadAvatar: (result: AvatarUploadResult) => Promise<void>
  removeAvatar: () => Promise<void>
}

export interface AuthProfileLabels {
  title: ReactNode
  accountSection: ReactNode
  id: ReactNode
  name: ReactNode
  namePlaceholder?: string
  nameHelp: ReactNode
  saveProfile: ReactNode
  savingProfile: ReactNode
  avatarTitle: ReactNode
  avatarEdit: ReactNode
  avatarUpload: ReactNode
  avatarUploading: ReactNode
  avatarRemove: ReactNode
  avatarRemoveConfirmTitle: ReactNode
  avatarRemoveConfirmDescription: ReactNode
  avatarRemoveConfirmCancel: ReactNode
  avatarRemoveConfirmAction: ReactNode
  emailSection: ReactNode
  email: ReactNode
  emailNotSet: ReactNode
  emailPlaceholder?: string
  emailNotSetHelp: ReactNode
  setEmail: ReactNode
  changeEmail: ReactNode
  hideEmail: ReactNode
  updateEmail: ReactNode
  savingEmail: ReactNode
  passwordSection: ReactNode
  passwordConfigured: ReactNode
  currentPassword: ReactNode
  newPassword: ReactNode
  confirmPassword: ReactNode
  currentPasswordPlaceholder?: string
  newPasswordPlaceholder?: string
  confirmPasswordPlaceholder?: string
  newPasswordHelp: ReactNode
  changePassword: ReactNode
  hidePassword: ReactNode
  updatePassword: ReactNode
  changingPassword: ReactNode
}

export interface AuthProfilePageProps {
  initialUser?: UserInfo | null
  onUserChange?: (user: UserInfo) => void
}

export interface AuthProfileViewProps {
  profile: AuthProfileModel
  labels: AuthProfileLabels
}
