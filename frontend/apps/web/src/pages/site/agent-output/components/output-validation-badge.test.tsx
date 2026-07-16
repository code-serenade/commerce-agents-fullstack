import { render, screen } from "@testing-library/react"
import { describe, expect, it } from "vitest"
import { OutputValidationBadge } from "./output-validation-badge"

describe("OutputValidationBadge", () => {
  it.each([
    ["passed", "[&_[data-slot=badge]]:bg-emerald-50"],
    ["unchecked", "[&_[data-slot=badge]]:bg-amber-50"],
    ["failed", "[&_[data-slot=badge]]:bg-red-50"],
  ])("uses a distinct color for %s reports", (status, colorClass) => {
    const label = ["Status", status].join(" ")

    render(<OutputValidationBadge label={label} status={status} />)

    expect(screen.getByText(label).parentElement).toHaveClass(colorClass)
  })
})
