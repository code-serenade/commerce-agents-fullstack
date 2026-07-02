import { describe, expect, it } from "vitest"
import { normalizeRuntimeOptions } from "./agent-runtime-option-normalization"

describe("normalizeRuntimeOptions", () => {
  it("folds legacy platform-specific ads roles into the generic ops role", () => {
    expect(
      normalizeRuntimeOptions({
        collaborationModes: [],
        learningLevels: [],
        responseModes: [],
        roles: [
          {
            id: "role-ops",
            name: "Ops",
            runtimeRole: "ops",
            capabilities: [],
            isDefault: true,
          },
          {
            id: "role-douyin-ads",
            name: "Legacy platform ads role",
            runtimeRole: "douyin_ads",
            capabilities: [],
            isDefault: false,
          },
        ],
        strategies: [],
      })?.roles
    ).toEqual([
      {
        id: "role-ops",
        name: "Ops",
        runtimeRole: "ops",
        capabilities: [],
        isDefault: true,
      },
    ])
  })
})
