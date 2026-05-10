import "@testing-library/jest-dom/vitest"
import { afterEach } from "vitest"

const memoryStorage = (() => {
  const store = new Map<string, string>()

  return {
    getItem(key: string) {
      return store.has(key) ? store.get(key)! : null
    },
    setItem(key: string, value: string) {
      store.set(key, value)
    },
    removeItem(key: string) {
      store.delete(key)
    },
    clear() {
      store.clear()
    },
  }
})()

Object.defineProperty(globalThis, "localStorage", {
  value: memoryStorage,
  configurable: true,
})

afterEach(() => {
  globalThis.localStorage.clear()
})
