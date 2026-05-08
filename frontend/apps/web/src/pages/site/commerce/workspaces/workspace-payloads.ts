import {
  toNumberOrNull,
  toOptionalString,
  type SimpleFormValues,
} from "../components/commerce-ui"

export function createTaskPayload(values: SimpleFormValues) {
  return {
    acceptanceCriteria: toOptionalString(values.acceptanceCriteria),
    dependsOn: [],
    description: toOptionalString(values.description),
    metadata: {},
    ownerRole: toOptionalString(values.ownerRole),
    priority: toNumberOrNull(values.priority) ?? 0,
    result: null,
    status: values.status || "pending",
    title: values.title,
  }
}

export function createRunTaskPayload(values: SimpleFormValues) {
  return {
    acceptanceNotes: toOptionalString(values.acceptanceNotes),
    acceptancePassed: values.acceptancePassed === "true",
    acceptanceScore: toNumberOrNull(values.acceptanceScore) ?? 0,
    attempts: toNumberOrNull(values.attempts) ?? 0,
    executionNotes: toOptionalString(values.executionNotes),
    metadata: {},
    ownerRole: toOptionalString(values.ownerRole),
    seq: toNumberOrNull(values.seq) ?? 1,
    status: values.status || "pending",
    workspaceTaskId: toOptionalString(values.workspaceTaskId),
  }
}
