import { LearningProgressView } from "./components/learning-progress-view"
import { useLearningProgressPage } from "./use-learning-progress-page"

export default function LearningProgressPage() {
  const page = useLearningProgressPage()
  return <LearningProgressView {...page} />
}
