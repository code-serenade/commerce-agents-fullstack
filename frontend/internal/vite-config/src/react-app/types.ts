export interface CreateReactAppViteConfigOptions {
  appDir: string
  devOverlay?: boolean
  host?: string
  port?: number
  strictPort?: boolean
  rewriteApiPath?: (proxyPath: string) => string
  testSetupFiles?: string[]
}

export interface CreateServerProxyOptions {
  rewriteApiPath?: (proxyPath: string) => string
}
