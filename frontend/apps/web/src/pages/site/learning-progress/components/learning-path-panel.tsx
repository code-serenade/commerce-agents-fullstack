import {
  CalendarClock,
  Layers3,
  MessageSquareText,
  Route,
  Target,
} from "lucide-react"
import { useTranslation } from "react-i18next"
import { Badge, Button, Separator } from "@workspace/ui-components"
import type { LearningDomainAssessment } from "@/api"
import {
  groupLearningPath,
  type LearningPathGroup,
} from "../domain/learning-support-surfaces"
import { translateLearningCode } from "../domain/learning-code-label"

const groupOrder: LearningPathGroup[] = ["priority", "evidence", "retention"]
const groupIcons = {
  priority: Target,
  evidence: Layers3,
  retention: CalendarClock,
} as const

export function LearningPathPanel({
  assessments,
  onContinueChat,
  onOpenConversation,
}: {
  assessments: LearningDomainAssessment[]
  onContinueChat: () => void
  onOpenConversation: (conversationId: string) => void
}) {
  const { t } = useTranslation()
  const groups = groupLearningPath(assessments)

  return (
    <section className="rounded-lg border border-(--agent-chat-soft-border) bg-(--agent-chat-elevated-surface) p-4 shadow-sm backdrop-blur-xl">
      <div className="flex flex-col gap-4 sm:flex-row sm:items-start sm:justify-between">
        <div className="min-w-0">
          <div className="flex items-center gap-2">
            <Route aria-hidden="true" className="size-4 text-primary" />
            <h2 className="text-sm font-semibold text-foreground">
              {t("learningProgress.learningPath.title")}
            </h2>
          </div>
          <p className="mt-1 text-xs/5 text-muted-foreground">
            {t("learningProgress.learningPath.description")}
          </p>
        </div>
        <Button size="sm" onClick={onContinueChat}>
          {t("learningProgress.learningPath.continueChat")}
        </Button>
      </div>

      {assessments.length === 0 ? (
        <div className="py-12 text-center">
          <Route aria-hidden="true" className="mx-auto size-6 text-primary" />
          <p className="mt-3 text-sm font-medium text-foreground">
            {t("learningProgress.learningPath.emptyTitle")}
          </p>
          <p className="mx-auto mt-1 max-w-xl text-xs/5 text-muted-foreground">
            {t("learningProgress.learningPath.emptyDescription")}
          </p>
        </div>
      ) : (
        <div className="mt-5 grid gap-6 lg:grid-cols-3">
          {groupOrder.map((group) => (
            <LearningPathGroupSection
              key={group}
              assessments={groups[group]}
              group={group}
              onContinueChat={onContinueChat}
              onOpenConversation={onOpenConversation}
            />
          ))}
        </div>
      )}
    </section>
  )
}

function LearningPathGroupSection({
  assessments,
  group,
  onContinueChat,
  onOpenConversation,
}: {
  assessments: LearningDomainAssessment[]
  group: LearningPathGroup
  onContinueChat: () => void
  onOpenConversation: (conversationId: string) => void
}) {
  const { t } = useTranslation()
  const Icon = groupIcons[group]

  return (
    <section aria-labelledby={`learning-path-${group}`}>
      <div className="flex items-start gap-2 border-b border-(--agent-chat-soft-border) pb-3">
        <Icon aria-hidden="true" className="mt-0.5 size-4 shrink-0 text-primary" />
        <div>
          <h3
            id={`learning-path-${group}`}
            className="text-xs font-semibold text-foreground"
          >
            {t(`learningProgress.learningPath.groups.${group}.title`)}
          </h3>
          <p className="mt-1 text-[11px]/4 text-muted-foreground">
            {t(`learningProgress.learningPath.groups.${group}.description`)}
          </p>
        </div>
      </div>
      {assessments.length === 0 ? (
        <p className="py-5 text-xs/5 text-muted-foreground">
          {t(`learningProgress.learningPath.groups.${group}.empty`)}
        </p>
      ) : (
        <div>
          {assessments.map((assessment, index) => {
            const conversationId = assessment.recentEvidence[0]?.conversationId
            return (
              <article key={assessment.knowledgeCode} className="py-4 last:pb-0">
                {index > 0 ? (
                  <div className="mb-4">
                    <Separator />
                  </div>
                ) : null}
                <div className="flex flex-wrap items-center justify-between gap-2">
                  <h4 className="text-sm font-medium text-foreground">
                    {t(`learningProgress.domains.${assessment.knowledgeCode}`)}
                  </h4>
                  <Badge variant="outline">
                    {t(
                      `learningProgress.domainStatus.${assessment.statusCode}`
                    )}
                  </Badge>
                </div>
                <p className="mt-2 text-xs/5 text-muted-foreground">
                  {translateLearningCode(
                    t,
                    "learningProgress.domainReasons",
                    assessment.reasonCode
                  )}
                </p>
                <p className="mt-2 text-xs/5 font-medium text-foreground">
                  {translateLearningCode(
                    t,
                    "learningProgress.domainNextActions",
                    assessment.nextActionCode
                  )}
                </p>
                <Button
                  className="mt-3"
                  size="sm"
                  variant="outline"
                  onClick={() =>
                    conversationId
                      ? onOpenConversation(conversationId)
                      : onContinueChat()
                  }
                >
                  <MessageSquareText aria-hidden="true" className="size-4" />
                  {conversationId
                    ? t("learningProgress.learningPath.openConversation")
                    : t("learningProgress.learningPath.startPractice")}
                </Button>
              </article>
            )
          })}
        </div>
      )}
    </section>
  )
}
