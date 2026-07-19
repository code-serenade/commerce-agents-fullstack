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

  it("can render a quieter desktop treatment for report lists", () => {
    const label = ["Status", "passed"].join(" ")

    render(
      <OutputValidationBadge label={label} quietOnDesktop status="passed" />
    )

    expect(screen.getByText(label).parentElement).toHaveClass(
      "md:**:data-[slot=badge]:bg-transparent"
    )
    expect(screen.getByText(label).parentElement).toHaveClass(
      "md:**:data-[slot=badge]:text-muted-foreground/80"
    )
    expect(screen.getByText(label).parentElement).toHaveClass(
      "md:before:bg-emerald-500/60"
    )
  })
})
