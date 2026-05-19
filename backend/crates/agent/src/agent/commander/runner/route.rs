use crate::{
    core::protocol::{TaskGraph, WorkerId},
    roles::RoleRoute,
};

pub(super) fn selected_runtime_roles(
    metadata: &std::collections::BTreeMap<String, String>,
) -> Vec<String> {
    metadata
        .get("strategy.selected_roles")
        .map(|value| {
            value
                .split(',')
                .map(str::trim)
                .filter(|item| !item.is_empty())
                .map(ToString::to_string)
                .collect()
        })
        .unwrap_or_default()
}

pub(super) fn normalize_requested_runtime_roles(
    collaboration_mode: &str,
    roles: Vec<String>,
    requested_primary: Option<String>,
) -> Vec<String> {
    let roles = unique_runtime_roles(roles);
    if roles.is_empty() {
        return Vec::new();
    }

    let non_chat_roles = roles
        .iter()
        .filter(|role| role.as_str() != "chat")
        .cloned()
        .collect::<Vec<_>>();
    let primary = requested_primary
        .filter(|role| role != "chat")
        .or_else(|| {
            non_chat_roles
                .iter()
                .find(|role| role.as_str() == "ops")
                .cloned()
        })
        .or_else(|| non_chat_roles.first().cloned())
        .or_else(|| roles.first().cloned());

    let Some(primary) = primary else {
        return Vec::new();
    };

    let mut selected = vec![primary.clone()];
    if collaboration_mode == "manual" {
        selected.extend(
            roles
                .into_iter()
                .filter(|role| role != "chat" && role != &primary),
        );
    }
    selected
}

fn unique_runtime_roles(roles: Vec<String>) -> Vec<String> {
    roles.into_iter().fold(Vec::new(), |mut unique, role| {
        let role = role.trim().to_string();
        if !role.is_empty() && !unique.contains(&role) {
            unique.push(role);
        }
        unique
    })
}

pub(super) fn primary_worker_id(role_route: &RoleRoute) -> WorkerId {
    WorkerId(format!("worker.{}", role_route.primary_runtime_role))
}

pub(super) fn primary_route_score(role_route: &RoleRoute) -> f32 {
    if !role_route.scores.is_empty() && role_route.scores.iter().all(|score| score.score >= 90.0) {
        return role_route
            .scores
            .iter()
            .find(|score| score.role_id == role_route.primary_role)
            .map(|score| score.score)
            .unwrap_or(0.0);
    }
    role_route
        .scores
        .iter()
        .find(|score| score.role_id == role_route.primary_role)
        .map(|score| score.score)
        .unwrap_or(0.0)
}

pub(super) fn primary_graph_role(graph: &TaskGraph) -> Option<&str> {
    graph.nodes.first().map(|node| node.role.as_str())
}
