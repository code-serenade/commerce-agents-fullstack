use std::{
    cmp::Ordering,
    sync::RwLock,
    time::{Duration, SystemTime},
};

use super::{
    DeterministicMemoryCompactor, MEMORY_TRUNCATED_TAG, MemoryBundle, MemoryCompactionPolicy,
    MemoryId, MemoryKind, MemoryQuery, MemoryRecord, MemoryScope, MemorySource, MemoryStore,
    MemoryWriteOutcome, SessionSnapshot, char_count, scope_identity_eq, scope_query_matches,
    truncate_chars,
};
use crate::agent_session::memory::MemoryCompactor;

#[derive(Debug, Default)]
pub struct InMemoryMemoryStore {
    records: RwLock<Vec<MemoryRecord>>,
    policy: MemoryCompactionPolicy,
    compactor: DeterministicMemoryCompactor,
}

impl InMemoryMemoryStore {
    pub fn new() -> Self {
        Self::default()
    }

    pub fn with_policy(policy: MemoryCompactionPolicy) -> Self {
        Self {
            records: RwLock::new(Vec::new()),
            policy,
            compactor: DeterministicMemoryCompactor,
        }
    }

    pub fn with_records(policy: MemoryCompactionPolicy, records: Vec<MemoryRecord>) -> Self {
        let store = Self::with_policy(policy);
        for record in records {
            MemoryStore::upsert_record(&store, record);
        }
        store
    }

    pub fn records(&self) -> Vec<MemoryRecord> {
        self.records
            .read()
            .expect("memory store lock poisoned when reading records")
            .clone()
    }
}

impl MemoryStore for InMemoryMemoryStore {
    fn load_bundle(&self, query: MemoryQuery) -> MemoryBundle {
        let mut char_budget = query.budget.max_chars;
        let limit = query.limit.min(query.budget.max_records);
        let mut records = self
            .records
            .read()
            .expect("memory store lock poisoned when loading bundle")
            .iter()
            .filter(|record| scope_query_matches(&query.scope, &record.scope))
            .filter(|record| query.kinds.is_empty() || query.kinds.contains(&record.kind))
            .cloned()
            .collect::<Vec<_>>();
        sort_records_for_query(&mut records, query.text.as_deref());
        let records = records
            .into_iter()
            .filter_map(|record| {
                if char_budget == 0 {
                    return None;
                }
                let mut record = record;
                let chars = char_count(&record.content);
                if chars > char_budget {
                    record.content = truncate_chars(&record.content, char_budget);
                    push_unique_term(&mut record.tags, MEMORY_TRUNCATED_TAG.to_string());
                }
                char_budget = char_budget.saturating_sub(char_count(&record.content));
                Some(record)
            })
            .take(limit)
            .collect::<Vec<_>>();
        let summary = records
            .iter()
            .find(|record| record.kind == MemoryKind::SessionSummary)
            .map(|record| record.content.clone());

        MemoryBundle { records, summary }
    }

    fn persist_session(
        &self,
        scope: &MemoryScope,
        snapshot: SessionSnapshot,
    ) -> MemoryWriteOutcome {
        if snapshot.summary.trim().is_empty() && snapshot.final_output.trim().is_empty() {
            return MemoryWriteOutcome::default();
        }

        let structured_cards = structured_memory_cards(scope, &snapshot);
        let mut outcome = MemoryWriteOutcome::default();
        let content = if snapshot.summary.trim().is_empty() {
            snapshot.final_output
        } else {
            snapshot.summary
        };

        outcome.add(self.upsert_record(MemoryRecord {
            id: MemoryId(next_memory_id()),
            scope: scope.clone(),
            kind: MemoryKind::SessionSummary,
            key: scope.session_id.as_ref().map(|id| id.0.clone()),
            content,
            confidence: 1.0,
            tags: vec!["session".to_string()],
            source: MemorySource::session_result(),
            created_at: SystemTime::now(),
            updated_at: SystemTime::now(),
        }));

        for (key, value) in snapshot.blackboard {
            if value.trim().is_empty() {
                continue;
            }
            outcome.add(self.upsert_record(MemoryRecord {
                id: MemoryId(next_memory_id()),
                scope: scope.clone(),
                kind: MemoryKind::ConversationFact,
                key: Some(key),
                content: value,
                confidence: 0.8,
                tags: vec!["blackboard".to_string()],
                source: MemorySource::session_result(),
                created_at: SystemTime::now(),
                updated_at: SystemTime::now(),
            }));
        }

        for task_graph in snapshot.task_graphs {
            if !task_graph.root_task.trim().is_empty() {
                outcome.add(self.upsert_record(MemoryRecord {
                    id: MemoryId(next_memory_id()),
                    scope: scope.clone(),
                    kind: MemoryKind::CrossRoleContext,
                    key: Some(format!("task_graph:{}", task_graph.graph_id.0)),
                    content: format_task_graph_memory(&task_graph),
                    confidence: 0.85,
                    tags: vec!["task_graph".to_string(), "session".to_string()],
                    source: MemorySource::session_result(),
                    created_at: SystemTime::now(),
                    updated_at: SystemTime::now(),
                }));
            }
        }

        for role_summary in snapshot.role_summaries {
            if role_summary.summary.trim().is_empty()
                && role_summary.findings.is_empty()
                && role_summary.recommendations.is_empty()
            {
                continue;
            }
            outcome.add(self.upsert_record(MemoryRecord {
                id: MemoryId(next_memory_id()),
                scope: scope.clone(),
                kind: MemoryKind::CrossRoleContext,
                key: Some(format!(
                    "role:{}:{}",
                    role_summary.graph_id.0, role_summary.node_id.0
                )),
                content: format_role_memory(&role_summary),
                confidence: 0.85,
                tags: vec!["role_summary".to_string(), role_summary.role.clone()],
                source: MemorySource::session_result(),
                created_at: SystemTime::now(),
                updated_at: SystemTime::now(),
            }));
        }

        if !snapshot.risks.is_empty() {
            outcome.add(
                self.upsert_record(MemoryRecord {
                    id: MemoryId(next_memory_id()),
                    scope: scope.clone(),
                    kind: MemoryKind::ConversationFact,
                    key: scope
                        .session_id
                        .as_ref()
                        .map(|id| format!("session:{}:risks", id.0)),
                    content: format_list("Risks", &snapshot.risks),
                    confidence: 0.8,
                    tags: vec!["risk".to_string(), "session".to_string()],
                    source: MemorySource::session_result(),
                    created_at: SystemTime::now(),
                    updated_at: SystemTime::now(),
                }),
            );
        }

        if !snapshot.open_questions.is_empty() {
            outcome.add(
                self.upsert_record(MemoryRecord {
                    id: MemoryId(next_memory_id()),
                    scope: scope.clone(),
                    kind: MemoryKind::ConversationFact,
                    key: scope
                        .session_id
                        .as_ref()
                        .map(|id| format!("session:{}:open_questions", id.0)),
                    content: format_list("Open questions", &snapshot.open_questions),
                    confidence: 0.8,
                    tags: vec!["open_question".to_string(), "session".to_string()],
                    source: MemorySource::session_result(),
                    created_at: SystemTime::now(),
                    updated_at: SystemTime::now(),
                }),
            );
        }

        if let Some(evaluation_summary) = snapshot.evaluation_summary
            && !evaluation_summary.trim().is_empty()
        {
            outcome.add(
                self.upsert_record(MemoryRecord {
                    id: MemoryId(next_memory_id()),
                    scope: scope.clone(),
                    kind: MemoryKind::ConversationFact,
                    key: scope
                        .session_id
                        .as_ref()
                        .map(|id| format!("session:{}:evaluation", id.0)),
                    content: evaluation_summary,
                    confidence: 0.75,
                    tags: vec!["evaluation".to_string(), "session".to_string()],
                    source: MemorySource::session_result(),
                    created_at: SystemTime::now(),
                    updated_at: SystemTime::now(),
                }),
            );
        }
        for card in structured_cards {
            outcome.add(self.upsert_record(card));
        }
        outcome
    }

    fn upsert_record(&self, record: MemoryRecord) -> MemoryWriteOutcome {
        let (record, record_stats) = match self.compactor.compact_record(record, &self.policy) {
            (Some(record), stats) => (record, stats),
            (None, stats) => {
                return MemoryWriteOutcome {
                    written: 0,
                    compaction: stats,
                };
            }
        };

        let scope = record.scope.clone();
        let kind = record.kind.clone();
        let mut outcome = MemoryWriteOutcome {
            written: 1,
            compaction: record_stats,
        };

        let mut records = self
            .records
            .write()
            .expect("memory store lock poisoned when upserting record");

        if let Some(key) = &record.key
            && let Some(existing) = records.iter_mut().find(|existing| {
                scope_identity_eq(&existing.scope, &record.scope)
                    && existing.kind == record.kind
                    && existing.key.as_ref() == Some(key)
            })
        {
            let original_id = existing.id.clone();
            let created_at = existing.created_at;
            existing.content = record.content;
            existing.confidence = record.confidence;
            existing.tags = record.tags;
            existing.source = record.source;
            existing.id = original_id;
            existing.created_at = created_at;
            existing.updated_at = SystemTime::now();
            outcome.compaction.merged_records += 1;
        } else {
            records.push(record);
        }

        let group = records
            .iter()
            .filter(|record| scope_identity_eq(&record.scope, &scope) && record.kind == kind)
            .cloned()
            .collect::<Vec<_>>();
        let (compacted_group, group_stats) = self.compactor.compact_scope_kind(group, &self.policy);
        outcome.compaction.add(group_stats);
        records.retain(|record| !(scope_identity_eq(&record.scope, &scope) && record.kind == kind));
        records.extend(compacted_group);
        outcome
    }
}

struct StructuredMemoryCard {
    key: String,
    title: String,
    values: Vec<String>,
    tags: Vec<String>,
    confidence: f32,
}

fn structured_memory_cards(scope: &MemoryScope, snapshot: &SessionSnapshot) -> Vec<MemoryRecord> {
    let mut cards = Vec::new();
    push_structured_memory_card(
        &mut cards,
        scope,
        StructuredMemoryCard {
            key: "facts".to_string(),
            title: "Facts".to_string(),
            values: structured_fact_values(snapshot),
            tags: vec!["memory_card".to_string(), "fact".to_string()],
            confidence: 0.82,
        },
    );
    push_structured_memory_card(
        &mut cards,
        scope,
        StructuredMemoryCard {
            key: "decisions".to_string(),
            title: "Decisions".to_string(),
            values: structured_decision_values(snapshot),
            tags: vec!["memory_card".to_string(), "decision".to_string()],
            confidence: 0.84,
        },
    );
    push_structured_memory_card(
        &mut cards,
        scope,
        StructuredMemoryCard {
            key: "constraints".to_string(),
            title: "Constraints".to_string(),
            values: snapshot.risks.clone(),
            tags: vec!["memory_card".to_string(), "constraint".to_string()],
            confidence: 0.8,
        },
    );
    push_structured_memory_card(
        &mut cards,
        scope,
        StructuredMemoryCard {
            key: "open_questions".to_string(),
            title: "Open questions".to_string(),
            values: snapshot.open_questions.clone(),
            tags: vec!["memory_card".to_string(), "open_question".to_string()],
            confidence: 0.8,
        },
    );
    cards
}

fn push_structured_memory_card(
    cards: &mut Vec<MemoryRecord>,
    scope: &MemoryScope,
    card: StructuredMemoryCard,
) {
    let values = unique_non_empty(card.values);
    if values.is_empty() {
        return;
    }
    cards.push(MemoryRecord {
        id: MemoryId(next_memory_id()),
        scope: MemoryScope {
            session_id: None,
            task_id: None,
            ..scope.clone()
        },
        kind: MemoryKind::ConversationFact,
        key: Some(format!("memory_card:{}", card.key)),
        content: format_list(&card.title, &values),
        confidence: card.confidence,
        tags: {
            let mut tags = card.tags;
            tags.extend(entity_tags(&values.join("\n")));
            tags.push("session".to_string());
            tags.sort();
            tags.dedup();
            tags
        },
        source: MemorySource::session_result(),
        created_at: SystemTime::now(),
        updated_at: SystemTime::now(),
    });
}

fn structured_fact_values(snapshot: &SessionSnapshot) -> Vec<String> {
    let mut values = Vec::new();
    if !snapshot.summary.trim().is_empty() {
        values.push(snapshot.summary.clone());
    } else if !snapshot.final_output.trim().is_empty() {
        values.push(snapshot.final_output.clone());
    }
    values.extend(
        snapshot
            .role_summaries
            .iter()
            .flat_map(|role| role.findings.clone()),
    );
    values
}

fn structured_decision_values(snapshot: &SessionSnapshot) -> Vec<String> {
    let mut values = snapshot
        .role_summaries
        .iter()
        .flat_map(|role| role.recommendations.clone())
        .collect::<Vec<_>>();
    values.extend(
        snapshot
            .task_graphs
            .iter()
            .filter_map(|task_graph| task_graph.process.as_ref())
            .flat_map(|process| process.next_actions.clone()),
    );
    values
}

fn unique_non_empty(values: Vec<String>) -> Vec<String> {
    values.into_iter().fold(Vec::new(), |mut unique, value| {
        let value = value.trim();
        if !value.is_empty() && !unique.iter().any(|existing| existing == value) {
            unique.push(value.to_string());
        }
        unique
    })
}

fn sort_records_for_query(records: &mut [MemoryRecord], query_text: Option<&str>) {
    let query_terms = query_text
        .map(query_terms)
        .filter(|terms| !terms.is_empty())
        .unwrap_or_default();
    if query_terms.is_empty() {
        records.sort_by(compare_updated_at_desc);
        return;
    }
    let mut scored = records
        .iter()
        .cloned()
        .map(|record| {
            let score = memory_query_score(&record, &query_terms);
            let ref_score = memory_query_context_ref_score(&record, &query_terms);
            ScoredMemoryRecord {
                record,
                score,
                ref_score,
            }
        })
        .collect::<Vec<_>>();
    scored.sort_by(|left, right| {
        right
            .ref_score
            .cmp(&left.ref_score)
            .then_with(|| right.score.cmp(&left.score))
            .then_with(|| {
                if left.ref_score == 0
                    && right.ref_score == 0
                    && left.score == 0
                    && right.score == 0
                {
                    return compare_updated_at_desc(&left.record, &right.record);
                }
                right
                    .record
                    .confidence
                    .partial_cmp(&left.record.confidence)
                    .unwrap_or(Ordering::Equal)
            })
            .then_with(|| compare_updated_at_desc(&left.record, &right.record))
    });
    for (target, scored) in records.iter_mut().zip(scored.into_iter()) {
        *target = scored.record;
    }
}

struct ScoredMemoryRecord {
    record: MemoryRecord,
    score: usize,
    ref_score: usize,
}

fn memory_query_context_ref_score(record: &MemoryRecord, query_terms: &[String]) -> usize {
    query_terms
        .iter()
        .filter(|term| is_context_ref_term(term))
        .filter(|term| memory_record_has_entity_tag(record, term))
        .map(|term| memory_entity_tag_weight(term))
        .sum()
}

fn memory_record_has_entity_tag(record: &MemoryRecord, term: &str) -> bool {
    record.tags.iter().any(|tag| {
        let Some((prefix, value)) = tag.split_once(':') else {
            return false;
        };
        if !prefix.eq_ignore_ascii_case("entity") {
            return false;
        }
        if is_context_ref_term(term) {
            value.eq_ignore_ascii_case(term)
        } else {
            value == term
        }
    })
}

fn compare_updated_at_desc(left: &MemoryRecord, right: &MemoryRecord) -> Ordering {
    if right.updated_at > left.updated_at {
        Ordering::Greater
    } else if right.updated_at < left.updated_at {
        Ordering::Less
    } else {
        Ordering::Equal
    }
}

fn memory_query_score(record: &MemoryRecord, query_terms: &[String]) -> usize {
    if query_terms.is_empty() {
        return 0;
    }
    let mut haystack = record.content.to_lowercase();
    if let Some(key) = &record.key {
        haystack.push(' ');
        haystack.push_str(&key.to_lowercase());
    }
    haystack.push(' ');
    haystack.push_str(
        &record
            .tags
            .iter()
            .map(|tag| tag.to_lowercase())
            .collect::<Vec<_>>()
            .join(" "),
    );
    query_terms
        .iter()
        .map(|term| {
            let mut score = if haystack.contains(term.as_str()) {
                memory_query_term_weight(term)
            } else {
                0
            };
            if memory_record_has_entity_tag(record, term) {
                score += memory_entity_tag_weight(term);
            }
            score
        })
        .sum()
}

fn query_terms(value: &str) -> Vec<String> {
    raw_query_terms(value)
        .into_iter()
        .fold(Vec::<String>::new(), |mut terms, term| {
            let Some(term) = normalize_memory_query_term(&term) else {
                return terms;
            };
            push_unique_term(&mut terms, term.clone());
            if contains_cjk(&term) {
                for ngram in cjk_ngrams(&term) {
                    if !is_memory_query_stop_term(&ngram) {
                        push_unique_term(&mut terms, ngram);
                    }
                }
            }
            terms
        })
}

fn memory_query_term_weight(term: &str) -> usize {
    if is_context_ref_term(term) {
        return 6;
    }
    let chars = term.chars().count();
    if contains_cjk(term) {
        if chars >= 5 {
            3
        } else if chars >= 3 {
            2
        } else {
            1
        }
    } else if chars >= 8 {
        3
    } else if chars >= 4 {
        2
    } else {
        1
    }
}

fn memory_entity_tag_weight(term: &str) -> usize {
    if is_context_ref_term(term) {
        return 12;
    }
    let chars = term.chars().count();
    if chars >= 5 { 8 } else { 4 }
}

fn entity_tags(value: &str) -> Vec<String> {
    raw_query_terms(value)
        .into_iter()
        .filter(|term| is_entity_like_term(term))
        .map(|term| format!("entity:{term}"))
        .fold(Vec::<String>::new(), |mut tags, tag| {
            if !tags.iter().any(|existing| existing == &tag) {
                tags.push(tag);
            }
            tags
        })
}

fn is_entity_like_term(term: &str) -> bool {
    if is_context_ref_term(term) {
        return true;
    }
    let chars = term.chars().count();
    (3..=18).contains(&chars)
        && term
            .chars()
            .any(|ch| ch.is_ascii_alphanumeric() || ('\u{4e00}'..='\u{9fff}').contains(&ch))
        && !term.chars().all(|ch| ch.is_ascii_digit())
}

fn raw_query_terms(value: &str) -> Vec<String> {
    let mut terms = context_ref_terms(value);
    value
        .split(|ch: char| {
            ch.is_whitespace()
                || matches!(
                    ch,
                    ',' | '，'
                        | '.'
                        | '。'
                        | ':'
                        | '：'
                        | ';'
                        | '；'
                        | '?'
                        | '？'
                        | '!'
                        | '！'
                        | '/'
                        | '\\'
                        | '|'
                        | '('
                        | ')'
                        | '（'
                        | '）'
                        | '['
                        | ']'
                        | '【'
                        | '】'
                )
        })
        .map(str::trim)
        .filter(|term| term.chars().count() >= 2)
        .map(|term| term.to_lowercase())
        .for_each(|term| {
            push_unique_term(&mut terms, term);
        });
    terms
}

fn context_ref_terms(value: &str) -> Vec<String> {
    value
        .split(|ch: char| {
            ch.is_whitespace()
                || matches!(
                    ch,
                    ',' | '，'
                        | ';'
                        | '；'
                        | '?'
                        | '？'
                        | '!'
                        | '！'
                        | '\''
                        | '"'
                        | '/'
                        | '\\'
                        | '|'
                        | '('
                        | ')'
                        | '（'
                        | '）'
                        | '['
                        | ']'
                        | '【'
                        | '】'
                        | '{'
                        | '}'
                )
        })
        .map(trim_context_ref_candidate)
        .filter_map(normalize_context_ref_term)
        .fold(Vec::<String>::new(), |mut terms, term| {
            push_unique_term(&mut terms, term);
            terms
        })
}

fn trim_context_ref_candidate(value: &str) -> &str {
    value.trim_matches(|ch: char| {
        !(ch.is_ascii_alphanumeric() || matches!(ch, ':' | '_' | '-' | '.'))
    })
}

fn is_context_ref_term(term: &str) -> bool {
    let Some((prefix, id)) = term.split_once(':') else {
        return false;
    };
    matches!(
        prefix,
        "product" | "campaign" | "conversation_file" | "workspace_source"
    ) && !id.is_empty()
        && id
            .chars()
            .all(|ch| ch.is_ascii_alphanumeric() || matches!(ch, '_' | '-' | '.'))
}

fn normalize_context_ref_term(term: &str) -> Option<String> {
    let (prefix, id) = term.split_once(':')?;
    let prefix = prefix.to_ascii_lowercase();
    if matches!(
        prefix.as_str(),
        "product" | "campaign" | "conversation_file" | "workspace_source"
    ) && !id.is_empty()
        && id
            .chars()
            .all(|ch| ch.is_ascii_alphanumeric() || matches!(ch, '_' | '-' | '.'))
    {
        Some(format!("{prefix}:{}", id.to_ascii_lowercase()))
    } else {
        None
    }
}

const MEMORY_QUERY_STOP_TERMS: &[&str] = &[
    "continue", "previous", "above", "earlier", "this", "that", "update", "revise", "adjust",
    "继续", "刚才", "上面", "上次", "前面", "这个", "那个", "这些", "那些", "调整", "修改", "优化",
];

fn normalize_memory_query_term(term: &str) -> Option<String> {
    let mut value = term.trim();
    if value.is_empty() {
        return None;
    }

    if contains_cjk(value) {
        value = trim_cjk_memory_query_stop_edges(value);
    }

    if value.is_empty() || is_memory_query_stop_term(value) {
        None
    } else {
        Some(value.to_string())
    }
}

fn trim_cjk_memory_query_stop_edges(mut value: &str) -> &str {
    loop {
        let Some(stop_term) = cjk_memory_query_stop_terms().find(|stop_term| {
            value.starts_with(**stop_term) && value.chars().count() > stop_term.chars().count()
        }) else {
            break;
        };
        value = &value[stop_term.len()..];
    }

    loop {
        let Some(stop_term) = cjk_memory_query_stop_terms().find(|stop_term| {
            value.ends_with(**stop_term) && value.chars().count() > stop_term.chars().count()
        }) else {
            break;
        };
        let end = value.len() - stop_term.len();
        value = &value[..end];
    }

    value
}

fn cjk_memory_query_stop_terms() -> impl Iterator<Item = &'static &'static str> {
    MEMORY_QUERY_STOP_TERMS
        .iter()
        .filter(|stop_term| contains_cjk(stop_term))
}

fn is_memory_query_stop_term(term: &str) -> bool {
    if MEMORY_QUERY_STOP_TERMS.contains(&term) {
        return true;
    }
    if !contains_cjk(term) {
        return false;
    }

    let mut rest = term;
    while !rest.is_empty() {
        let Some(stop_term) =
            cjk_memory_query_stop_terms().find(|stop_term| rest.starts_with(**stop_term))
        else {
            return false;
        };
        rest = &rest[stop_term.len()..];
    }
    true
}

fn push_unique_term(terms: &mut Vec<String>, term: String) {
    if !terms.iter().any(|existing| existing == &term) {
        terms.push(term);
    }
}

fn contains_cjk(value: &str) -> bool {
    value
        .chars()
        .any(|ch| ('\u{4e00}'..='\u{9fff}').contains(&ch))
}

fn cjk_ngrams(value: &str) -> Vec<String> {
    let chars = value.chars().collect::<Vec<_>>();
    let mut terms = Vec::new();
    for width in 2..=6 {
        if chars.len() < width {
            continue;
        }
        for start in 0..=chars.len() - width {
            let term = chars[start..start + width].iter().collect::<String>();
            if contains_cjk(&term) {
                push_unique_term(&mut terms, term);
            }
        }
    }
    terms
}

fn next_memory_id() -> String {
    use std::sync::atomic::{AtomicU64, Ordering};

    static NEXT_MEMORY_ID: AtomicU64 = AtomicU64::new(1);

    let millis = SystemTime::now()
        .duration_since(SystemTime::UNIX_EPOCH)
        .unwrap_or(Duration::ZERO)
        .as_millis();
    let sequence = NEXT_MEMORY_ID.fetch_add(1, Ordering::Relaxed);
    format!("mem-{millis}-{sequence}")
}

fn format_task_graph_memory(summary: &super::TaskGraphMemorySummary) -> String {
    let mut lines = vec![
        format!("Task graph: {}", summary.graph_id.0),
        format!("Root task: {}", summary.root_task),
        format!("Roles: {}", summary.roles.join(", ")),
    ];
    if let Some(process) = &summary.process {
        lines.push(format!(
            "Process panel: {} (default_collapsed={}, anchor={})",
            process.panel.label, process.panel.default_collapsed, process.panel.anchor
        ));
        if let Some(primary) = &process.primary {
            lines.push(format!(
                "Primary role: {} ({})",
                primary.role_label, primary.role
            ));
            if let Some(summary) = &primary.summary
                && !summary.trim().is_empty()
            {
                lines.push(format!("Primary summary: {}", summary.trim()));
            }
        }
        if !process.handoffs.is_empty() {
            let handoffs = process
                .handoffs
                .iter()
                .map(|handoff| format!("- {}", handoff.summary.trim()))
                .collect::<Vec<_>>()
                .join("\n");
            lines.push(format!("Process handoffs:\n{handoffs}"));
        }
        if !process.next_actions.is_empty() {
            lines.push(format_list("Process next actions", &process.next_actions));
        }
        if !process.deliverables.is_empty() {
            lines.push(format_list("Process deliverables", &process.deliverables));
        }
        if !process.incomplete.is_empty() {
            let gaps = process
                .incomplete
                .iter()
                .map(|gap| {
                    format!(
                        "- {} ({}) {}: {}",
                        gap.title, gap.role_label, gap.state, gap.reason
                    )
                })
                .collect::<Vec<_>>()
                .join("\n");
            lines.push(format!("Process incomplete:\n{gaps}"));
        }
    }
    if !summary.risks.is_empty() {
        lines.push(format_list("Risks", &summary.risks));
    }
    if !summary.open_questions.is_empty() {
        lines.push(format_list("Open questions", &summary.open_questions));
    }
    if let Some(evaluation) = &summary.evaluation_summary
        && !evaluation.trim().is_empty()
    {
        lines.push(format!("Evaluation:\n{evaluation}"));
    }
    lines.join("\n")
}

fn format_role_memory(summary: &super::RoleMemorySummary) -> String {
    let mut lines = vec![
        format!("Role: {}", summary.role),
        format!("Worker: {}", summary.worker_id.0),
        format!("Summary: {}", summary.summary),
    ];
    if !summary.findings.is_empty() {
        lines.push(format_list("Findings", &summary.findings));
    }
    if !summary.recommendations.is_empty() {
        lines.push(format_list("Recommendations", &summary.recommendations));
    }
    if !summary.evidence.is_empty() {
        lines.push(format_list("Evidence", &summary.evidence));
    }
    if !summary.risks.is_empty() {
        lines.push(format_list("Risks", &summary.risks));
    }
    if !summary.open_questions.is_empty() {
        lines.push(format_list("Open questions", &summary.open_questions));
    }
    lines.join("\n")
}

fn format_list(label: &str, items: &[String]) -> String {
    let items = items
        .iter()
        .filter(|item| !item.trim().is_empty())
        .map(|item| format!("- {}", item.trim()))
        .collect::<Vec<_>>()
        .join("\n");
    format!("{label}:\n{items}")
}
