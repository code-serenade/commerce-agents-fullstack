import { useTranslation } from "react-i18next"
import type { AuthProfileLabels } from "./types"

export function useAuthProfileLabels(): AuthProfileLabels {
  const { t } = useTranslation("pages")

  return {
    title: t("profile.title"),
    accountSection: t("profile.sections.account"),
    id: t("profile.fields.id"),
    name: t("profile.fields.name"),
    namePlaceholder: t("profile.placeholders.name"),
    nameHelp: t("profile.help.name"),
    saveProfile: t("profile.actions.save"),
    savingProfile: t("profile.actions.saving"),
    avatarTitle: t("profile.avatar.title"),
    avatarEdit: t("profile.avatar.edit"),
    avatarUpload: t("profile.avatar.upload"),
    avatarUploading: t("profile.avatar.uploading"),
    avatarRemove: t("profile.avatar.remove"),
    avatarRemoveConfirmTitle: t("profile.avatar.removeConfirm.title"),
    avatarRemoveConfirmDescription: t(
      "profile.avatar.removeConfirm.description"
    ),
    avatarRemoveConfirmCancel: t("profile.avatar.removeConfirm.cancel"),
    avatarRemoveConfirmAction: t("profile.avatar.removeConfirm.action"),
    emailSection: t("profile.sections.email"),
    email: t("profile.fields.email"),
    emailNotSet: t("profile.email.notSet"),
    emailPlaceholder: t("profile.placeholders.email"),
    emailNotSetHelp: t("profile.help.emailNotSet"),
    setEmail: t("profile.actions.setEmail"),
    changeEmail: t("profile.actions.changeEmail"),
    hideEmail: t("profile.actions.hideEmail"),
    updateEmail: t("profile.actions.updateEmail"),
    savingEmail: t("profile.actions.savingEmail"),
    passwordSection: t("profile.sections.security"),
    passwordConfigured: t("profile.password.configured"),
    currentPassword: t("profile.fields.currentPassword"),
    newPassword: t("profile.fields.newPassword"),
    confirmPassword: t("profile.fields.confirmPassword"),
    currentPasswordPlaceholder: t("profile.placeholders.currentPassword"),
    newPasswordPlaceholder: t("profile.placeholders.newPassword"),
    confirmPasswordPlaceholder: t("profile.placeholders.confirmPassword"),
    newPasswordHelp: t("profile.help.newPassword"),
    changePassword: t("profile.actions.changePassword"),
    hidePassword: t("profile.actions.hidePassword"),
    updatePassword: t("profile.actions.updatePassword"),
    changingPassword: t("profile.actions.changingPassword"),
  }
}
