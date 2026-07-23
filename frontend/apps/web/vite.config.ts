import { dirname } from "node:path"
import { fileURLToPath } from "node:url"
import { createReactAppViteConfig } from "@workspace/vite-config/react-app"

const appDir = dirname(fileURLToPath(import.meta.url))

export default createReactAppViteConfig({
  appDir,
  devOverlay: false,
  host: "0.0.0.0",
  port: 5173,
  strictPort: true,
})
