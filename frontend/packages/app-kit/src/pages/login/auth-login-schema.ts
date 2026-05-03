import { z } from "zod"

type Translate = (key: string, options?: Record<string, unknown>) => string

export function createAuthLoginSchema(t: Translate) {
  return z.object({
    identifier: z
      .string()
      .trim()
      .min(1, t("validation.required", { ns: "common" })),
    password: z
      .string()
      .trim()
      .min(1, t("validation.required", { ns: "common" })),
  })
}
