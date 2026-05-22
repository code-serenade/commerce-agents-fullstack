use super::{
    builder::build_configured_task_graph,
    config::{
        TaskGraphRule, TaskGraphRuleMatch, TaskGraphRuleSet, configured_task_graph_rules_path,
        load_task_graph_rules,
    },
};
use crate::{
    agent::commander::task_graph::util::{contains_keyword, contains_string_any},
    core::protocol::{TaskGraph, TaskId},
    roles::RoleRoute,
};

pub(super) fn plan_configured_task_graph(
    task_id: &TaskId,
    task: &str,
    route: Option<&RoleRoute>,
) -> Option<TaskGraph> {
    let path = configured_task_graph_rules_path()?;

    let rules = match load_task_graph_rules(&path) {
        Ok(rules) => rules,
        Err(error) => {
            crate::log_error_msg!(&format!(
                "task graph rules ignored: path={}, error={error}",
                path.display()
            ));
            return None;
        }
    };
    let graph = plan_task_graph_from_rules(task_id, task, route, &rules)?;
    Some(graph)
}

fn plan_task_graph_from_rules(
    task_id: &TaskId,
    task: &str,
    route: Option<&RoleRoute>,
    rules: &TaskGraphRuleSet,
) -> Option<TaskGraph> {
    let rule = rules
        .rules
        .iter()
        .filter(|rule| rule.matches(task, route, rules))
        .max_by(|left, right| {
            left.priority
                .cmp(&right.priority)
                .then_with(|| right.id.cmp(&left.id))
        })?;
    let graph = build_configured_task_graph(task_id, task, route, rules, rule);
    (!graph.nodes.is_empty()).then_some(graph)
}

impl TaskGraphRule {
    pub(super) fn matches(
        &self,
        task: &str,
        route: Option<&RoleRoute>,
        rules: &TaskGraphRuleSet,
    ) -> bool {
        if self.id == "direct_chat_short_answer" && route_has_non_chat_roles(route) {
            return false;
        }
        self.legacy_match().matches(task, route, rules)
            && self.r#match.matches(task, route, rules)
            && (!self.r#match.explicit_team_request
                || contains_string_any(task, &explicit_team_keywords(rules)))
            && (!self.dynamic_roles.from_explicit_request.enabled
                || contains_string_any(
                    task,
                    &self.dynamic_roles.from_explicit_request.trigger_keywords,
                )
                || self
                    .dynamic_roles
                    .from_explicit_request
                    .trigger_keywords
                    .is_empty())
    }

    fn legacy_match(&self) -> TaskGraphRuleMatch {
        TaskGraphRuleMatch {
            any_keywords: self.match_any.clone(),
            all_keywords: self.match_all.clone(),
            all_keyword_groups: self.match_all_any.clone(),
            allow_without_route: true,
            ..TaskGraphRuleMatch::default()
        }
    }
}

impl TaskGraphRuleMatch {
    pub(super) fn matches(
        &self,
        task: &str,
        route: Option<&RoleRoute>,
        rules: &TaskGraphRuleSet,
    ) -> bool {
        if let Some(max_chars) = self.max_chars
            && task.trim().chars().count() > max_chars
        {
            return false;
        }
        if !self
            .all_keywords
            .iter()
            .all(|keyword| contains_keyword(task, keyword))
        {
            return false;
        }
        if !self.any_keywords.is_empty()
            && !self
                .any_keywords
                .iter()
                .any(|keyword| contains_keyword(task, keyword))
        {
            return false;
        }
        if !self
            .all_keyword_groups
            .iter()
            .all(|group| group.iter().any(|keyword| contains_keyword(task, keyword)))
        {
            return false;
        }
        if self
            .none_keywords
            .iter()
            .any(|keyword| contains_keyword(task, keyword))
        {
            return false;
        }
        if let Some(min_aliases) = self.min_explicit_role_aliases {
            let explicit_role_count = rules
                .role_aliases
                .values()
                .filter(|aliases| aliases.iter().any(|alias| contains_keyword(task, alias)))
                .count();
            if explicit_role_count < min_aliases {
                return false;
            }
        }
        self.route_matches(route)
    }

    fn route_matches(&self, route: Option<&RoleRoute>) -> bool {
        let needs_route = !self.route_primary_any.is_empty()
            || !self.route_support_any.is_empty()
            || self.route_score_min.is_some()
            || self.route_min_non_chat_roles.is_some();
        if !needs_route {
            return true;
        }
        let Some(route) = route else {
            return self.allow_without_route;
        };
        if !self.route_primary_any.is_empty()
            && !self
                .route_primary_any
                .iter()
                .any(|role| role == &route.primary_runtime_role)
        {
            return false;
        }
        if !self.route_support_any.is_empty()
            && !route.support_runtime_roles.iter().any(|role| {
                self.route_support_any
                    .iter()
                    .any(|expected| expected == role)
            })
        {
            return false;
        }
        if let Some(min_score) = self.route_score_min
            && (route.scores.is_empty() || route.scores.iter().any(|score| score.score < min_score))
        {
            return false;
        }
        if let Some(min_roles) = self.route_min_non_chat_roles {
            let count = std::iter::once(route.primary_runtime_role.as_str())
                .chain(route.support_runtime_roles.iter().map(String::as_str))
                .filter(|role| *role != "chat")
                .count();
            if count < min_roles {
                return false;
            }
        }
        true
    }
}

fn route_has_non_chat_roles(route: Option<&RoleRoute>) -> bool {
    let Some(route) = route else {
        return false;
    };
    route.primary_runtime_role != "chat"
        || route
            .support_runtime_roles
            .iter()
            .any(|role| role != "chat")
}

pub(super) fn explicit_team_keywords(rules: &TaskGraphRuleSet) -> Vec<String> {
    let mut keywords = rules.defaults.explicit_team_keywords.clone();
    keywords.extend(rules.rules.iter().flat_map(|rule| {
        rule.dynamic_roles
            .from_explicit_request
            .trigger_keywords
            .clone()
    }));
    keywords
}

#[cfg(test)]
mod tests {
    use super::{load_task_graph_rules, plan_task_graph_from_rules};
    use crate::core::protocol::TaskId;

    fn configured_rules_path() -> String {
        format!(
            "{}/../../config/json/agent/task-graph/index.json",
            env!("CARGO_MANIFEST_DIR")
        )
    }

    #[test]
    fn image_content_question_routes_to_single_chat_graph() {
        let rules =
            load_task_graph_rules(configured_rules_path()).expect("task graph rules should load");
        let graph = plan_task_graph_from_rules(
            &TaskId("task-image-question".to_string()),
            "图片内容是什么",
            None,
            &rules,
        )
        .expect("image question should match a graph");

        assert_eq!(graph.nodes.len(), 1);
        assert_eq!(graph.nodes[0].id.0, "chat");
        assert_eq!(graph.nodes[0].role, "chat");
        assert_eq!(graph.nodes[0].title, "图片直答");
    }

    #[test]
    fn generic_content_word_does_not_route_to_content_creation_team() {
        let rules =
            load_task_graph_rules(configured_rules_path()).expect("task graph rules should load");
        let graph = plan_task_graph_from_rules(
            &TaskId("task-content-word".to_string()),
            "这个资料内容是什么",
            None,
            &rules,
        );

        assert!(
            graph
                .as_ref()
                .is_none_or(|graph| graph.nodes.iter().all(|node| node.role == "chat")),
            "generic content question must not start a non-chat team graph"
        );
    }
}
