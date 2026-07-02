import {
  getCurrentAppProfileApi,
  updateCurrentAppProfileApi,
  type CurrentAppProfileResponse,
} from "@/api"

export type AppProfile = CurrentAppProfileResponse

export async function loadAppProfile() {
  return getCurrentAppProfileApi({ suppressGlobalError: true })
}

export async function saveAppProfile(input: {
  schoolName: string
  teamName: string
}) {
  return updateCurrentAppProfileApi({
    schoolName: normalizeOptionalValue(input.schoolName),
    teamName: normalizeOptionalValue(input.teamName),
  })
}

function normalizeOptionalValue(value: string) {
  const normalized = value.trim()
  return normalized ? normalized : null
}
