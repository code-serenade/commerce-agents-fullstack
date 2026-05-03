import { useEffect, useMemo } from "react"
import { createAdminNotificationSocket } from "@workspace/services/realtime/admin-notification-socket"

interface UseAdminNotificationSocketOptions {
  token?: string | null
}

export function useAdminNotificationSocket({
  token,
}: UseAdminNotificationSocketOptions) {
  const wsBaseUrl = import.meta.env.VITE_API_PROXY || null

  const socket = useMemo(() => {
    if (!token) {
      return null
    }

    return createAdminNotificationSocket({
      token,
      baseHttpUrl: wsBaseUrl,
      onOpen() {
        console.info("[admin-ws] opened")
      },
      onConnected(message) {
        console.info("[admin-ws] connected", message)
      },
      onNotification(message) {
        console.info("[admin-ws] notification", message)
      },
      onErrorMessage(message) {
        console.warn("[admin-ws] server error", message)
      },
      onUnknownMessage(message) {
        console.debug("[admin-ws] unknown message", message)
      },
      onSocketError(event) {
        console.error("[admin-ws] socket error", event)
      },
      onClose(event) {
        console.info("[admin-ws] closed", {
          code: event.code,
          reason: event.reason,
        })
      },
    })
  }, [token, wsBaseUrl])

  useEffect(() => {
    if (!socket) {
      return
    }

    const timer = window.setTimeout(() => {
      socket.connect()
    }, 0)

    return () => {
      window.clearTimeout(timer)
      socket.disconnect()
    }
  }, [socket])
}
