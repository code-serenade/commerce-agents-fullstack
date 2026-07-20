import { spawn } from "node:child_process"
import { createRequire } from "node:module"

const rawArgs = process.argv.slice(2)

const targetArgs = []
const passthroughArgs = []

for (const arg of rawArgs) {
  if (arg.startsWith("-")) {
    passthroughArgs.push(arg)
  } else {
    targetArgs.push(`--filter=${arg}`)
  }
}

const turboArgs = ["dev", ...targetArgs, ...passthroughArgs]
const require = createRequire(import.meta.url)
const turboBin = require.resolve("turbo/bin/turbo")

const child = spawn(process.execPath, [turboBin, ...turboArgs], {
  stdio: "inherit",
})

child.on("exit", (code, signal) => {
  if (signal) {
    process.kill(process.pid, signal)
    return
  }
  process.exit(code ?? 0)
})
