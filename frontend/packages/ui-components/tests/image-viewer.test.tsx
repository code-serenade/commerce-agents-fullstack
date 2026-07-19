// @vitest-environment jsdom

import { fireEvent, render, screen } from "@testing-library/react"
import { describe, expect, it, vi } from "vitest"
import { ImageViewer } from "../src/components/stable/image-viewer"

const images = [
  { src: "/image-one.png", alt: "First product" },
  { src: "/image-two.png", alt: "Second product" },
]

describe("ImageViewer", () => {
  it("opens from its thumbnail using the controlled callback", () => {
    const onOpenChange = vi.fn()

    render(
      <ImageViewer
        images={images}
        open={false}
        index={0}
        onOpenChange={onOpenChange}
        onIndexChange={vi.fn()}
      />
    )

    fireEvent.click(screen.getByRole("button", { name: "Preview image" }))
    expect(onOpenChange).toHaveBeenCalledWith(true)
  })

  it("supports a compact 48 pixel thumbnail", () => {
    render(
      <ImageViewer
        images={images}
        open={false}
        index={0}
        onOpenChange={vi.fn()}
        onIndexChange={vi.fn()}
        thumbnailSize="compact"
      />
    )

    expect(
      screen
        .getByRole("button", { name: "Preview image" })
        .classList.contains("size-12")
    ).toBe(true)
  })

  it("zooms, rotates, resets, and closes the open viewer", () => {
    const onOpenChange = vi.fn()

    render(
      <ImageViewer
        images={images}
        open
        index={0}
        onOpenChange={onOpenChange}
        onIndexChange={vi.fn()}
      />
    )

    fireEvent.click(screen.getByRole("button", { name: "Zoom in" }))
    expect(screen.getByText("125%")).toBeTruthy()

    fireEvent.click(screen.getByRole("button", { name: "Rotate image" }))
    expect(screen.getAllByAltText("First product")[1].getAttribute("style")).toContain("rotate(90deg)")

    fireEvent.click(screen.getByRole("button", { name: "Reset image" }))
    expect(screen.getByText("100%")).toBeTruthy()

    fireEvent.click(screen.getByRole("button", { name: "Close image viewer" }))
    expect(onOpenChange).toHaveBeenCalledWith(false)
  })

  it("cycles through images from the navigation controls", () => {
    const onIndexChange = vi.fn()

    render(
      <ImageViewer
        images={images}
        open
        index={0}
        onOpenChange={vi.fn()}
        onIndexChange={onIndexChange}
      />
    )

    fireEvent.click(screen.getByRole("button", { name: "Previous image" }))
    fireEvent.click(screen.getByRole("button", { name: "Next image" }))

    expect(onIndexChange).toHaveBeenNthCalledWith(1, 1)
    expect(onIndexChange).toHaveBeenNthCalledWith(2, 1)
  })
})
