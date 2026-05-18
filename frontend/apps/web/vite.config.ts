import { createReactAppViteConfig } from "@workspace/vite-config/react-app"

export default createReactAppViteConfig({
  appDir: __dirname,
  devOverlay: false,
  host: "127.0.0.1",
  port: 5173,
  strictPort: true,
})
