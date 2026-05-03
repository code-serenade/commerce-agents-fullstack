import type { CreateServerProxyOptions } from "./types.ts"

export function createServerProxy(
  env: Record<string, string>,
  { rewriteApiPath }: CreateServerProxyOptions
) {
  return {
    "/api": {
      target: env.VITE_API_PROXY,
      changeOrigin: true,
      ws: false,
      ...(rewriteApiPath ? { rewrite: rewriteApiPath } : {}),
    },
    "/auth": {
      target: env.VITE_AUTH_PROXY,
      changeOrigin: true,
      ws: false,
    },
    "/file": {
      target: env.VITE_FILE_PROXY,
      changeOrigin: true,
      ws: false,
    },
  }
}
