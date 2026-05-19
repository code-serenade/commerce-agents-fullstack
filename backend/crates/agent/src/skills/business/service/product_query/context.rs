use serde_json::{Map, Value};

use super::{
    matching::{
        ProductContextMatch, matching_product_ids, matching_product_ids_by_ref, matching_refs,
        matching_sources_by_product_id, matching_sources_by_product_ref, matching_sources_by_text,
        source_product_id,
    },
    metadata::{
        collect_metadata_list, collect_metadata_sources, collect_metadata_text_parts,
        collect_metadata_texts,
    },
    output::{eq_ignore_ascii_case_trimmed, strip_product_ref_prefix, truncate_chars},
};

#[derive(Clone, Debug, Default)]
pub(super) struct ExplicitProductContext {
    pub(super) product_ids: Vec<String>,
    pub(super) refs: Vec<String>,
    pub(super) sources: Vec<Value>,
    pub(super) source_titles: Vec<String>,
    pub(super) summary: String,
    explicit_signal: bool,
}

impl ExplicitProductContext {
    pub(super) fn has_explicit_context(&self) -> bool {
        self.explicit_signal
    }

    pub(super) fn match_query(
        &self,
        product_ref: Option<&str>,
        product_id: Option<&str>,
        product_name: Option<&str>,
    ) -> ProductContextMatch {
        if let Some(product_ref) = product_ref.filter(|value| !value.trim().is_empty()) {
            if !self.matches_product_ref(product_ref) {
                return ProductContextMatch::default();
            }
            return ProductContextMatch {
                matched: true,
                product_ids: matching_product_ids_by_ref(&self.product_ids, product_ref),
                refs: matching_refs(&self.refs, product_ref),
                sources: matching_sources_by_product_ref(&self.sources, product_ref),
            };
        }

        if let Some(product_id) = product_id.filter(|value| !value.trim().is_empty()) {
            if !self.matches_product_id(product_id) {
                return ProductContextMatch::default();
            }
            return ProductContextMatch {
                matched: true,
                product_ids: matching_product_ids(&self.product_ids, product_id),
                refs: matching_refs(&self.refs, product_id),
                sources: matching_sources_by_product_id(&self.sources, product_id),
            };
        }

        if let Some(product_name) = product_name.filter(|value| !value.trim().is_empty()) {
            if !self.matches_product_name(product_name) {
                return ProductContextMatch::default();
            }
            return ProductContextMatch {
                matched: true,
                product_ids: self.product_ids.clone(),
                refs: self.refs.clone(),
                sources: matching_sources_by_text(&self.sources, product_name),
            };
        }

        ProductContextMatch {
            matched: true,
            product_ids: self.product_ids.clone(),
            refs: self.refs.clone(),
            sources: self.sources.clone(),
        }
    }

    fn matches_product_ref(&self, product_ref: &str) -> bool {
        let product_ref = product_ref.trim();
        let product_id = strip_product_ref_prefix(product_ref).unwrap_or(product_ref);
        self.product_ids
            .iter()
            .any(|item| eq_ignore_ascii_case_trimmed(item, product_id))
            || self
                .refs
                .iter()
                .any(|item| eq_ignore_ascii_case_trimmed(item, product_ref))
            || self.sources.iter().any(|source| {
                source_product_id(source)
                    .as_deref()
                    .is_some_and(|value| eq_ignore_ascii_case_trimmed(value, product_id))
                    || text_contains_case_insensitive(&source.to_string(), product_ref)
                    || text_contains_case_insensitive(&source.to_string(), product_id)
            })
            || text_contains_case_insensitive(&self.summary, product_ref)
            || text_contains_case_insensitive(&self.summary, product_id)
            || self.source_titles.iter().any(|item| {
                text_contains_case_insensitive(item, product_ref)
                    || text_contains_case_insensitive(item, product_id)
            })
    }

    fn matches_product_id(&self, product_id: &str) -> bool {
        let product_id = product_id.trim();
        self.product_ids
            .iter()
            .any(|item| eq_ignore_ascii_case_trimmed(item, product_id))
            || self
                .refs
                .iter()
                .any(|item| text_contains_case_insensitive(item, product_id))
            || self.sources.iter().any(|source| {
                source_product_id(source)
                    .as_deref()
                    .is_some_and(|value| eq_ignore_ascii_case_trimmed(value, product_id))
                    || text_contains_case_insensitive(&source.to_string(), product_id)
            })
            || text_contains_case_insensitive(&self.summary, product_id)
            || self
                .source_titles
                .iter()
                .any(|item| text_contains_case_insensitive(item, product_id))
    }

    fn matches_product_name(&self, product_name: &str) -> bool {
        let product_name = product_name.trim();
        text_contains_case_insensitive(&self.summary, product_name)
            || self
                .source_titles
                .iter()
                .any(|item| text_contains_case_insensitive(item, product_name))
            || self
                .sources
                .iter()
                .any(|source| text_contains_case_insensitive(&source.to_string(), product_name))
    }
}

pub(super) fn explicit_product_context(metadata: &Map<String, Value>) -> ExplicitProductContext {
    let product_ids = collect_metadata_list(
        metadata,
        &["request.context.product_ids", "context.product_ids"],
    );
    let refs = collect_metadata_list(
        metadata,
        &[
            "request.context.refs",
            "context.refs",
            "request.context.combined_refs",
            "context.combined_refs",
        ],
    );
    let sources = collect_metadata_sources(
        metadata,
        &["request.context.sources_json", "context.sources_json"],
    );
    let source_titles = collect_metadata_text_parts(
        metadata,
        &["request.context.source_titles", "context.source_titles"],
    );
    let selected_summaries = collect_metadata_texts(
        metadata,
        &[
            "request.context.selected_summary",
            "context.selected_summary",
            "request.context.summary",
            "context.summary",
        ],
    );
    let combined_summaries = collect_metadata_texts(
        metadata,
        &[
            "request.context.combined_summary",
            "context.combined_summary",
        ],
    );
    let mut summary_parts = selected_summaries.clone();
    summary_parts.extend(combined_summaries.clone());
    let summary = truncate_chars(&summary_parts.join("\n\n"), 4000);

    let explicit_signal = !product_ids.is_empty()
        || refs.iter().any(|item| item.starts_with("product:"))
        || !sources.is_empty()
        || !source_titles.is_empty()
        || !selected_summaries.is_empty();

    ExplicitProductContext {
        product_ids,
        refs,
        sources,
        source_titles,
        summary,
        explicit_signal,
    }
}

fn text_contains_case_insensitive(haystack: &str, needle: &str) -> bool {
    haystack
        .to_lowercase()
        .contains(&needle.trim().to_lowercase())
}
