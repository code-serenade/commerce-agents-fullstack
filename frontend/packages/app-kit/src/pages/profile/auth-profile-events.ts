import type { UserInfo } from "@workspace/services/api/auth"

export const authProfileUserUpdatedEvent = "workspace:auth-profile-user-updated"

export function dispatchAuthProfileUserUpdated(user: UserInfo) {
  window.dispatchEvent(
    new CustomEvent<UserInfo>(authProfileUserUpdatedEvent, {
      detail: user,
    })
  )
}

export function addAuthProfileUserUpdatedListener(
  listener: (user: UserInfo) => void
) {
  const handleUserUpdated = (event: Event) => {
    listener((event as CustomEvent<UserInfo>).detail)
  }

  window.addEventListener(authProfileUserUpdatedEvent, handleUserUpdated)

  return () => {
    window.removeEventListener(authProfileUserUpdatedEvent, handleUserUpdated)
  }
}
