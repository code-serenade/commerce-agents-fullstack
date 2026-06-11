import { describe, expect, it } from "vitest"
import { createAgentChatTestI18n } from "@/test/create-agent-chat-test-i18n"
import { buildFormDraftProcessNotice } from "./commerce-form-draft-process"

const modelKey = "model"
const promptKey = "promptKey"
const baseUrlKey = "baseUrl"

describe("buildFormDraftProcessNotice", () => {
  it("summarizes the safe agent draft process without exposing internals", async () => {
    const i18n = await createAgentChatTestI18n()
    const notice = buildFormDraftProcessNotice(
      {
        agentContext: {
          [baseUrlKey]: "http://127.0.0.1:11434",
          draftQuality: {
            invalidModelJsonPolicy: "reject_before_form_fill",
          },
          generationSource: "agent_llm",
          llm: {
            analysisKind: "form_draft",
            route: "web_server_agent_manager",
            systemAgentId: "form_draft_generator",
            used: true,
            [modelKey]: "internal-model-name",
          },
          [promptKey]: "zh/form_drafts/product_form_draft.md",
          saveBoundary: {
            requiresUserReview: true,
          },
        },
        generationMode: "agent_llm",
      },
      i18n.t
    )

    expect(notice).toContain(i18n.t("commerce.common.smartFillProcess.agentRoute"))
    expect(notice).toContain(i18n.t("commerce.common.smartFillProcess.reviewRequired"))
    expect(notice).toContain(i18n.t("commerce.common.smartFillProcess.invalidJsonRejected"))
    expect(notice).not.toContain("internal-model-name")
    expect(notice).not.toContain("127.0.0.1")
    expect(notice).not.toContain("product_form_draft")
  })

  it("summarizes the deterministic template process", async () => {
    const i18n = await createAgentChatTestI18n()

    expect(
      buildFormDraftProcessNotice(
        {
          agentContext: {
            generationSource: "deterministic_template",
            requiresUserReview: true,
          },
        },
        i18n.t
      )
    ).toContain(i18n.t("commerce.common.smartFillProcess.templateRoute"))
  })

  it("adds safe stream stages without exposing model details", async () => {
    const i18n = await createAgentChatTestI18n()
    const notice = buildFormDraftProcessNotice(
      {
        agentContext: {
          [baseUrlKey]: "http://127.0.0.1:11434",
          generationSource: "agent_llm",
          llm: {
            used: true,
            [modelKey]: "hidden-model",
          },
          [promptKey]: "zh/form_drafts/product_form_draft.md",
        },
        stages: [
          "form_draft_started",
          "form_draft_input_preflighted",
          "form_draft_agent_started",
          "form_draft_output_validation_started",
          "form_draft_output_validation_passed",
          "form_draft_review_ready",
        ],
      },
      i18n.t
    )

    expect(notice).toContain(i18n.t("commerce.common.smartFillProcess.streamStarted"))
    expect(notice).toContain(i18n.t("commerce.common.smartFillProcess.streamInputPreflighted"))
    expect(notice).toContain(i18n.t("commerce.common.smartFillProcess.streamAgentStarted"))
    expect(notice).toContain(i18n.t("commerce.common.smartFillProcess.streamValidationStarted"))
    expect(notice).toContain(i18n.t("commerce.common.smartFillProcess.streamValidationPassed"))
    expect(notice).toContain(i18n.t("commerce.common.smartFillProcess.streamReviewReady"))
    expect(notice).not.toContain("hidden-model")
    expect(notice).not.toContain("127.0.0.1")
    expect(notice).not.toContain("product_form_draft")
  })

  it("summarizes failed stream stages without exposing failure internals", async () => {
    const i18n = await createAgentChatTestI18n()
    const notice = buildFormDraftProcessNotice(
      {
        agentContext: {
          failureKind: "agent_run_failed",
          generationSource: "agent_llm",
          messageAvailable: true,
          [promptKey]: "zh/form_drafts/product_form_draft.md",
        },
        stages: ["form_draft_failed"],
      },
      i18n.t
    )

    expect(notice).toContain(i18n.t("commerce.common.smartFillProcess.streamFailed"))
    expect(notice).not.toContain("agent_run_failed")
    expect(notice).not.toContain("product_form_draft")
  })
})
