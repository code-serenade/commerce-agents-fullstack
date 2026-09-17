import { useStudentLearningProbe } from "../view-model/use-student-learning-probe"
import { StudentLearningProbe } from "./student-learning-probe"
import { StudentLearningProbeHistory } from "./student-learning-probe-history"

export function StudentLearningProbeSlot({
  conversationId,
  messageId,
  onAnswer,
  isCurrent,
}: {
  conversationId: string
  messageId: string
  onAnswer: (question: string) => void
  isCurrent: boolean
}) {
  const learningProbe = useStudentLearningProbe({ conversationId, messageId })

  return (
    <>
      <StudentLearningProbeHistory items={learningProbe.completedProbes} />
      <StudentLearningProbe
        isCurrent={isCurrent}
        onAnswer={onAnswer}
        onEngage={learningProbe.onEngage}
        onSubmitChoice={learningProbe.onSubmitChoice}
        onContinue={learningProbe.onContinue}
        probe={learningProbe.probe}
        ready={learningProbe.ready}
        result={learningProbe.result}
        resultFailed={learningProbe.resultFailed}
        onRetryResult={learningProbe.onRetryResult}
        continuing={learningProbe.continuing}
        continueFailed={learningProbe.continueFailed}
        continueUnavailable={learningProbe.continueUnavailable}
        submittingChoice={learningProbe.submittingChoice}
      />
    </>
  )
}
