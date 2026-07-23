import { dirname } from "node:path"
import { fileURLToPath } from "node:url"
import { createShowcaseViteConfig } from "@workspace/vite-config/showcase"

const appDir = dirname(fileURLToPath(import.meta.url))

export default createShowcaseViteConfig({
  appDir,
})
