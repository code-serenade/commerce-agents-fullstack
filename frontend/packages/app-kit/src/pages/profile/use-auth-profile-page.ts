import { useCallback, useEffect, useState } from "react"
import { useTranslation } from "react-i18next"
import {
  meApi,
  updateEmailApi,
  updatePasswordApi,
  updateProfileApi,
  type UserInfo,
} from "@workspace/services/api/auth"
import type { AvatarUploadResult } from "../../components/file-upload"
import { dispatchAuthProfileUserUpdated } from "./auth-profile-events"
import type { AuthProfileModel } from "./types"

function getInitial(name: string) {
  return name.trim().charAt(0).toUpperCase() || "U"
}

export interface UseAuthProfilePageOptions {
  initialUser?: UserInfo | null
  onUserChange?: (user: UserInfo) => void
}

export function useAuthProfilePage({
  initialUser = null,
  onUserChange,
}: UseAuthProfilePageOptions = {}) {
  const { t } = useTranslation("pages")
  const [user, setUser] = useState<UserInfo | null>(initialUser)
  const [loading, setLoading] = useState(!initialUser)
  const [loadError, setLoadError] = useState("")
  const [name, setName] = useState(initialUser?.name ?? "")
  const [emailInput, setEmailInput] = useState(initialUser?.email ?? "")
  const [currentPassword, setCurrentPassword] = useState("")
  const [newPassword, setNewPassword] = useState("")
  const [confirmPassword, setConfirmPassword] = useState("")
  const [saving, setSaving] = useState(false)
  const [emailSaving, setEmailSaving] = useState(false)
  const [passwordSaving, setPasswordSaving] = useState(false)
  const [status, setStatus] = useState("")
  const [emailStatus, setEmailStatus] = useState("")
  const [passwordStatus, setPasswordStatus] = useState("")

  const applyUser = useCallback(
    (nextUser: UserInfo) => {
      setUser(nextUser)
      onUserChange?.(nextUser)
      dispatchAuthProfileUserUpdated(nextUser)
    },
    [onUserChange]
  )

  useEffect(() => {
    if (!initialUser) return
    setUser(initialUser)
  }, [initialUser])

  useEffect(() => {
    let active = true

    async function loadProfile() {
      setLoading(true)
      setLoadError("")
      try {
        const nextUser = await meApi()
        if (active) {
          applyUser(nextUser)
        }
      } catch (error) {
        if (active) {
          setLoadError(resolveError(error, t("profile.status.failed")))
        }
      } finally {
        if (active) {
          setLoading(false)
        }
      }
    }

    void loadProfile()

    return () => {
      active = false
    }
  }, [applyUser, t])

  const displayName = user?.name || t("profile.fallback.name")
  const displayId = user?.display_id || user?.id || t("profile.fallback.id")
  const email = user?.email ?? ""
  const displayEmail = email || t("profile.fallback.email")
  const hasEmail = Boolean(email)

  useEffect(() => {
    setName(displayName)
  }, [displayName])

  useEffect(() => {
    setEmailInput(email)
  }, [email])

  const refreshMe = async () => {
    const nextUser = await meApi()
    applyUser(nextUser)
  }

  const resolveProfileError = (error: unknown) =>
    resolveError(error, t("profile.status.failed"))

  const saveProfile = async () => {
    setSaving(true)
    setStatus("")
    try {
      await updateProfileApi({ displayName: name.trim() || null })
      await refreshMe()
      setStatus(t("profile.status.saved"))
    } catch (error) {
      setStatus(resolveProfileError(error))
    } finally {
      setSaving(false)
    }
  }

  const saveEmail = async () => {
    setEmailSaving(true)
    setEmailStatus("")
    try {
      await updateEmailApi({ email: emailInput.trim() || null })
      await refreshMe()
      setEmailStatus(t("profile.status.emailSaved"))
    } catch (error) {
      setEmailStatus(resolveProfileError(error))
    } finally {
      setEmailSaving(false)
    }
  }

  const changePassword = async () => {
    setPasswordSaving(true)
    setPasswordStatus("")
    try {
      if (newPassword !== confirmPassword) {
        setPasswordStatus(t("profile.status.passwordMismatch"))
        return
      }

      await updatePasswordApi({
        oldPassword: currentPassword,
        newPassword,
      })
      setCurrentPassword("")
      setNewPassword("")
      setConfirmPassword("")
      setPasswordStatus(t("profile.status.passwordChanged"))
    } catch (error) {
      setPasswordStatus(resolveProfileError(error))
    } finally {
      setPasswordSaving(false)
    }
  }

  const uploadAvatar = async (result: AvatarUploadResult) => {
    setSaving(true)
    setStatus("")
    try {
      await updateProfileApi({ avatar: result.key })
      await refreshMe()
      setStatus(t("profile.status.avatarSaved"))
    } catch (error) {
      setStatus(resolveProfileError(error))
      throw error
    } finally {
      setSaving(false)
    }
  }

  const removeAvatar = async () => {
    setSaving(true)
    setStatus("")
    try {
      await updateProfileApi({ avatar: null })
      await refreshMe()
      setStatus(t("profile.status.avatarRemoved"))
    } catch (error) {
      setStatus(resolveProfileError(error))
    } finally {
      setSaving(false)
    }
  }

  const profile: AuthProfileModel = {
    userAvatar: user?.avatar,
    displayName,
    displayId,
    email: displayEmail,
    hasEmail,
    avatarFallback: getInitial(displayName),
    name,
    emailInput,
    currentPassword,
    newPassword,
    confirmPassword,
    saving,
    emailSaving,
    passwordSaving,
    status,
    emailStatus,
    passwordStatus,
    setName,
    setEmailInput,
    setCurrentPassword,
    setNewPassword,
    setConfirmPassword,
    saveProfile,
    saveEmail,
    changePassword,
    uploadAvatar,
    removeAvatar,
  }

  return {
    profile,
    loading,
    loadError,
  }
}

function resolveError(error: unknown, fallback: string) {
  return error instanceof Error ? error.message : fallback
}
