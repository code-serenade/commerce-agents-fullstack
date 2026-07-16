import { QueryClient, QueryClientProvider } from "@tanstack/react-query"
import { render, screen } from "@testing-library/react"
import { beforeEach, describe, expect, it, vi } from "vitest"
import { AgentChatTestI18nProvider } from "@/test/AgentChatTestI18nProvider"
import { createAgentChatTestI18n } from "@/test/create-agent-chat-test-i18n"
import { file } from "../domain/test-helpers"
import { FilesImageThumbnail } from "./files-image-thumbnail"

const getConversationFileAccessApi = vi.hoisted(() => vi.fn())

vi.mock("@/api", () => ({ getConversationFileAccessApi }))

describe("FilesImageThumbnail", () => {
  beforeEach(() => {
    getConversationFileAccessApi.mockReset()
  })

  it("loads a signed URL into the compact ImageViewer thumbnail", async () => {
    const i18n = await createAgentChatTestI18n()
    const imageFile = file({
      fileName: "catalog.png",
      mimeType: "image/png",
    })
    getConversationFileAccessApi.mockResolvedValue({
      downloadUrl: "https://files.example/catalog.png?signature=test",
      expiresSecs: 900,
      file: imageFile,
      method: "GET",
    })
    const queryClient = new QueryClient({
      defaultOptions: { queries: { retry: false } },
    })

    render(
      <QueryClientProvider client={queryClient}>
        <AgentChatTestI18nProvider i18n={i18n}>
          <FilesImageThumbnail file={imageFile} />
        </AgentChatTestI18nProvider>
      </QueryClientProvider>
    )

    const preview = await screen.findByRole("button", {
      name: i18n.t("commerce.files.imageViewer.preview"),
    })

    expect(preview).toHaveClass("size-12")
    expect(screen.getByAltText("catalog.png")).toHaveAttribute(
      "src",
      "https://files.example/catalog.png?signature=test"
    )
    expect(getConversationFileAccessApi).toHaveBeenCalledWith(imageFile.id)
  })
})
