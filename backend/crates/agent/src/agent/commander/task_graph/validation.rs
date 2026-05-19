use std::collections::{HashMap, HashSet};

use super::config::{TaskGraphRule, TaskGraphRuleSet};
use crate::roles::RoleCatalog;

pub(super) fn validate_task_graph_rules(rules: &TaskGraphRuleSet) -> Result<(), String> {
    validate_task_graph_rules_with_catalog(rules, &RoleCatalog::configured())
}

fn validate_task_graph_rules_with_catalog(
    rules: &TaskGraphRuleSet,
    catalog: &RoleCatalog,
) -> Result<(), String> {
    if rules.schema_version > 2 {
        return Err(format!(
            "unsupported task graph rules schema_version {}",
            rules.schema_version
        ));
    }

    let valid_roles = catalog
        .roles()
        .iter()
        .map(|role| role.runtime_role.clone())
        .collect::<HashSet<_>>();
    let mut rule_ids = HashSet::new();

    for rule in &rules.rules {
        validate_rule(rule, &valid_roles)?;
        if !rule_ids.insert(rule.id.clone()) {
            return Err(format!("duplicate rule id '{}'", rule.id));
        }
    }

    Ok(())
}

fn validate_rule(rule: &TaskGraphRule, valid_roles: &HashSet<String>) -> Result<(), String> {
    if rule.id.trim().is_empty() {
        return Err("rule id is required".to_string());
    }

    let mut node_ids = HashSet::new();
    for node in &rule.nodes {
        validate_node_shape(&rule.id, &node.id, &node.role, valid_roles)?;
        if !node_ids.insert(node.id.clone()) {
            return Err(format!(
                "rule '{}' has duplicate node '{}'",
                rule.id, node.id
            ));
        }
    }

    for node in &rule.conditional_nodes {
        validate_node_shape(&rule.id, &node.id, &node.role, valid_roles)?;
        if !node_ids.insert(node.id.clone()) {
            return Err(format!(
                "rule '{}' has duplicate node '{}'",
                rule.id, node.id
            ));
        }
    }

    if let Some(final_node) = &rule.final_node
        && final_node.enabled
    {
        validate_node_shape(&rule.id, &final_node.id, &final_node.role, valid_roles)?;
    }

    if rule.nodes.is_empty()
        && rule.conditional_nodes.is_empty()
        && !rule.dynamic_roles.from_explicit_request.enabled
        && !rule.dynamic_roles.from_route.enabled
    {
        return Err(format!(
            "rule '{}' must declare nodes or enable dynamic roles",
            rule.id
        ));
    }

    validate_static_dependencies(rule, &node_ids)?;
    validate_rule_has_no_static_cycles(rule)
}

fn validate_node_shape(
    rule_id: &str,
    node_id: &str,
    role: &str,
    valid_roles: &HashSet<String>,
) -> Result<(), String> {
    if node_id.trim().is_empty() {
        return Err(format!("rule '{rule_id}' has node without id"));
    }
    if !valid_roles.contains(role) {
        return Err(format!(
            "rule '{rule_id}' node '{node_id}' uses unknown role '{role}'"
        ));
    }
    Ok(())
}

fn validate_static_dependencies(
    rule: &TaskGraphRule,
    node_ids: &HashSet<String>,
) -> Result<(), String> {
    for node in &rule.nodes {
        validate_node_dependencies(&rule.id, &node.id, &node.depends_on, node_ids)?;
    }
    for node in &rule.conditional_nodes {
        validate_node_dependencies(&rule.id, &node.id, &node.depends_on, node_ids)?;
    }
    Ok(())
}

fn validate_node_dependencies(
    rule_id: &str,
    node_id: &str,
    depends_on: &[String],
    node_ids: &HashSet<String>,
) -> Result<(), String> {
    for dep in depends_on {
        if !node_ids.contains(dep) {
            return Err(format!(
                "rule '{rule_id}' node '{node_id}' depends on unknown node '{dep}'"
            ));
        }
        if dep == node_id {
            return Err(format!(
                "rule '{rule_id}' node '{node_id}' cannot depend on itself"
            ));
        }
    }
    Ok(())
}

fn validate_rule_has_no_static_cycles(rule: &TaskGraphRule) -> Result<(), String> {
    #[derive(Clone, Copy, PartialEq, Eq)]
    enum VisitState {
        Visiting,
        Done,
    }

    fn visit(
        node_id: &str,
        deps: &HashMap<&str, Vec<&str>>,
        states: &mut HashMap<String, VisitState>,
        rule_id: &str,
    ) -> Result<(), String> {
        match states.get(node_id).copied() {
            Some(VisitState::Visiting) => {
                return Err(format!(
                    "rule '{rule_id}' contains dependency cycle at node '{node_id}'"
                ));
            }
            Some(VisitState::Done) => return Ok(()),
            None => {}
        }

        states.insert(node_id.to_string(), VisitState::Visiting);
        for dep in deps.get(node_id).into_iter().flatten() {
            visit(dep, deps, states, rule_id)?;
        }
        states.insert(node_id.to_string(), VisitState::Done);
        Ok(())
    }

    let mut deps = rule
        .nodes
        .iter()
        .map(|node| {
            (
                node.id.as_str(),
                node.depends_on
                    .iter()
                    .map(String::as_str)
                    .collect::<Vec<_>>(),
            )
        })
        .collect::<HashMap<_, _>>();
    deps.extend(rule.conditional_nodes.iter().map(|node| {
        (
            node.id.as_str(),
            node.depends_on
                .iter()
                .map(String::as_str)
                .collect::<Vec<_>>(),
        )
    }));

    let mut states = HashMap::new();
    for node_id in deps.keys() {
        visit(node_id, &deps, &mut states, &rule.id)?;
    }
    Ok(())
}
