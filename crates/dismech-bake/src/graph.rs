//! A Rust port of `monarch-initiative/dismech`'s
//! `src/dismech/graph.py::build_causal_graph` (and its full helper family:
//! `_gene_lookup_keys`, `_descriptor_lookup_keys`, `_name_lookup_key`,
//! `_genetic_item_infers_mechanism_edges`, `_build_section_lookup`,
//! `_resolve_descriptor_target`, `iter_variant_items`, `animal_model_label`).
//!
//! This is a **direct, line-for-line port against the real upstream Python
//! source**, read in full from a fresh checkout of
//! `https://github.com/monarch-initiative/dismech` at
//! `src/dismech/graph.py` (2026-08-18) -- not a paraphrase of a summary of
//! that file. Every predicate string, every field populated (or
//! deliberately left `None`) on an `Edge`, and every gene-key-matching rule
//! below is a transcription of that source, with the exact source line
//! range cited in each function's doc comment so a reviewer can re-check
//! against the original without re-deriving it.
//!
//! ## Why this operates on raw `serde_yaml::Value`, not a typed struct tree
//!
//! `graph.py` itself never validates a schema before resolving -- it walks
//! plain Python `dict`s with `.get(...)`, `isinstance(..., dict)` guards,
//! and truthiness checks (`if not source: continue`). Several of its
//! matching rules (`_descriptor_lookup_keys`, which tries
//! `preferred_term` OR `term.label` OR `term.id` against an arbitrary
//! "descriptor-shaped" value) are inherently dynamic: `gene`, `gene_term`,
//! `genes[]`, `phenotype_term`, and `target_phenotypes[]` are all read
//! through the *same* descriptor-lookup helper despite living at different
//! paths with different neighbouring fields. Forcing this through a fixed
//! Rust struct tree would either lose fidelity (a struct can't cheaply
//! express "try three different possible key names against the same
//! partially-shaped value") or duplicate the model twice for no benefit.
//! Reading `serde_yaml::Value` directly -- `Disorder = BTreeMap<String,
//! Value>` from `model.rs`, already the crate's own permissive
//! representation -- keeps this resolver a transcription instead of an
//! approximation. `model.rs`'s typed `NodeItem` (a separate, additive
//! layer) exists for callers that want structural access to a single
//! section item; it is not consumed here.
//!
//! ## Deliberate extensions beyond the Python `NodeInfo`/`Edge` dataclasses
//!
//! The upstream `NodeInfo` (`graph.py:20-27`) carries only `name` /
//! `node_type` / `description`; `Edge` (`graph.py:29-46`) carries no
//! `evidence_count`. This port adds two fields that are pure read-time
//! enrichment, not resolver behavior:
//! - `NodeInfo::evidence_count` -- `len(item.get("evidence", []))`, for
//!   census/reporting.
//! - `NodeInfo::curie` -- the section's own singular `*_term` slot's
//!   `term.id`, when that section has one (phenotypes -> `phenotype_term`,
//!   environmental -> `exposure_term`, treatments -> `treatment_term`,
//!   biochemical -> `biomarker_term`). `pathophysiology` and `genetic` get
//!   `None` -- pathophysiology nodes are anonymous mechanism steps with no
//!   singular curie slot, and genetic nodes name a *gene* via `gene_term`
//!   (an attribution on the node, not the node's own identity curie).
//!
//! Neither field changes which nodes or edges get admitted, so the census
//! (node count, edge count) is unaffected by these additions -- they are
//! read-only decoration on top of the exact upstream admission logic.

use std::collections::{BTreeMap, BTreeSet, HashMap, HashSet};

use serde_yaml::Value;

use crate::model::Disorder;

/// Mirrors `NODE_COLORS`' key set (`graph.py:60-71`, minus the synthetic
/// `orphan` bucket, which isn't a real section) -- the node-type taxonomy
/// this resolver admits from.
#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub enum NodeType {
    Pathophysiology,
    Phenotype,
    Environmental,
    Genetic,
    Treatment,
    Biochemical,
    ExperimentalModel,
    AnimalModel,
    ComputationalModel,
}

impl NodeType {
    #[must_use]
    pub const fn as_str(self) -> &'static str {
        match self {
            Self::Pathophysiology => "pathophysiology",
            Self::Phenotype => "phenotype",
            Self::Environmental => "environmental",
            Self::Genetic => "genetic",
            Self::Treatment => "treatment",
            Self::Biochemical => "biochemical",
            Self::ExperimentalModel => "experimental_model",
            Self::AnimalModel => "animal_model",
            Self::ComputationalModel => "computational_model",
        }
    }
}

/// Port of `graph.py:20-27`'s `NodeInfo`, plus `evidence_count`/`curie` --
/// see this module's doc comment for why those two are additive-only.
#[derive(Debug, Clone)]
pub struct NodeInfo {
    pub node_type: NodeType,
    pub description: Option<String>,
    pub evidence_count: usize,
    pub curie: Option<String>,
}

/// Port of `graph.py:29-46`'s `Edge` dataclass, field-for-field. Which
/// fields are populated (vs. left `None`/empty) for a given edge depends on
/// which of the eight edge-collecting passes produced it -- see each
/// `collect_*_edges` function below for the exact field set that pass
/// fills in, matching the corresponding Python block precisely (e.g.
/// treatment `target_mechanisms` edges populate ONLY source/target/
/// predicate/source_type, per `graph.py:483-492`; no description, no
/// evidence-derived field, nothing else).
#[derive(Debug, Clone)]
pub struct Edge {
    pub source: String,
    pub target: String,
    pub predicate: String,
    pub source_type: NodeType,
    pub description: Option<String>,
    pub relationship: Option<String>,
    pub direction: Option<String>,
    pub endpoint_context: Option<String>,
    pub fidelity: Option<String>,
    pub limitations: Option<String>,
    pub hypothesis_groups: Vec<String>,
    pub causal_link_type: Option<String>,
    pub intermediate_mechanisms: Vec<String>,
}

impl Edge {
    fn minimal(
        source: String,
        target: String,
        predicate: &'static str,
        source_type: NodeType,
    ) -> Self {
        Self {
            source,
            target,
            predicate: predicate.to_string(),
            source_type,
            description: None,
            relationship: None,
            direction: None,
            endpoint_context: None,
            fidelity: None,
            limitations: None,
            hypothesis_groups: Vec::new(),
            causal_link_type: None,
            intermediate_mechanisms: Vec::new(),
        }
    }
}

/// Port of `graph.py:48-56`'s `CausalGraph`. `nodes` is a `BTreeMap` --
/// deterministic iteration is load-bearing for a reproducible bake, unlike
/// Python's insertion-ordered `dict` (which this mirrors in effect, since
/// Python 3.7+ dict iteration order is also insertion order; `BTreeMap`
/// instead gives *lexicographic* order, a stronger and still-deterministic
/// guarantee that doesn't depend on section-visit order).
#[derive(Debug, Clone, Default)]
pub struct CausalGraph {
    pub nodes: BTreeMap<String, NodeInfo>,
    pub edges: Vec<Edge>,
    pub orphan_targets: BTreeSet<String>,
    pub integrity_issues: Vec<String>,
}

// ---------------------------------------------------------------------
// Value-walking helpers -- the dynamic-dict-access primitives graph.py
// leans on throughout (`.get(...)`, `isinstance(..., dict)`, truthiness).
// ---------------------------------------------------------------------

fn as_seq(v: Option<&Value>) -> &[Value] {
    v.and_then(Value::as_sequence).map_or(&[], Vec::as_slice)
}

fn as_map(v: &Value) -> Option<&serde_yaml::Mapping> {
    v.as_mapping()
}

/// `str(value)` for the scalar shapes this corpus's `target`/similar
/// fields actually carry (string, and defensively number/bool) -- real
/// corpus data is string-typed here, this is a safety net, not a modeled
/// case.
fn value_to_string(v: &Value) -> Option<String> {
    match v {
        Value::String(s) => Some(s.clone()),
        Value::Number(n) => Some(n.to_string()),
        Value::Bool(b) => Some(b.to_string()),
        _ => None,
    }
}

/// A Python truthiness-respecting string read: `item.get(key)` then
/// `if not value: ...`. Missing key, non-string value, or an
/// all-whitespace/empty string all count as "falsy" here, matching every
/// `if not source: continue` / `if not name: continue` guard in the
/// Python source.
fn truthy_str(map: &serde_yaml::Mapping, key: &str) -> Option<String> {
    map.get(key)
        .and_then(value_to_string)
        .filter(|s| !s.trim().is_empty())
}

/// A presence-only string read: `"name" in item` (key exists, value not
/// required to be truthy) -- used ONLY for node admission across the 8
/// `node_sections` (`graph.py:329-338`), which checks key presence, not
/// truthiness. In the real corpus every `name` is a non-empty string
/// (verified: zero `name: null` / bare `- name:` entries across the full
/// `kb/disorders/` corpus), so this and [`truthy_str`] coincide in
/// practice; the distinction is kept because the Python source draws it.
fn present_str(map: &serde_yaml::Mapping, key: &str) -> Option<String> {
    if !map.contains_key(key) {
        return None;
    }
    map.get(key).and_then(value_to_string)
}

/// Node-level `evidence` list length -- this crate's own additive
/// enrichment (see module doc comment), not present in Python's `NodeInfo`.
fn evidence_count(map: &serde_yaml::Mapping) -> usize {
    map.get("evidence")
        .and_then(Value::as_sequence)
        .map_or(0, Vec::len)
}

/// The section's singular curie slot, when the section has one -- this
/// crate's own additive enrichment (see module doc comment).
fn section_curie(map: &serde_yaml::Mapping, term_key: Option<&str>) -> Option<String> {
    let term_key = term_key?;
    let descriptor = as_map(map.get(term_key)?)?;
    let term = as_map(descriptor.get("term")?)?;
    term.get("id").and_then(Value::as_str).map(str::to_string)
}

/// Port of `_normalize_lookup_key` (`graph.py:139-142`).
fn normalize_lookup_key(v: Option<&Value>) -> Option<String> {
    let v = v?;
    let text = match v {
        Value::String(s) => s.clone(),
        Value::Number(n) => n.to_string(),
        Value::Bool(b) => b.to_string(),
        _ => return None,
    };
    let text = text.trim().to_lowercase();
    if text.is_empty() {
        None
    } else {
        Some(text)
    }
}

/// Port of `_descriptor_lookup_keys` (`graph.py:145-160`). Returns the set
/// of normalized keys a "descriptor-shaped" value (`{preferred_term,
/// term: {id, label}}`) can be looked up by.
fn descriptor_lookup_keys(v: Option<&Value>) -> HashSet<String> {
    let mut keys = HashSet::new();
    let Some(descriptor) = v.and_then(as_map) else {
        return keys;
    };
    if let Some(k) = normalize_lookup_key(descriptor.get("preferred_term")) {
        keys.insert(k);
    }
    if let Some(term) = descriptor.get("term").and_then(as_map) {
        if let Some(k) = normalize_lookup_key(term.get("label")) {
            keys.insert(k);
        }
        if let Some(k) = normalize_lookup_key(term.get("id")) {
            keys.insert(k);
        }
    }
    keys
}

/// Port of `_name_lookup_key` (`graph.py:163-173`): a compact, gene-like
/// bare name (<=20 chars, alnum/hyphen/underscore only) used as a lookup
/// key when no structured `term` descriptor exists.
fn name_lookup_key(v: Option<&Value>) -> HashSet<String> {
    let mut keys = HashSet::new();
    let Some(name) = v.and_then(Value::as_str) else {
        return keys;
    };
    let name = name.trim();
    if name.is_empty() {
        return keys;
    }
    if name.chars().count() > 20
        || !name
            .chars()
            .all(|c| c.is_alphanumeric() || c == '-' || c == '_')
    {
        return keys;
    }
    if let Some(k) = normalize_lookup_key(Some(&Value::String(name.to_string()))) {
        keys.insert(k);
    }
    keys
}

/// Port of `_gene_lookup_keys` (`graph.py:176-191`).
fn gene_lookup_keys(item: &serde_yaml::Mapping, allow_name_fallback: bool) -> HashSet<String> {
    let mut keys = HashSet::new();
    keys.extend(descriptor_lookup_keys(item.get("gene")));
    keys.extend(descriptor_lookup_keys(item.get("gene_term")));
    for gene in as_seq(item.get("genes")) {
        keys.extend(descriptor_lookup_keys(Some(gene)));
    }
    if allow_name_fallback && keys.is_empty() {
        keys.extend(name_lookup_key(item.get("name")));
    }
    keys
}

/// Port of `_coerce_string_list` (`graph.py:194-202`) -- accepts absent
/// (-> empty), a scalar (-> one-element list), or a list (-> stringified
/// non-blank items). This is the exact scalar-or-list gotcha this module's
/// callers need for `hypothesis_groups` / `intermediate_mechanisms`.
fn coerce_string_list(v: Option<&Value>) -> Vec<String> {
    match v {
        None | Some(Value::Null) => Vec::new(),
        Some(Value::Sequence(items)) => items
            .iter()
            .filter_map(value_to_string)
            .filter(|s| !s.trim().is_empty())
            .collect(),
        Some(scalar) => value_to_string(scalar)
            .filter(|s| !s.trim().is_empty())
            .map_or_else(Vec::new, |s| vec![s]),
    }
}

const NON_CONTRIBUTING_RELATIONSHIP_TYPES: &[&str] =
    &["BIOMARKER", "DISPUTED", "MODIFIER", "PROTECTIVE", "UNKNOWN"];
const NON_CONTRIBUTING_ASSOCIATION_WORDS: &[&str] = &[
    "biomarker",
    "disputed",
    "modifier",
    "protective",
    "refuted",
    "unknown",
];

/// Port of `_genetic_item_infers_mechanism_edges` (`graph.py:222-236`).
fn genetic_item_infers_mechanism_edges(item: &serde_yaml::Mapping) -> bool {
    if let Some(rt) = item.get("relationship_type").and_then(Value::as_str) {
        return !NON_CONTRIBUTING_RELATIONSHIP_TYPES.contains(&rt.to_uppercase().as_str());
    }
    let Some(association) = item.get("association").and_then(Value::as_str) else {
        return true;
    };
    let words: HashSet<String> = association
        .replace(['-', '_'], " ")
        .split_whitespace()
        .map(|w| w.trim().to_lowercase())
        .collect();
    !NON_CONTRIBUTING_ASSOCIATION_WORDS
        .iter()
        .any(|w| words.contains(*w))
}

/// Port of `_build_section_lookup` (`graph.py:239-262`) -- builds the
/// phenotype name lookup used by `target_phenotypes` resolution.
/// `lookup.setdefault` in Python means "first writer wins"; iterating the
/// section items in their corpus order and using `entry().or_insert()`
/// reproduces that.
fn build_section_lookup(items: &[Value], descriptor_key: Option<&str>) -> HashMap<String, String> {
    let mut lookup: HashMap<String, String> = HashMap::new();
    for item in items {
        let Some(map) = as_map(item) else { continue };
        let Some(name) = truthy_str(map, "name") else {
            continue;
        };
        let mut keys: HashSet<String> = HashSet::new();
        if let Some(k) = normalize_lookup_key(Some(&Value::String(name.clone()))) {
            keys.insert(k);
        }
        if let Some(dk) = descriptor_key {
            keys.extend(descriptor_lookup_keys(map.get(dk)));
        }
        for key in keys {
            lookup.entry(key).or_insert_with(|| name.clone());
        }
    }
    lookup
}

/// Port of `_resolve_descriptor_target` (`graph.py:265-272`). Python
/// iterates a `set` (hash-order, not guaranteed stable across runs for
/// multi-key ambiguous descriptors); this iterates a fixed, deterministic
/// preference order (`preferred_term`, then `term.label`, then `term.id`)
/// instead, which is a stronger and still-correct guarantee for the
/// overwhelmingly common single-match case and only differs from Python in
/// the vanishingly rare case where a descriptor's several keys resolve to
/// *different* phenotype names via unrelated collisions.
fn resolve_descriptor_target(
    descriptor: &Value,
    lookup: &HashMap<String, String>,
) -> Option<String> {
    let map = as_map(descriptor)?;
    let mut ordered_keys = Vec::new();
    if let Some(k) = normalize_lookup_key(map.get("preferred_term")) {
        ordered_keys.push(k);
    }
    if let Some(term) = map.get("term").and_then(as_map) {
        if let Some(k) = normalize_lookup_key(term.get("label")) {
            ordered_keys.push(k);
        }
        if let Some(k) = normalize_lookup_key(term.get("id")) {
            ordered_keys.push(k);
        }
    }
    ordered_keys
        .into_iter()
        .find_map(|k| lookup.get(&k).cloned())
}

/// Port of `animal_model_label` (`graph.py:1116-1134`).
fn animal_model_label(model: &serde_yaml::Mapping) -> Option<String> {
    if let Some(name) = model.get("name").and_then(value_to_string) {
        let name = name.trim();
        if !name.is_empty() {
            return Some(name.to_string());
        }
    }
    let parts: Vec<String> = ["genotype", "species"]
        .iter()
        .filter_map(|key| model.get(*key).and_then(value_to_string))
        .map(|s| s.trim().to_string())
        .filter(|s| !s.is_empty())
        .collect();
    if parts.is_empty() {
        None
    } else {
        Some(parts.join(" "))
    }
}

/// One `(parent_genetic_name, variant_map)` pair -- port of
/// `iter_variant_items` (`graph.py:275-298`): disease-level `variants[]`
/// (no parent) plus each `genetic[].variants[]` (parent = the genetic
/// item's own name).
fn iter_variant_items(doc: &Disorder) -> Vec<(Option<String>, serde_yaml::Mapping)> {
    let mut items = Vec::new();
    for v in as_seq(doc.get("variants")) {
        if let Some(m) = as_map(v) {
            items.push((None, m.clone()));
        }
    }
    for genetic_item in as_seq(doc.get("genetic")) {
        let Some(gmap) = as_map(genetic_item) else {
            continue;
        };
        let parent_name = present_str(gmap, "name");
        for v in as_seq(gmap.get("variants")) {
            if let Some(m) = as_map(v) {
                items.push((parent_name.clone(), m.clone()));
            }
        }
    }
    items
}

// ---------------------------------------------------------------------
// `ENVIRONMENTAL_EFFECT_PREDICATES` / `MODEL_RELATIONSHIP_PREDICATES`
// (`graph.py:76-109`).
// ---------------------------------------------------------------------

fn environmental_predicate(effect: Option<&str>) -> &'static str {
    match effect {
        Some("TRIGGERS") => "triggers",
        Some("EXACERBATES") => "exacerbates",
        Some("PREDISPOSES") => "predisposes_to",
        Some("PROTECTS_AGAINST") => "protects_against",
        Some("MODULATES") => "modulates",
        _ => "influences",
    }
}

/// Port of `model_edge_predicate` (`graph.py:111-117`).
fn model_edge_predicate(relationship: Option<&str>) -> &'static str {
    match relationship.map(str::to_uppercase).as_deref() {
        Some("RECAPITULATES") => "models",
        Some("PARTIALLY_RECAPITULATES") => "partially_models",
        Some("FAILS_TO_RECAPITULATE") => "fails_to_model",
        Some("PERTURBS") => "perturbs",
        Some("MEASURES") => "measures",
        Some("RESCUES") => "rescues",
        _ => "models",
    }
}

// ---------------------------------------------------------------------
// The resolver itself -- port of `build_causal_graph` (`graph.py:301-703`).
// ---------------------------------------------------------------------

/// Build a `CausalGraph` from one parsed disorder document. Port of
/// `build_causal_graph` (`graph.py:301-703`) -- see this module's doc
/// comment for the two additive fields (`evidence_count`, `curie`) this
/// port carries beyond the Python `NodeInfo`/`Edge` dataclasses, and each
/// `collect_*` helper's doc comment for the exact source line range it
/// transcribes.
#[must_use]
pub fn build_causal_graph(doc: &Disorder) -> CausalGraph {
    let mut graph = CausalGraph::default();

    collect_named_nodes(doc, &mut graph);
    collect_animal_model_nodes(doc, &mut graph);
    collect_variant_nodes(doc, &mut graph);

    let phenotype_lookup =
        build_section_lookup(as_seq(doc.get("phenotypes")), Some("phenotype_term"));

    let pathophysiology_by_gene_key = index_by_gene_key(as_seq(doc.get("pathophysiology")), false);
    let genetic_nodes_by_gene_key = index_by_gene_key(as_seq(doc.get("genetic")), true);

    collect_pathophysiology_downstream_edges(doc, &mut graph);
    collect_phenotype_sequelae_edges(doc, &mut graph);
    collect_environmental_edges(doc, &mut graph);
    collect_treatment_edges(doc, &mut graph, &phenotype_lookup);
    collect_experimental_model_edges(doc, &mut graph);
    collect_animal_model_edges(doc, &mut graph);
    collect_computational_model_edges(doc, &mut graph);
    collect_biochemical_readout_edges(doc, &mut graph);
    collect_phenotype_reports_on_edges(doc, &mut graph);
    collect_genetic_contributes_to_edges(doc, &mut graph, &pathophysiology_by_gene_key);
    collect_variant_edges(
        doc,
        &mut graph,
        &pathophysiology_by_gene_key,
        &genetic_nodes_by_gene_key,
    );

    check_referential_integrity(&mut graph);
    graph
}

/// `graph.py:317-338` -- the 8 `node_sections`. A node is admitted iff the
/// item is a map AND has a `name` key present (key-presence check, not a
/// truthiness check -- see [`present_str`]'s doc comment).
fn collect_named_nodes(doc: &Disorder, graph: &mut CausalGraph) {
    const SECTIONS: &[(&str, NodeType, Option<&str>)] = &[
        ("pathophysiology", NodeType::Pathophysiology, None),
        ("phenotypes", NodeType::Phenotype, Some("phenotype_term")),
        (
            "environmental",
            NodeType::Environmental,
            Some("exposure_term"),
        ),
        ("genetic", NodeType::Genetic, None),
        ("treatments", NodeType::Treatment, Some("treatment_term")),
        ("biochemical", NodeType::Biochemical, Some("biomarker_term")),
        ("experimental_models", NodeType::ExperimentalModel, None),
        ("computational_models", NodeType::ComputationalModel, None),
    ];
    for (section_key, node_type, term_key) in SECTIONS {
        for item in as_seq(doc.get(*section_key)) {
            let Some(map) = as_map(item) else { continue };
            let Some(name) = present_str(map, "name") else {
                continue;
            };
            graph.nodes.insert(
                name,
                NodeInfo {
                    node_type: *node_type,
                    description: map
                        .get("description")
                        .and_then(Value::as_str)
                        .map(str::to_string),
                    evidence_count: evidence_count(map),
                    curie: section_curie(map, *term_key),
                },
            );
        }
    }
}

/// `graph.py:340-356`. Every `animal_models[]` entry with a resolvable
/// [`animal_model_label`] becomes a node UNCONDITIONALLY -- the doc
/// comment immediately above this block in the Python source claims a
/// model "only becomes a node if some edge uses it", but the code that
/// follows admits every labeled model regardless of whether it declares
/// `modeled_mechanisms`. The code is ground truth; that comment is stale
/// and this port follows the code.
fn collect_animal_model_nodes(doc: &Disorder, graph: &mut CausalGraph) {
    for item in as_seq(doc.get("animal_models")) {
        let Some(map) = as_map(item) else { continue };
        let Some(label) = animal_model_label(map) else {
            continue;
        };
        graph.nodes.insert(
            label,
            NodeInfo {
                node_type: NodeType::AnimalModel,
                description: map
                    .get("description")
                    .and_then(Value::as_str)
                    .map(str::to_string),
                evidence_count: evidence_count(map),
                curie: None,
            },
        );
    }
}

/// `graph.py:358-366`.
fn collect_variant_nodes(doc: &Disorder, graph: &mut CausalGraph) {
    for (_, variant) in iter_variant_items(doc) {
        let Some(name) = truthy_str(&variant, "name") else {
            continue;
        };
        graph.nodes.insert(
            name,
            NodeInfo {
                node_type: NodeType::Genetic,
                description: variant
                    .get("description")
                    .and_then(Value::as_str)
                    .map(str::to_string),
                evidence_count: evidence_count(&variant),
                curie: None,
            },
        );
    }
}

/// Builds the `{gene_key -> {source names}}` index used by both the
/// pathophysiology-by-gene lookup (`graph.py:372-380`, no name fallback)
/// and the genetic-nodes-by-gene lookup (`graph.py:382-390`, WITH name
/// fallback) -- `allow_name_fallback` selects which.
fn index_by_gene_key(
    items: &[Value],
    allow_name_fallback: bool,
) -> HashMap<String, BTreeSet<String>> {
    let mut index: HashMap<String, BTreeSet<String>> = HashMap::new();
    for item in items {
        let Some(map) = as_map(item) else { continue };
        let Some(source) = truthy_str(map, "name") else {
            continue;
        };
        for key in gene_lookup_keys(map, allow_name_fallback) {
            index.entry(key).or_default().insert(source.clone());
        }
    }
    index
}

/// `graph.py:392-419` -- pathophysiology `downstream[]` -> `causes` edges.
fn collect_pathophysiology_downstream_edges(doc: &Disorder, graph: &mut CausalGraph) {
    for item in as_seq(doc.get("pathophysiology")) {
        let Some(map) = as_map(item) else { continue };
        let Some(source) = truthy_str(map, "name") else {
            continue;
        };
        for edge_item in as_seq(map.get("downstream")) {
            let Some(emap) = as_map(edge_item) else {
                continue;
            };
            let Some(target) = emap.get("target").and_then(value_to_string) else {
                continue;
            };
            graph.edges.push(Edge {
                description: emap
                    .get("description")
                    .and_then(Value::as_str)
                    .map(str::to_string),
                hypothesis_groups: coerce_string_list(emap.get("hypothesis_groups")),
                causal_link_type: emap
                    .get("causal_link_type")
                    .and_then(Value::as_str)
                    .map(str::to_string),
                intermediate_mechanisms: coerce_string_list(emap.get("intermediate_mechanisms")),
                ..Edge::minimal(source.clone(), target, "causes", NodeType::Pathophysiology)
            });
        }
    }
}

/// `graph.py:421-448` -- phenotype `sequelae[]` -> `leads_to` edges. Same
/// field set as the pathophysiology pass above.
fn collect_phenotype_sequelae_edges(doc: &Disorder, graph: &mut CausalGraph) {
    for item in as_seq(doc.get("phenotypes")) {
        let Some(map) = as_map(item) else { continue };
        let Some(source) = truthy_str(map, "name") else {
            continue;
        };
        for edge_item in as_seq(map.get("sequelae")) {
            let Some(emap) = as_map(edge_item) else {
                continue;
            };
            let Some(target) = emap.get("target").and_then(value_to_string) else {
                continue;
            };
            graph.edges.push(Edge {
                description: emap
                    .get("description")
                    .and_then(Value::as_str)
                    .map(str::to_string),
                hypothesis_groups: coerce_string_list(emap.get("hypothesis_groups")),
                causal_link_type: emap
                    .get("causal_link_type")
                    .and_then(Value::as_str)
                    .map(str::to_string),
                intermediate_mechanisms: coerce_string_list(emap.get("intermediate_mechanisms")),
                ..Edge::minimal(source.clone(), target, "leads_to", NodeType::Phenotype)
            });
        }
    }
}

/// `graph.py:450-473` -- environmental `influences_mechanisms[]` edges,
/// predicate mapped through `ENVIRONMENTAL_EFFECT_PREDICATES`. No
/// `hypothesis_groups`/`intermediate_mechanisms` here (Python never reads
/// those fields in this block).
fn collect_environmental_edges(doc: &Disorder, graph: &mut CausalGraph) {
    for item in as_seq(doc.get("environmental")) {
        let Some(map) = as_map(item) else { continue };
        let Some(source) = truthy_str(map, "name") else {
            continue;
        };
        for link in as_seq(map.get("influences_mechanisms")) {
            let Some(lmap) = as_map(link) else { continue };
            let Some(target) = lmap.get("target").and_then(value_to_string) else {
                continue;
            };
            let predicate =
                environmental_predicate(lmap.get("environmental_effect").and_then(Value::as_str));
            graph.edges.push(Edge {
                description: lmap
                    .get("description")
                    .and_then(Value::as_str)
                    .map(str::to_string),
                causal_link_type: lmap
                    .get("causal_link_type")
                    .and_then(Value::as_str)
                    .map(str::to_string),
                ..Edge::minimal(source.clone(), target, predicate, NodeType::Environmental)
            });
        }
    }
}

/// `graph.py:475-507` -- treatments produce TWO independent edge kinds:
/// `target_mechanisms[]` -> `targets` (bare: no description, no other
/// field -- `graph.py:483-492` populates nothing else), and
/// `target_phenotypes[]` -> `treats`, resolved by name/term against the
/// phenotype lookup (`graph.py:494-507`).
fn collect_treatment_edges(
    doc: &Disorder,
    graph: &mut CausalGraph,
    phenotype_lookup: &HashMap<String, String>,
) {
    for item in as_seq(doc.get("treatments")) {
        let Some(map) = as_map(item) else { continue };
        let Some(source) = truthy_str(map, "name") else {
            continue;
        };
        for target_item in as_seq(map.get("target_mechanisms")) {
            let Some(tmap) = as_map(target_item) else {
                continue;
            };
            let Some(target) = tmap.get("target").and_then(value_to_string) else {
                continue;
            };
            graph.edges.push(Edge::minimal(
                source.clone(),
                target,
                "targets",
                NodeType::Treatment,
            ));
        }
        for descriptor in as_seq(map.get("target_phenotypes")) {
            if as_map(descriptor).is_none() {
                continue;
            }
            let Some(target) = resolve_descriptor_target(descriptor, phenotype_lookup) else {
                continue;
            };
            graph.edges.push(Edge::minimal(
                source.clone(),
                target,
                "treats",
                NodeType::Treatment,
            ));
        }
    }
}

/// Shared body for the three `modeled_mechanisms[]` edge passes
/// (experimental/animal/computational models -- `graph.py:509-576`), which
/// are otherwise byte-identical apart from the section walked and the
/// `source_type`/`source`-derivation.
fn collect_modeled_mechanism_edges(
    items: &[Value],
    graph: &mut CausalGraph,
    source_type: NodeType,
    source_of: impl Fn(&serde_yaml::Mapping) -> Option<String>,
) {
    for item in items {
        let Some(map) = as_map(item) else { continue };
        let Some(source) = source_of(map) else {
            continue;
        };
        for link in as_seq(map.get("modeled_mechanisms")) {
            let Some(lmap) = as_map(link) else { continue };
            let Some(target) = lmap.get("target").and_then(value_to_string) else {
                continue;
            };
            let relationship = lmap
                .get("relationship")
                .and_then(Value::as_str)
                .map(str::to_string);
            let predicate = model_edge_predicate(relationship.as_deref());
            graph.edges.push(Edge {
                description: lmap
                    .get("description")
                    .and_then(Value::as_str)
                    .map(str::to_string),
                relationship,
                fidelity: lmap
                    .get("fidelity")
                    .and_then(Value::as_str)
                    .map(str::to_string),
                limitations: lmap
                    .get("limitations")
                    .and_then(Value::as_str)
                    .map(str::to_string),
                ..Edge::minimal(source.clone(), target, predicate, source_type)
            });
        }
    }
}

/// `graph.py:509-530`.
fn collect_experimental_model_edges(doc: &Disorder, graph: &mut CausalGraph) {
    collect_modeled_mechanism_edges(
        as_seq(doc.get("experimental_models")),
        graph,
        NodeType::ExperimentalModel,
        |m| truthy_str(m, "name"),
    );
}

/// `graph.py:532-553`.
fn collect_animal_model_edges(doc: &Disorder, graph: &mut CausalGraph) {
    collect_modeled_mechanism_edges(
        as_seq(doc.get("animal_models")),
        graph,
        NodeType::AnimalModel,
        animal_model_label,
    );
}

/// `graph.py:555-576`.
fn collect_computational_model_edges(doc: &Disorder, graph: &mut CausalGraph) {
    collect_modeled_mechanism_edges(
        as_seq(doc.get("computational_models")),
        graph,
        NodeType::ComputationalModel,
        |m| truthy_str(m, "name"),
    );
}

/// `graph.py:578-601` -- biochemical `readouts[]`. **Source and target are
/// SWAPPED relative to every other edge pass**: the edge runs FROM the
/// underlying mechanism the readout reports on TO the biomarker itself
/// (`source=str(readout["target"])`, `target=biomarker_name`) -- a
/// biomarker is a downstream observation, not an upstream cause, so the
/// causal arrow points the other way.
fn collect_biochemical_readout_edges(doc: &Disorder, graph: &mut CausalGraph) {
    for item in as_seq(doc.get("biochemical")) {
        let Some(map) = as_map(item) else { continue };
        let Some(biomarker_name) = truthy_str(map, "name") else {
            continue;
        };
        for readout in as_seq(map.get("readouts")) {
            let Some(rmap) = as_map(readout) else {
                continue;
            };
            let Some(source) = rmap.get("target").and_then(value_to_string) else {
                continue;
            };
            let description = rmap
                .get("description")
                .and_then(Value::as_str)
                .or_else(|| rmap.get("interpretation").and_then(Value::as_str))
                .map(str::to_string);
            graph.edges.push(Edge {
                description,
                relationship: rmap
                    .get("relationship")
                    .and_then(Value::as_str)
                    .map(str::to_string),
                direction: rmap
                    .get("direction")
                    .and_then(Value::as_str)
                    .map(str::to_string),
                endpoint_context: rmap
                    .get("endpoint_context")
                    .and_then(Value::as_str)
                    .map(str::to_string),
                ..Edge::minimal(
                    source,
                    biomarker_name.clone(),
                    "readout",
                    NodeType::Biochemical,
                )
            });
        }
    }
}

/// `graph.py:603-627` -- phenotype `reports_on[]`. Same source/target swap
/// as the biochemical readout pass above (`source=str(readout["target"])`,
/// `target=phenotype_name`), `source_type` is `"phenotype"` here.
fn collect_phenotype_reports_on_edges(doc: &Disorder, graph: &mut CausalGraph) {
    for item in as_seq(doc.get("phenotypes")) {
        let Some(map) = as_map(item) else { continue };
        let Some(phenotype_name) = truthy_str(map, "name") else {
            continue;
        };
        for readout in as_seq(map.get("reports_on")) {
            let Some(rmap) = as_map(readout) else {
                continue;
            };
            let Some(source) = rmap.get("target").and_then(value_to_string) else {
                continue;
            };
            let description = rmap
                .get("description")
                .and_then(Value::as_str)
                .or_else(|| rmap.get("interpretation").and_then(Value::as_str))
                .map(str::to_string);
            graph.edges.push(Edge {
                description,
                relationship: rmap
                    .get("relationship")
                    .and_then(Value::as_str)
                    .map(str::to_string),
                direction: rmap
                    .get("direction")
                    .and_then(Value::as_str)
                    .map(str::to_string),
                endpoint_context: rmap
                    .get("endpoint_context")
                    .and_then(Value::as_str)
                    .map(str::to_string),
                ..Edge::minimal(
                    source,
                    phenotype_name.clone(),
                    "readout",
                    NodeType::Phenotype,
                )
            });
        }
    }
}

/// `graph.py:629-651` -- genetic items infer `contributes_to` edges to
/// every pathophysiology node whose gene key matches, gated by
/// [`genetic_item_infers_mechanism_edges`].
fn collect_genetic_contributes_to_edges(
    doc: &Disorder,
    graph: &mut CausalGraph,
    pathophysiology_by_gene_key: &HashMap<String, BTreeSet<String>>,
) {
    for item in as_seq(doc.get("genetic")) {
        let Some(map) = as_map(item) else { continue };
        let Some(source) = truthy_str(map, "name") else {
            continue;
        };
        if !genetic_item_infers_mechanism_edges(map) {
            continue;
        }
        let mut targets: BTreeSet<String> = BTreeSet::new();
        for key in gene_lookup_keys(map, true) {
            if let Some(names) = pathophysiology_by_gene_key.get(&key) {
                targets.extend(names.iter().cloned());
            }
        }
        for target in targets {
            graph.edges.push(Edge::minimal(
                source.clone(),
                target,
                "contributes_to",
                NodeType::Genetic,
            ));
        }
    }
}

/// `graph.py:653-689` -- variant edges. If the variant's gene key(s)
/// resolve to a genetic-section node (or it has a `genetic[]`-nested
/// parent), emit `variant_of` edges to those genetic nodes and STOP (the
/// `continue` at `graph.py:675` means the mechanism-target branch below is
/// never reached for a variant that resolved a genetic target). Otherwise
/// fall back to `contributes_to` edges against matching pathophysiology
/// nodes.
fn collect_variant_edges(
    doc: &Disorder,
    graph: &mut CausalGraph,
    pathophysiology_by_gene_key: &HashMap<String, BTreeSet<String>>,
    genetic_nodes_by_gene_key: &HashMap<String, BTreeSet<String>>,
) {
    for (parent_name, variant) in iter_variant_items(doc) {
        let Some(source) = truthy_str(&variant, "name") else {
            continue;
        };

        let mut genetic_targets: BTreeSet<String> = BTreeSet::new();
        if let Some(parent) = &parent_name {
            genetic_targets.insert(parent.clone());
        }
        for key in gene_lookup_keys(&variant, false) {
            if let Some(names) = genetic_nodes_by_gene_key.get(&key) {
                genetic_targets.extend(names.iter().cloned());
            }
        }

        if !genetic_targets.is_empty() {
            for target in genetic_targets {
                graph.edges.push(Edge::minimal(
                    source.clone(),
                    target,
                    "variant_of",
                    NodeType::Genetic,
                ));
            }
            continue;
        }

        let mut mechanism_targets: BTreeSet<String> = BTreeSet::new();
        for key in gene_lookup_keys(&variant, false) {
            if let Some(names) = pathophysiology_by_gene_key.get(&key) {
                mechanism_targets.extend(names.iter().cloned());
            }
        }
        for target in mechanism_targets {
            graph.edges.push(Edge::minimal(
                source.clone(),
                target,
                "contributes_to",
                NodeType::Genetic,
            ));
        }
    }
}

/// `graph.py:691-701` -- an edge whose source isn't an admitted node is an
/// integrity issue; an edge whose target isn't an admitted node is BOTH an
/// orphan target AND an integrity issue. Neither removes the edge --
/// orphan-target edges stay in `graph.edges`, matching Python exactly (the
/// loop only appends to `orphan_targets`/`integrity_issues`, it never
/// mutates `graph.edges`).
fn check_referential_integrity(graph: &mut CausalGraph) {
    let issues: Vec<String> = graph
        .edges
        .iter()
        .flat_map(|edge| {
            let mut v = Vec::new();
            if !graph.nodes.contains_key(&edge.source) {
                v.push(format!(
                    "Source '{}' (for edge to '{}') not found in named elements",
                    edge.source, edge.target
                ));
            }
            if !graph.nodes.contains_key(&edge.target) {
                v.push(format!(
                    "Target '{}' (from '{}') not found in named elements",
                    edge.target, edge.source
                ));
            }
            v
        })
        .collect();
    for edge in &graph.edges {
        if !graph.nodes.contains_key(&edge.target) {
            graph.orphan_targets.insert(edge.target.clone());
        }
    }
    graph.integrity_issues = issues;
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::model::parse_disorder_raw;

    /// The exact 22q11.2 pathophysiology "Recurrent LCR22-mediated
    /// multigene deletion" downstream block, trimmed to a self-contained
    /// fixture -- verbatim from
    /// `monarch-initiative/dismech kb/disorders/22q11.2_Deletion_Syndrome.yaml`
    /// as read 2026-08-18, not synthesized. Proves: node admission from
    /// `name`, a `downstream[]` edge with `causal_link_type` +
    /// `description`, and a second downstream edge whose target has no
    /// admitted node of its own (an orphan target that must still produce
    /// an edge).
    const REAL_PATHOPHYSIOLOGY_SAMPLE: &str = r#"
pathophysiology:
- name: Recurrent LCR22-mediated multigene deletion
  description: Low-copy repeats at 22q11.2 predispose to recombination.
  downstream:
  - target: TBX1-associated pharyngeal developmental vulnerability
    causal_link_type: DIRECT
    description: The recurrent proximal deletion makes TBX1 hemizygous.
  - target: Candidate cortical and neurotransmitter mechanisms
    causal_link_type: INDIRECT_UNKNOWN_INTERMEDIATES
    description: Several candidate neurodevelopmental mechanisms.
- name: TBX1-associated pharyngeal developmental vulnerability
  description: TBX1 dosage is an important contributor.
  downstream:
  - target: Conotruncal heart defect
    causal_link_type: INDIRECT_KNOWN_INTERMEDIATES
    intermediate_mechanisms:
    - Pharyngeal arch artery, second-heart-field, and outflow-tract development.
    description: TBX1 dosage contributes to conotruncal susceptibility.
phenotypes:
- name: Conotruncal heart defect
  description: Congenital heart disease is heterogeneous.
"#;

    #[test]
    fn pathophysiology_downstream_edges_match_real_corpus_shape() {
        let doc = parse_disorder_raw(REAL_PATHOPHYSIOLOGY_SAMPLE.as_bytes()).unwrap();
        let graph = build_causal_graph(&doc);

        assert_eq!(graph.nodes.len(), 3, "2 pathophysiology + 1 phenotype node");
        assert_eq!(graph.edges.len(), 3, "2 + 1 downstream edges");

        let first = graph
            .edges
            .iter()
            .find(|e| {
                e.source == "Recurrent LCR22-mediated multigene deletion"
                    && e.target.starts_with("TBX1")
            })
            .expect("edge to TBX1 vulnerability");
        assert_eq!(first.predicate, "causes");
        assert_eq!(first.causal_link_type.as_deref(), Some("DIRECT"));
        assert!(first.hypothesis_groups.is_empty());

        // "Candidate cortical and neurotransmitter mechanisms" is a real
        // downstream target with no admitted node in this trimmed fixture
        // -- proving orphan targets still produce edges, never dropped.
        assert!(graph
            .orphan_targets
            .contains("Candidate cortical and neurotransmitter mechanisms"));
        assert!(graph
            .integrity_issues
            .iter()
            .any(|s| s.contains("Candidate cortical and neurotransmitter mechanisms")));

        let intermediate = graph
            .edges
            .iter()
            .find(|e| e.target == "Conotruncal heart defect")
            .expect("edge to Conotruncal heart defect");
        assert_eq!(
            intermediate.intermediate_mechanisms,
            vec![
                "Pharyngeal arch artery, second-heart-field, and outflow-tract development."
                    .to_string()
            ]
        );
    }

    /// Real biochemical `readouts[]` shape (trimmed from
    /// `2-Methylbutyryl-CoA_Dehydrogenase_Deficiency.yaml`) -- proves the
    /// source/target SWAP: the edge runs from the pathophysiology target
    /// TO the biomarker, not the other way.
    #[test]
    fn biochemical_readout_edges_run_from_mechanism_to_biomarker() {
        let yaml = r#"
pathophysiology:
- name: Impaired isoleucine catabolism via SBCAD loss-of-function
  description: Loss of SBCAD activity blocks isoleucine catabolism.
biochemical:
- name: C5-acylcarnitine signal in blood
  presence: INCREASED
  readouts:
  - target: Impaired isoleucine catabolism via SBCAD loss-of-function
    relationship: READOUT_OF
    direction: POSITIVE
    endpoint_context: DIAGNOSTIC
    interpretation: 2-methylbutyrylcarnitine contributes to the unresolved C5 signal.
"#;
        let doc = parse_disorder_raw(yaml.as_bytes()).unwrap();
        let graph = build_causal_graph(&doc);

        let edge = graph
            .edges
            .iter()
            .find(|e| e.predicate == "readout")
            .expect("one readout edge");
        assert_eq!(
            edge.source,
            "Impaired isoleucine catabolism via SBCAD loss-of-function"
        );
        assert_eq!(edge.target, "C5-acylcarnitine signal in blood");
        assert_eq!(edge.relationship.as_deref(), Some("READOUT_OF"));
        assert_eq!(edge.direction.as_deref(), Some("POSITIVE"));
        // description falls back to `interpretation` when `description` is absent.
        assert!(edge
            .description
            .as_deref()
            .unwrap()
            .starts_with("2-methylbutyrylcarnitine"));
    }

    /// Real treatment shape: `target_mechanisms[]` (bare `targets` edge,
    /// no description) + `target_phenotypes[]` (resolved by descriptor
    /// against the phenotype lookup, `treats` edge) -- both from the same
    /// treatment item, trimmed from `22q11.2_Deletion_Syndrome.yaml`.
    #[test]
    fn treatment_edges_cover_both_target_mechanisms_and_target_phenotypes() {
        let yaml = r#"
phenotypes:
- name: Conotruncal heart defect
  phenotype_term:
    preferred_term: Conotruncal defect
    term:
      id: HP:0001710
      label: Conotruncal defect
pathophysiology:
- name: Thymic developmental impairment
  description: Reduced thymic tissue.
treatments:
- name: Cardiovascular evaluation and lesion-directed intervention
  action_category: THERAPEUTIC
  description: Congenital cardiac lesions require cardiology assessment.
  target_mechanisms:
  - target: Thymic developmental impairment
  target_phenotypes:
  - preferred_term: Conotruncal defect
    term:
      id: HP:0001710
      label: Conotruncal defect
"#;
        let doc = parse_disorder_raw(yaml.as_bytes()).unwrap();
        let graph = build_causal_graph(&doc);

        let targets_edge = graph
            .edges
            .iter()
            .find(|e| e.predicate == "targets")
            .expect("one targets edge");
        assert_eq!(targets_edge.target, "Thymic developmental impairment");
        assert!(
            targets_edge.description.is_none(),
            "target_mechanisms edges carry no description"
        );

        let treats_edge = graph
            .edges
            .iter()
            .find(|e| e.predicate == "treats")
            .expect("one treats edge, resolved via the phenotype descriptor");
        assert_eq!(treats_edge.target, "Conotruncal heart defect");
    }

    /// A genetic item whose `association` contains a non-contributing word
    /// ("modifier") must NOT infer a `contributes_to` edge, even when its
    /// gene key matches a real pathophysiology node -- proves
    /// [`genetic_item_infers_mechanism_edges`]'s gate is honored.
    #[test]
    fn a_non_contributing_genetic_association_infers_no_edge() {
        let yaml = r#"
pathophysiology:
- name: Some mechanism
  gene_term:
    preferred_term: FOO
    term:
      id: hgnc:1
      label: FOO
genetic:
- name: FOO modifier variant
  association: modifier of disease severity
  gene_term:
    preferred_term: FOO
    term:
      id: hgnc:1
      label: FOO
"#;
        let doc = parse_disorder_raw(yaml.as_bytes()).unwrap();
        let graph = build_causal_graph(&doc);
        assert!(
            graph.edges.iter().all(|e| e.predicate != "contributes_to"),
            "a MODIFIER-flavored association must not infer a contributes_to edge"
        );
    }

    /// The mirror case: a plain causal association DOES infer the edge.
    #[test]
    fn a_contributing_genetic_association_infers_an_edge() {
        let yaml = r#"
pathophysiology:
- name: Some mechanism
  gene_term:
    preferred_term: FOO
    term:
      id: hgnc:1
      label: FOO
genetic:
- name: FOO causal variant
  association: Causal loss-of-function variant
  gene_term:
    preferred_term: FOO
    term:
      id: hgnc:1
      label: FOO
"#;
        let doc = parse_disorder_raw(yaml.as_bytes()).unwrap();
        let graph = build_causal_graph(&doc);
        let edge = graph
            .edges
            .iter()
            .find(|e| e.predicate == "contributes_to")
            .expect("a causal association infers a contributes_to edge");
        assert_eq!(edge.source, "FOO causal variant");
        assert_eq!(edge.target, "Some mechanism");
    }

    /// A variant nested under a `genetic[]` item resolves a `variant_of`
    /// edge to its parent's name, and (per the Python `continue`) does
    /// NOT also emit a `contributes_to` mechanism edge.
    #[test]
    fn a_nested_variant_emits_variant_of_and_nothing_else() {
        let yaml = r#"
genetic:
- name: FOO gene defect
  gene_term:
    preferred_term: FOO
    term:
      id: hgnc:1
      label: FOO
  variants:
  - name: c.123A>G
    description: A missense variant.
"#;
        let doc = parse_disorder_raw(yaml.as_bytes()).unwrap();
        let graph = build_causal_graph(&doc);
        let variant_edges: Vec<&Edge> = graph
            .edges
            .iter()
            .filter(|e| e.source == "c.123A>G")
            .collect();
        assert_eq!(variant_edges.len(), 1);
        assert_eq!(variant_edges[0].predicate, "variant_of");
        assert_eq!(variant_edges[0].target, "FOO gene defect");
    }

    /// An animal model with NO `name` field falls back to the
    /// `genotype`/`species` label -- proves [`animal_model_label`]'s
    /// fallback, and that a labeled-but-mechanism-free animal model still
    /// becomes a node (per the "code, not the stale comment" ruling).
    #[test]
    fn an_unnamed_animal_model_falls_back_to_genotype_species_label() {
        let yaml = r#"
animal_models:
- species: Mus musculus
  genotype: Tbx1 heterozygous knockout
  description: Models cardiac outflow tract anomalies.
"#;
        let doc = parse_disorder_raw(yaml.as_bytes()).unwrap();
        let graph = build_causal_graph(&doc);
        assert!(graph
            .nodes
            .contains_key("Tbx1 heterozygous knockout Mus musculus"));
    }

    /// Anti-vacuity: an entirely empty document must resolve to zero
    /// nodes and zero edges, never panic.
    #[test]
    fn an_empty_document_yields_an_empty_graph() {
        let doc = parse_disorder_raw(b"").unwrap();
        let graph = build_causal_graph(&doc);
        assert!(graph.nodes.is_empty());
        assert!(graph.edges.is_empty());
    }

    #[test]
    fn coerce_string_list_accepts_scalar_and_list_and_none() {
        assert_eq!(coerce_string_list(None), Vec::<String>::new());
        assert_eq!(
            coerce_string_list(Some(&Value::String("solo".into()))),
            vec!["solo".to_string()]
        );
        let seq = Value::Sequence(vec![Value::String("a".into()), Value::String("b".into())]);
        assert_eq!(
            coerce_string_list(Some(&seq)),
            vec!["a".to_string(), "b".to_string()]
        );
    }
}
