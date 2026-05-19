use std::collections::HashSet;

use super::{
    DEFAULT_NODE_ACCEPTANCE,
    config::{
        FinalNodeDependsOn, RouteSupportMode, TaskGraphConditionalNode, TaskGraphFinalNode,
        TaskGraphRule, TaskGraphRuleNode, TaskGraphRuleSet,
    },
    planner::explicit_team_keywords_set,
    util::{contains_keyword, contains_string_any},
};
use crate::{
    core::protocol::{TaskGraph, TaskGraphId, TaskId, TaskNode, TaskNodeId, WorkerId},
    roles::RoleRoute,
};

pub(super) fn build_configured_task_graph(
    task_id: &TaskId,
    task: &str,
    route: Option<&RoleRoute>,
    rules: &TaskGraphRuleSet,
    rule: &TaskGraphRule,
) -> TaskGraph {
    let graph_id = TaskGraphId(format!("graph-{}", task_id.0));
    let mut specs = Vec::new();
    specs.extend(rule.nodes.iter().cloned().map(TaskGraphNodeSpec::Node));
    specs.extend(
        rule.conditional_nodes
            .iter()
            .filter(|node| node.when.matches(task, route))
            .cloned()
            .map(TaskGraphNodeSpec::Conditional),
    );
    specs.extend(dynamic_node_specs(task, route, rules, rule));
    dedupe_node_specs(&mut specs);
    sort_node_specs(&mut specs, &rule.role_order);

    let mut nodes = specs
        .iter()
        .map(|spec| task_node_from_spec(spec, task, rules, rule))
        .collect::<Vec<_>>();
    if let Some(final_node) = &rule.final_node
        && final_node.enabled
    {
        add_final_node(&mut nodes, final_node, task, rules, rule);
    }

    TaskGraph {
        graph_id,
        root_task: task.to_string(),
        nodes,
    }
}

fn render_rule_template(template: &str, task: &str, rule: &TaskGraphRule) -> String {
    template
        .replace("{{task}}", task)
        .replace("{{rule_id}}", &rule.id)
        .replace("{{rule_description}}", &rule.description)
}

#[derive(Debug, Clone)]
enum TaskGraphNodeSpec {
    Node(TaskGraphRuleNode),
    Conditional(TaskGraphConditionalNode),
    Dynamic {
        id: String,
        role: String,
        depends_on: Vec<String>,
    },
}

impl TaskGraphNodeSpec {
    fn id(&self) -> &str {
        match self {
            Self::Node(node) => &node.id,
            Self::Conditional(node) => &node.id,
            Self::Dynamic { id, .. } => id,
        }
    }

    fn role(&self) -> &str {
        match self {
            Self::Node(node) => &node.role,
            Self::Conditional(node) => &node.role,
            Self::Dynamic { role, .. } => role,
        }
    }
}

fn dynamic_node_specs(
    task: &str,
    route: Option<&RoleRoute>,
    rules: &TaskGraphRuleSet,
    rule: &TaskGraphRule,
) -> Vec<TaskGraphNodeSpec> {
    let mut specs = Vec::new();
    if rule.dynamic_roles.from_explicit_request.enabled {
        for role in explicit_roles_from_aliases(task, rules) {
            specs.push(TaskGraphNodeSpec::Dynamic {
                id: role.clone(),
                depends_on: default_dynamic_depends_on(&role, &specs),
                role,
            });
        }
        if let Some(final_role) = rules.defaults.final_role.as_ref()
            && !specs.iter().any(|spec| spec.role() == final_role)
            && contains_string_any(
                task,
                &rule
                    .dynamic_roles
                    .from_explicit_request
                    .include_final_role_when_keywords,
            )
        {
            specs.push(TaskGraphNodeSpec::Dynamic {
                id: final_role.clone(),
                role: final_role.clone(),
                depends_on: Vec::new(),
            });
        }
    }

    if rule.dynamic_roles.from_route.enabled
        && let Some(route) = route
    {
        let mut roles = Vec::new();
        if rule.dynamic_roles.from_route.include_primary {
            roles.push(route.primary_runtime_role.clone());
        }
        if should_include_route_support(rule, task, route) {
            roles.extend(route.support_runtime_roles.iter().cloned());
        }
        for role in roles {
            if rule
                .dynamic_roles
                .from_route
                .exclude_roles
                .iter()
                .any(|excluded| excluded == &role)
            {
                continue;
            }
            specs.push(TaskGraphNodeSpec::Dynamic {
                id: role.clone(),
                depends_on: default_dynamic_depends_on(&role, &specs),
                role,
            });
        }
    }
    specs
}

fn should_include_route_support(rule: &TaskGraphRule, task: &str, route: &RoleRoute) -> bool {
    match rule.dynamic_roles.from_route.include_support {
        RouteSupportMode::Never => false,
        RouteSupportMode::Always => true,
        RouteSupportMode::WhenRequestedOrPrimaryData => {
            route.primary_runtime_role == "data"
                || contains_string_any(task, &explicit_team_keywords_set())
        }
    }
}

fn explicit_roles_from_aliases(task: &str, rules: &TaskGraphRuleSet) -> Vec<String> {
    let mut roles = Vec::new();
    for (role, aliases) in &rules.role_aliases {
        if aliases.iter().any(|alias| contains_keyword(task, alias)) {
            roles.push(role.clone());
        }
    }
    order_roles(roles, &rules.default_role_order())
}

fn default_dynamic_depends_on(role: &str, existing: &[TaskGraphNodeSpec]) -> Vec<String> {
    if role == "data" || !existing.iter().any(|spec| spec.role() == "data") {
        Vec::new()
    } else {
        vec!["data".to_string()]
    }
}

fn dedupe_node_specs(specs: &mut Vec<TaskGraphNodeSpec>) {
    let mut seen = HashSet::new();
    specs.retain(|spec| seen.insert(spec.id().to_string()));
}

fn sort_node_specs(specs: &mut [TaskGraphNodeSpec], role_order: &[String]) {
    specs.sort_by(|left, right| {
        role_rank(left.role(), role_order)
            .cmp(&role_rank(right.role(), role_order))
            .then_with(|| left.id().cmp(right.id()))
    });
}

fn task_node_from_spec(
    spec: &TaskGraphNodeSpec,
    task: &str,
    rules: &TaskGraphRuleSet,
    rule: &TaskGraphRule,
) -> TaskNode {
    match spec {
        TaskGraphNodeSpec::Node(node) => concrete_task_node(
            &node.id,
            &node.role,
            &node.title,
            &node.objective,
            &node.depends_on,
            node.max_attempts,
            &node.acceptance,
            task,
            rules,
            rule,
        ),
        TaskGraphNodeSpec::Conditional(node) => concrete_task_node(
            &node.id,
            &node.role,
            &node.title,
            &node.objective,
            &node.depends_on,
            node.max_attempts,
            &node.acceptance,
            task,
            rules,
            rule,
        ),
        TaskGraphNodeSpec::Dynamic {
            id,
            role,
            depends_on,
        } => concrete_task_node(id, role, "", "", depends_on, None, &[], task, rules, rule),
    }
}

#[allow(clippy::too_many_arguments)]
fn concrete_task_node(
    id: &str,
    role: &str,
    title: &str,
    objective: &str,
    depends_on: &[String],
    max_attempts: Option<u8>,
    acceptance: &[String],
    task: &str,
    rules: &TaskGraphRuleSet,
    rule: &TaskGraphRule,
) -> TaskNode {
    let role_default = rules.role_defaults.get(role);
    let title = if title.trim().is_empty() {
        role_default
            .map(|default| default.title.as_str())
            .filter(|value| !value.trim().is_empty())
            .unwrap_or("角色判断")
    } else {
        title
    };
    let objective = if objective.trim().is_empty() {
        role_default
            .map(|default| default.objective.as_str())
            .filter(|value| !value.trim().is_empty())
            .unwrap_or("围绕本岗位边界完成判断：{{task}}")
    } else {
        objective
    };
    let acceptance = if !acceptance.is_empty() {
        acceptance.to_vec()
    } else if let Some(default) = role_default
        && !default.acceptance.is_empty()
    {
        default.acceptance.clone()
    } else if !rules.defaults.node_acceptance.is_empty() {
        rules.defaults.node_acceptance.clone()
    } else {
        vec![DEFAULT_NODE_ACCEPTANCE.to_string()]
    };

    TaskNode {
        id: TaskNodeId(id.to_string()),
        role: role.to_string(),
        worker_id: WorkerId(format!("worker.{role}")),
        title: render_rule_template(title, task, rule),
        objective: render_rule_template(objective, task, rule),
        input_refs: depends_on
            .iter()
            .map(|dep| TaskNodeId(dep.clone()))
            .collect(),
        acceptance,
        max_attempts: max_attempts
            .or_else(|| role_default.and_then(|default| default.max_attempts))
            .or(rules.defaults.max_attempts)
            .unwrap_or(2)
            .max(1),
    }
}

fn add_final_node(
    nodes: &mut Vec<TaskNode>,
    final_node: &TaskGraphFinalNode,
    task: &str,
    rules: &TaskGraphRuleSet,
    rule: &TaskGraphRule,
) {
    if nodes.iter().any(|node| node.id.0 == final_node.id) {
        return;
    }
    let depends_on = final_node_depends_on(nodes, final_node);
    nodes.push(concrete_task_node(
        &final_node.id,
        &final_node.role,
        &final_node.title,
        &final_node.objective,
        &depends_on,
        final_node.max_attempts,
        &final_node.acceptance,
        task,
        rules,
        rule,
    ));
}

fn final_node_depends_on(nodes: &[TaskNode], final_node: &TaskGraphFinalNode) -> Vec<String> {
    match &final_node.depends_on {
        FinalNodeDependsOn::List(values) => values.clone(),
        FinalNodeDependsOn::Special(value) => match value.as_str() {
            "$all_previous" | "$all_previous_non_self" | "$all_non_final" => nodes
                .iter()
                .filter(|node| node.id.0 != final_node.id)
                .map(|node| node.id.0.clone())
                .collect(),
            "$primary" => nodes
                .first()
                .map(|node| vec![node.id.0.clone()])
                .unwrap_or_default(),
            _ => Vec::new(),
        },
        FinalNodeDependsOn::Empty => Vec::new(),
    }
}

fn order_roles(roles: Vec<String>, role_order: &[String]) -> Vec<String> {
    let mut roles = roles.into_iter().fold(Vec::new(), |mut unique, role| {
        if !unique.contains(&role) {
            unique.push(role);
        }
        unique
    });
    roles.sort_by(|left, right| {
        role_rank(left, role_order)
            .cmp(&role_rank(right, role_order))
            .then_with(|| left.cmp(right))
    });
    roles
}

fn role_rank(role: &str, role_order: &[String]) -> usize {
    role_order
        .iter()
        .position(|candidate| candidate == role)
        .unwrap_or(usize::MAX)
}
