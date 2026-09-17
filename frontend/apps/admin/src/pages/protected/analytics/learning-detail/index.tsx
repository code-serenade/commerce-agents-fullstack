import { useCallback } from "react"
import { useNavigate, useParams } from "react-router"
import { useLearningStudentDetail } from "./learning-detail-data"
import { LearningStudentDetailView } from "./components/learning-detail-view"

export default function LearningStudentDetailPage() {
  const { userId } = useParams<{ userId: string }>()
  const navigate = useNavigate()
  const detailQuery = useLearningStudentDetail(userId)
  const onBack = useCallback(
    () => navigate("/analytics/learning"),
    [navigate]
  )

  return (
    <LearningStudentDetailView
      detail={detailQuery.data}
      isLoading={detailQuery.isLoading}
      onBack={onBack}
    />
  )
}
