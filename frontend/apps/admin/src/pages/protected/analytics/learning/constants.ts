import type { DataTableAuditColumnKey } from "@workspace/app-kit"
import type {
  LearningQuestionTableQuery,
  LearningStudentTableQuery,
} from "./types"

export const learningStudentsQueryKey = [
  "admin",
  "learning",
  "students",
] as const

export const learningMetricsQueryKey = [
  "admin",
  "learning",
  "metrics",
] as const

export const learningAuditColumns = [
  "updatedAt",
] as const satisfies readonly DataTableAuditColumnKey[]

export const learningInitialQuery: LearningStudentTableQuery = {
  keyword: "",
}

export const learningQuestionInitialQuery: LearningQuestionTableQuery = {
  keyword: "",
  knowledgeCode: "",
  releaseStatus: "",
}

export const learningPageSizeOptions = [10, 15, 30, 50] as const
