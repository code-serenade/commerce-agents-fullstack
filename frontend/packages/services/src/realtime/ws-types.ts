export interface WsConnectedMessage {
  type: "connected"
  user_id?: string
  payload?: unknown
}

export interface WsPongMessage {
  type: "pong"
  payload?: unknown
}

export interface WsNotificationPayload {
  title: string
  content: string
  level: "info" | "success" | "warning" | "error"
  data?: Record<string, unknown> | null
}

export interface WsNotificationMessage {
  type: "notification"
  payload?: WsNotificationPayload
}

export interface WsErrorMessage {
  type: "error"
  message?: string
}

export interface WsUnknownMessage {
  type: string
  user_id?: string
  payload?: unknown
  message?: string
}

export type AdminWsServerMessage =
  | WsConnectedMessage
  | WsPongMessage
  | WsNotificationMessage
  | WsErrorMessage
  | WsUnknownMessage

export interface WsPingMessage {
  type: "ping"
}
