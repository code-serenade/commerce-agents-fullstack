import { createReactAppViteConfig } from "@workspace/vite-config/react-app"

export default createReactAppViteConfig({
  appDir: __dirname,
  // rewriteApiPath: (proxyPath) => proxyPath.replace(/^\/api/, ""),
})
