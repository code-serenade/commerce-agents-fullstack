import { dirname } from "node:path"
import { fileURLToPath } from "node:url"
import { createReactAppViteConfig } from "@workspace/vite-config/react-app"

const appDir = dirname(fileURLToPath(import.meta.url))

export default createReactAppViteConfig({
  appDir,
  // rewriteApiPath: (proxyPath) => proxyPath.replace(/^\/api/, ""),
})
