import { spawnSync } from "node:child_process"

const result = spawnSync("vitest", [
  "run",
  "--root",
  "..",
  "--testTimeout",
  "30000",
  "rules",
], {
  cwd: process.cwd(),
  env: {
    ...process.env,
    APP_ROOT: process.cwd(),
  },
  shell: process.platform === "win32",
  stdio: "inherit",
})

if (result.error) {
  console.error(result.error)
  process.exit(1)
}

process.exit(result.status ?? 1)
