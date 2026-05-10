use axum::{Router, routing::get};

use crate::handlers::app::{
    create_workspace, create_workspace_artifact, create_workspace_run, create_workspace_run_task,
    create_workspace_task, delete_workspace, delete_workspace_artifact, delete_workspace_task,
    evaluate_agent_run_quality, get_workspace, get_workspace_artifact, get_workspace_context_pack,
    get_workspace_context_summary, get_workspace_run, get_workspace_task,
    list_agent_learning_records, list_agent_quality_checks, list_agent_trust_scores,
    list_workspace_artifacts, list_workspace_run_tasks, list_workspace_runs, list_workspace_tasks,
    list_workspaces, submit_agent_run_feedback, update_workspace, update_workspace_artifact,
    update_workspace_run, update_workspace_run_task, update_workspace_task,
};

pub fn workspace_routes() -> Router {
    Router::new()
        .route("/workspaces", get(list_workspaces).post(create_workspace))
        .route(
            "/workspaces/{workspace_id}",
            get(get_workspace)
                .patch(update_workspace)
                .delete(delete_workspace),
        )
        .route(
            "/workspaces/{workspace_id}/tasks",
            get(list_workspace_tasks).post(create_workspace_task),
        )
        .route(
            "/workspaces/{workspace_id}/tasks/{task_id}",
            get(get_workspace_task)
                .patch(update_workspace_task)
                .delete(delete_workspace_task),
        )
        .route(
            "/workspaces/{workspace_id}/runs",
            get(list_workspace_runs).post(create_workspace_run),
        )
        .route(
            "/workspaces/{workspace_id}/runs/{run_id}",
            get(get_workspace_run).patch(update_workspace_run),
        )
        .route(
            "/workspaces/{workspace_id}/runs/{run_id}/quality-checks",
            get(list_agent_quality_checks).post(evaluate_agent_run_quality),
        )
        .route(
            "/workspaces/{workspace_id}/runs/{run_id}/feedback",
            axum::routing::post(submit_agent_run_feedback),
        )
        .route(
            "/workspaces/{workspace_id}/runs/{run_id}/tasks",
            get(list_workspace_run_tasks).post(create_workspace_run_task),
        )
        .route(
            "/workspaces/{workspace_id}/runs/{run_id}/tasks/{run_task_id}",
            axum::routing::patch(update_workspace_run_task),
        )
        .route(
            "/workspaces/{workspace_id}/artifacts",
            get(list_workspace_artifacts).post(create_workspace_artifact),
        )
        .route(
            "/workspaces/{workspace_id}/context-summary",
            get(get_workspace_context_summary),
        )
        .route(
            "/workspaces/{workspace_id}/context-pack",
            get(get_workspace_context_pack),
        )
        .route(
            "/workspaces/{workspace_id}/artifacts/{artifact_id}",
            get(get_workspace_artifact)
                .patch(update_workspace_artifact)
                .delete(delete_workspace_artifact),
        )
        .route(
            "/workspaces/{workspace_id}/learnings",
            get(list_agent_learning_records),
        )
        .route("/trust-scores", get(list_agent_trust_scores))
}
