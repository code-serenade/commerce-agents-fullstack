import type { DataTableSortState } from "@workspace/app-kit"
import type {
  LearningStudentResponse,
  ListLearningStudentsRequest,
} from "@/api"
import { normalizeLearningAssessmentSummary } from "../../learning-assessment"
import { isLearningSortBy } from "./sort"
import type {
  LearningStudentRow,
  LearningStudentTableQuery,
} from "../types"

export function mapLearningStudentRow(
  student: LearningStudentResponse
): LearningStudentRow {
  return {
    user_id: student.userId,
    display_id: student.displayId?.trim() || student.userId,
    display_name: student.displayName?.trim() || student.userId,
    school_name: student.schoolName?.trim() || null,
    team_name: student.teamName?.trim() || null,
    coverage_rate: student.coverageRate,
    observed_count: student.observedKnowledgeCount,
    mastery_status: student.masteryStatus,
    assessment_summary: normalizeLearningAssessmentSummary(
      student.assessmentSummary
    ),
    enabled: student.enabled,
    last_analyzed_at: student.lastAnalyzedAt ?? null,
    createdAt: student.createdAt ?? null,
    updatedAt: student.updatedAt ?? null,
  }
}

export function buildLearningListParams(
  page: number,
  pageSize: number,
  query: LearningStudentTableQuery,
  sort: DataTableSortState | null
): ListLearningStudentsRequest {
  const params: ListLearningStudentsRequest = {
    page,
    pageSize,
  }
  const keyword = query.keyword.trim()

  if (keyword) {
    params.keyword = keyword
  }

  if (sort && isLearningSortBy(sort.columnKey)) {
    params.sortBy = sort.columnKey
    params.sortOrder = sort.direction
  }

  return params
}
