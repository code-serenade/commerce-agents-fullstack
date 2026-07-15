import { useCallback, useEffect, useState } from "react"
import { useTranslation } from "react-i18next"
import { useQueryClient } from "@tanstack/react-query"
import { toRequestError } from "@workspace/services/errors/request-error"
import { currentTeamUsageQuotaQueryKey } from "@/api"
import { loadAppProfile, saveAppProfile, type AppProfile } from "./profile-data"

export function useProfilePage() {
  const { t } = useTranslation()
  const queryClient = useQueryClient()
  const [profile, setProfile] = useState<AppProfile | null>(null)
  const [schoolName, setSchoolName] = useState("")
  const [teamName, setTeamName] = useState("")
  const [isLoading, setIsLoading] = useState(true)
  const [isSaving, setIsSaving] = useState(false)
  const [loadError, setLoadError] = useState("")
  const [saveStatus, setSaveStatus] = useState("")

  useEffect(() => {
    let active = true

    async function load() {
      setIsLoading(true)
      setLoadError("")
      try {
        const nextProfile = await loadAppProfile()
        if (!active) return
        setProfile(nextProfile)
        setSchoolName(nextProfile.schoolName ?? "")
        setTeamName(nextProfile.teamName ?? "")
      } catch (error) {
        if (active) {
          setLoadError(
            resolveError(error, {
              fallback: t("profile.appProfile.loadFailed"),
              missingAppUser: t("profile.appProfile.missingAppUser"),
            })
          )
        }
      } finally {
        if (active) {
          setIsLoading(false)
        }
      }
    }

    void load()

    return () => {
      active = false
    }
  }, [t])

  const save = useCallback(async () => {
    setIsSaving(true)
    setSaveStatus("")
    try {
      const nextProfile = await saveAppProfile({ schoolName, teamName })
      setProfile(nextProfile)
      setSchoolName(nextProfile.schoolName ?? "")
      setTeamName(nextProfile.teamName ?? "")
      await queryClient.invalidateQueries({
        queryKey: currentTeamUsageQuotaQueryKey,
      })
      setSaveStatus(t("profile.appProfile.saved"))
    } catch (error) {
      setSaveStatus(
        resolveError(error, {
          fallback: t("profile.appProfile.saveFailed"),
          missingAppUser: t("profile.appProfile.missingAppUser"),
        })
      )
    } finally {
      setIsSaving(false)
    }
  }, [queryClient, schoolName, t, teamName])

  return {
    appProfile: {
      profile,
      schoolName,
      teamName,
      isLoading,
      isSaving,
      loadError,
      saveStatus,
      setSchoolName,
      setTeamName,
      save,
    },
  }
}

function resolveError(
  error: unknown,
  messages: { fallback: string; missingAppUser: string }
) {
  if (isMissingCurrentAppUserError(error)) {
    return messages.missingAppUser
  }
  return error instanceof Error ? error.message : messages.fallback
}

function isMissingCurrentAppUserError(error: unknown) {
  const requestError = toRequestError(error)

  return (
    requestError.code === -12000 ||
    requestError.status === 404 ||
    requestError.code === -10002
  )
}
