use axum::{Router, routing::get};

use crate::handlers::app::{
    create_workspace, create_workspace_run, create_workspace_run_task, create_workspace_task,
    delete_workspace, delete_workspace_task, get_workspace, get_workspace_run, get_workspace_task,
    list_workspace_run_tasks, list_workspace_runs, list_workspace_tasks, list_workspaces,
    update_workspace, update_workspace_run, update_workspace_run_task, update_workspace_task,
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
            "/workspaces/{workspace_id}/runs/{run_id}/tasks",
            get(list_workspace_run_tasks).post(create_workspace_run_task),
        )
        .route(
            "/workspaces/{workspace_id}/runs/{run_id}/tasks/{run_task_id}",
            axum::routing::patch(update_workspace_run_task),
        )
}
