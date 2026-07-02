import { useTranslation } from "react-i18next"
import { useNavigate } from "react-router"
import { Button } from "@workspace/ui-components"
import { useOptionalAgentChatContext } from "../agent-chat/context/use-agent-chat-context"
import {
  getStudentPromptRecipeItems,
  getStudentTaskEntries,
} from "../agent-chat/view-model/student-next-actions-view-model"
import {
  CommerceBody,
  CommercePageHeader,
  CommerceSurface,
  InlineAlert,
} from "../commerce/components/commerce-ui"
import {
  QuestionIdeasRecipePanel,
  QuestionIdeasTaskPanel,
} from "./components/question-ideas-panels"

export default function QuestionIdeasPage() {
  const { t } = useTranslation()
  const chat = useOptionalAgentChatContext()
  const navigate = useNavigate()
  const taskEntries = getStudentTaskEntries(t)
  const recipeItems = getStudentPromptRecipeItems(t)

  const applyPrompt = (prompt: string) => {
    chat?.setInput(prompt)
    navigate("/commerce")
  }

  return (
    <CommerceSurface>
      <CommercePageHeader
        eyebrow={t("questionIdeas.header.eyebrow")}
        title={t("questionIdeas.header.title")}
        description={t("questionIdeas.header.description")}
        actions={
          <Button
            size="sm"
            variant="outline"
            onClick={() => navigate("/commerce")}
          >
            {t("questionIdeas.header.action")}
          </Button>
        }
      />
      <CommerceBody>
        <InlineAlert
          variant="success"
          title={t("questionIdeas.principle.title")}
          description={t("questionIdeas.principle.description")}
        />
        <div className="grid gap-4 lg:grid-cols-[1.15fr_0.85fr]">
          <QuestionIdeasTaskPanel
            items={taskEntries}
            onSelectPrompt={applyPrompt}
          />
          <QuestionIdeasRecipePanel items={recipeItems} />
        </div>
      </CommerceBody>
    </CommerceSurface>
  )
}
