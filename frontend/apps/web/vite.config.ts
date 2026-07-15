import { createReactAppViteConfig } from "@workspace/vite-config/react-app"

export default createReactAppViteConfig({
  appDir: __dirname,
  devOverlay: false,
  host: "0.0.0.0",
  port: 5173,
  strictPort: true,
})
