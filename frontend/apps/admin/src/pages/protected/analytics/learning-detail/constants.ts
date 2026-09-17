export const learningStudentDetailQueryKey = (userId: string) =>
  ["admin", "learning", "students", "detail", userId] as const
