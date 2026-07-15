import { Lightbulb } from "lucide-react"
import { useTranslation } from "react-i18next"
import { useNavigate } from "react-router"
import { useOptionalAgentChatContext } from "../agent-chat/context/use-agent-chat-context"
import {
  getStudentPromptRecipeItems,
  getStudentTaskEntries,
} from "../agent-chat/view-model/student-next-actions-view-model"
import {
  CommerceBody,
  CommercePageHeader,
  CommerceSurface,
} from "../commerce/components/commerce-ui"
import { DocumentIntroCard } from "../commerce/components/document-section"
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
      />
      <CommerceBody className="min-[1200px]:px-8">
        <DocumentIntroCard
          title={t("questionIdeas.principle.title")}
          description={t("questionIdeas.principle.description")}
          icon={<Lightbulb className="size-4" />}
        />
        <div className="space-y-6 [&>section+section]:border-t [&>section+section]:border-(--app-border)/60 [&>section+section]:pt-6">
          <QuestionIdeasRecipePanel items={recipeItems} />
          <QuestionIdeasTaskPanel
            items={taskEntries}
            onSelectPrompt={applyPrompt}
          />
        </div>
      </CommerceBody>
    </CommerceSurface>
  )
}
