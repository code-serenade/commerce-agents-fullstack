export type BaseMode = "styled" | "primitive"

export const DEFAULT_MODE: BaseMode = "styled"

export type WithBaseMode = {
  mode?: BaseMode
}

export function isPrimitiveMode(mode: BaseMode | undefined): boolean {
  return mode === "primitive"
}
