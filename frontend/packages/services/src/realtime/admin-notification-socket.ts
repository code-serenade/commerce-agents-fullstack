import {
  buildWebSocketUrl,
  createWebSocketClient,
  type WebSocketClientHandle,
} from "./ws-client"
import type {
  AdminWsServerMessage,
  WsConnectedMessage,
  WsErrorMessage,
  WsNotificationMessage,
  WsPongMessage,
} from "./ws-types"

export interface AdminNotificationSocketOptions {
  token: string
  baseHttpUrl?: string | null
  path?: string
  pingIntervalMs?: number
  reconnect?: boolean
  reconnectInitialDelayMs?: number
  reconnectMaxDelayMs?: number
  onConnected?: (message: AdminWsServerMessage) => void
  onPong?: (message: WsPongMessage) => void
  onNotification?: (message: WsNotificationMessage) => void
  onErrorMessage?: (message: AdminWsServerMessage) => void
  onUnknownMessage?: (message: AdminWsServerMessage) => void
  onOpen?: () => void
  onClose?: (event: CloseEvent) => void
  onSocketError?: (event: Event) => void
}

export interface AdminNotificationSocketHandle {
  connect: () => WebSocket
  disconnect: () => void
  sendPing: () => boolean
  getSocket: WebSocketClientHandle["getSocket"]
}

export function createAdminNotificationSocket({
  token,
  baseHttpUrl,
  path = "/api/ws",
  pingIntervalMs = 30_000,
  reconnect = true,
  reconnectInitialDelayMs = 1_000,
  reconnectMaxDelayMs = 30_000,
  onConnected,
  onPong,
  onNotification,
  onErrorMessage,
  onUnknownMessage,
  onOpen,
  onClose,
  onSocketError,
}: AdminNotificationSocketOptions): AdminNotificationSocketHandle {
  let pingTimer: ReturnType<typeof globalThis.setInterval> | null = null
  let reconnectTimer: ReturnType<typeof globalThis.setTimeout> | null = null
  let reconnectAttempt = 0
  let manuallyClosed = false

  const clearPingTimer = () => {
    if (pingTimer == null) {
      return
    }

    globalThis.clearInterval(pingTimer)
    pingTimer = null
  }

  const clearReconnectTimer = () => {
    if (reconnectTimer == null) {
      return
    }

    globalThis.clearTimeout(reconnectTimer)
    reconnectTimer = null
  }

  const startPingTimer = (socket: WebSocket) => {
    clearPingTimer()

    pingTimer = globalThis.setInterval(() => {
      if (socket.readyState !== WebSocket.OPEN) {
        clearPingTimer()
        return
      }

      socket.send(JSON.stringify({ type: "ping" }))
    }, pingIntervalMs)
  }

  const scheduleReconnect = () => {
    if (!reconnect || manuallyClosed || reconnectTimer != null) {
      return
    }

    const delay = Math.min(
      reconnectInitialDelayMs * 2 ** reconnectAttempt,
      reconnectMaxDelayMs
    )

    reconnectAttempt += 1
    reconnectTimer = globalThis.setTimeout(() => {
      reconnectTimer = null

      if (manuallyClosed) {
        return
      }

      try {
        client.connect()
      } catch {
        scheduleReconnect()
      }
    }, delay)
  }

  const client = createWebSocketClient({
    url: buildWebSocketUrl({
      path,
      token,
      baseHttpUrl,
    }),
    onOpen(event, socket) {
      reconnectAttempt = 0
      clearReconnectTimer()
      onOpen?.()
      socket.send(JSON.stringify({ type: "ping" }))
      startPingTimer(socket)
      void event
    },
    onMessage(event) {
      const message = parseAdminWsMessage(event.data)

      if (!message) {
        return
      }

      switch (message.type) {
        case "connected":
          onConnected?.(message as WsConnectedMessage)
          return
        case "pong":
          onPong?.(message as WsPongMessage)
          return
        case "notification":
          onNotification?.(message as WsNotificationMessage)
          return
        case "error":
          onErrorMessage?.(message as WsErrorMessage)
          return
        default:
          onUnknownMessage?.(message)
      }
    },
    onError(event) {
      onSocketError?.(event)
    },
    onClose(event) {
      clearPingTimer()
      onClose?.(event)
      scheduleReconnect()
    },
  })

  return {
    connect() {
      manuallyClosed = false
      clearReconnectTimer()
      return client.connect()
    },
    disconnect() {
      manuallyClosed = true
      clearPingTimer()
      clearReconnectTimer()
      client.disconnect(1000, "admin shell cleanup")
    },
    sendPing() {
      return client.sendText(JSON.stringify({ type: "ping" }))
    },
    getSocket: client.getSocket,
  }
}

function parseAdminWsMessage(raw: string): AdminWsServerMessage | null {
  try {
    return JSON.parse(raw) as AdminWsServerMessage
  } catch {
    return null
  }
}
