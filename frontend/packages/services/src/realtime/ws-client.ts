export interface WebSocketUrlOptions {
  path: string
  token?: string | null
  baseHttpUrl?: string | null
}

export interface WebSocketClientOptions {
  url: string
  protocols?: string | string[]
  onOpen?: (event: Event, socket: WebSocket) => void
  onMessage?: (event: MessageEvent<string>, socket: WebSocket) => void
  onError?: (event: Event, socket: WebSocket) => void
  onClose?: (event: CloseEvent) => void
}

export interface WebSocketClientHandle {
  connect: () => WebSocket
  disconnect: (code?: number, reason?: string) => void
  sendText: (message: string) => boolean
  getSocket: () => WebSocket | null
}

export function buildWebSocketUrl({
  path,
  token,
  baseHttpUrl,
}: WebSocketUrlOptions): string {
  const origin = resolveWebSocketOrigin(baseHttpUrl)
  const url = new URL(path, origin)

  if (token) {
    url.searchParams.set("token", token)
  }

  return url.toString()
}

export function createWebSocketClient({
  url,
  protocols,
  onOpen,
  onMessage,
  onError,
  onClose,
}: WebSocketClientOptions): WebSocketClientHandle {
  let socket: WebSocket | null = null

  return {
    connect() {
      if (
        socket &&
        (socket.readyState === WebSocket.OPEN ||
          socket.readyState === WebSocket.CONNECTING)
      ) {
        return socket
      }

      socket = protocols ? new WebSocket(url, protocols) : new WebSocket(url)

      socket.addEventListener("open", (event) => {
        if (!socket) return
        onOpen?.(event, socket)
      })

      socket.addEventListener("message", (event) => {
        if (!socket || typeof event.data !== "string") return
        onMessage?.(event as MessageEvent<string>, socket)
      })

      socket.addEventListener("error", (event) => {
        if (!socket) return
        onError?.(event, socket)
      })

      socket.addEventListener("close", (event) => {
        socket = null
        onClose?.(event)
      })

      return socket
    },

    disconnect(code, reason) {
      socket?.close(code, reason)
      socket = null
    },

    sendText(message) {
      if (!socket || socket.readyState !== WebSocket.OPEN) {
        return false
      }

      socket.send(message)
      return true
    },

    getSocket() {
      return socket
    },
  }
}

function resolveWebSocketOrigin(baseHttpUrl?: string | null): string {
  if (baseHttpUrl) {
    return httpToWebSocketOrigin(baseHttpUrl)
  }

  if (typeof window !== "undefined") {
    return httpToWebSocketOrigin(window.location.origin)
  }

  throw new Error("Unable to resolve WebSocket origin")
}

function httpToWebSocketOrigin(value: string): string {
  const url = new URL(value)
  url.protocol = url.protocol === "https:" ? "wss:" : "ws:"
  return url.toString()
}
