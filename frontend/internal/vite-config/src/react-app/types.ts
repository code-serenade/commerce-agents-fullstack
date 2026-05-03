export interface CreateReactAppViteConfigOptions {
  appDir: string
  rewriteApiPath?: (proxyPath: string) => string
  testSetupFiles?: string[]
}

export interface CreateServerProxyOptions {
  rewriteApiPath?: (proxyPath: string) => string
}
