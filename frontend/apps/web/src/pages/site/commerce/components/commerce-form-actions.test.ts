import { describe, expect, it, vi } from "vitest"
import { buildSimpleFormAuxiliaryActions } from "./commerce-form-actions"
import type { SimpleFormValues } from "./commerce-form"

describe("buildSimpleFormAuxiliaryActions", () => {
  it("builds sample, template draft, and agent draft actions in order", () => {
    const agentLabel = "Agent fill"
    const sampleLabel = "Fill sample"
    const templateLabel = "Template fill"
    const actions = buildSimpleFormAuxiliaryActions({
      agentSmartFillAction: {
        label: agentLabel,
        onClick: vi.fn(),
      },
      sampleAction: {
        label: sampleLabel,
        values: { name: "Sample product" },
      },
      setValues: vi.fn(),
      smartFillAction: {
        label: templateLabel,
        onClick: vi.fn(),
      },
      values: { name: "" },
    })

    expect(actions.map((action) => action.label)).toEqual([
      sampleLabel,
      templateLabel,
      agentLabel,
    ])
  })

  it("merges draft values back into the current form values", async () => {
    const agentLabel = "Agent fill"
    const draftResults: unknown[] = []
    const nextValues: SimpleFormValues[] = []
    const actions = buildSimpleFormAuxiliaryActions({
      agentSmartFillAction: {
        disabled: true,
        label: agentLabel,
        onClick: async () => ({
          agentContext: { generationSource: "agent_llm" },
          generationMode: "agent_llm",
          values: { brand: "Agent brand", name: "Agent draft" },
        }),
      },
      onDraftResult: (result) => draftResults.push(result),
      setValues: (values) => nextValues.push(values),
      values: { currency: "CNY", name: "Original name" },
    })

    expect(actions[0]?.disabled).toBe(true)
    await actions[0]?.onClick()

    expect(nextValues).toEqual([
      {
        brand: "Agent brand",
        currency: "CNY",
        name: "Agent draft",
      },
    ])
    expect(draftResults).toEqual([
      {
        agentContext: { generationSource: "agent_llm" },
        generationMode: "agent_llm",
        values: { brand: "Agent brand", name: "Agent draft" },
      },
    ])
  })

  it("clears the previous draft process when a sample is applied", async () => {
    const draftResults: unknown[] = []
    const sampleLabel = "Fill sample"
    const actions = buildSimpleFormAuxiliaryActions({
      onDraftResult: (result) => draftResults.push(result),
      sampleAction: {
        label: sampleLabel,
        values: { name: "Sample product" },
      },
      setValues: vi.fn(),
      values: { name: "" },
    })

    await actions[0]?.onClick()

    expect(draftResults).toEqual([null])
  })
})
