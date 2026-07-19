import {
  listWorkspaceArtifactsApi,
  listWorkspacesApi,
  type WorkspaceArtifactResponse,
  type WorkspaceResponse,
} from "@/api"

export interface AgentOutputDeliverable extends WorkspaceArtifactResponse {
  workspace: WorkspaceResponse
}

export async function loadBackendDeliverables() {
  const workspaceResponse = await listWorkspacesApi({
    pageSize: 50,
    status: "active",
  })
  const workspaces = workspaceResponse.workspaces
  const artifactGroups = await Promise.all(
    workspaces.map(async (workspace) => {
      const artifactResponse = await listWorkspaceArtifactsApi(workspace.id, {
        pageSize: 20,
        status: "active",
      })

      return artifactResponse.artifacts.map((artifact) => ({
        ...artifact,
        workspace,
      }))
    })
  )
  const deliverables = artifactGroups.flat().sort(compareDeliverables)

  return {
    deliverables,
    workspaces,
  }
}

function compareDeliverables(
  left: AgentOutputDeliverable,
  right: AgentOutputDeliverable
) {
  return readDeliverableTime(right) - readDeliverableTime(left)
}

function readDeliverableTime(artifact: WorkspaceArtifactResponse) {
  return Math.max(readTime(artifact.updatedAt), readTime(artifact.createdAt))
}

function readTime(value?: string) {
  if (!value) return 0
  const parsed = Date.parse(value)
  return Number.isFinite(parsed) ? parsed : 0
}
