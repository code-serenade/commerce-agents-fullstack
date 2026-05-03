import { spawn } from "node:child_process"

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

const child = spawn("turbo", turboArgs, {
  stdio: "inherit",
  shell: process.platform === "win32",
})

child.on("exit", (code, signal) => {
  if (signal) {
    process.kill(process.pid, signal)
    return
  }
  process.exit(code ?? 0)
})
