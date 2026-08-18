//! Permissive YAML model for a `monarch-initiative/dismech` disorder file.
//!
//! Two layers live here, additively:
//!
//! 1. **Disorder identity** (`DisorderIdentity`) -- `name` / `description`
//!    / `category` / the MONDO xref. This crate's original vertical slice.
//! 2. **The causal-mechanism-graph node shape** (`NodeItem` and its edge
//!    raw types) -- a permissive, untyped-first model of one entry in any
//!    of the 8 section lists (`pathophysiology[]`, `phenotypes[]`, ...).
//!    This layer exists for structural/typed access to a single section
//!    item; the actual `build_causal_graph` resolver (`graph.rs`) does
//!    NOT consume it -- it walks raw `serde_yaml::Value` directly, because
//!    several of the real resolver's matching rules are inherently dynamic
//!    (see `graph.rs`'s module doc comment for why). `NodeItem` is kept
//!    here as an additive, independently useful typed view -- e.g. for a
//!    future `pack.rs` extension that wants one section item's fields
//!    without re-deriving this parse.
//!
//! `monarch-initiative/dismech` (the real upstream; the `AdaWorldAPI/dismech`
//! fork's default branch is STALE and carries no `kb/` corpus at all --
//! confirmed 2026-08-17, do not clone the fork) DOES ship a real Python
//! resolver application (`src/dismech/graph.py::build_causal_graph`,
//! falsified against 1,903 committed `pathographs/MONDO_*.json` oracle
//! files) -- an earlier "no separate resolver application, tooling only"
//! claim here and in EPIPHANIES.md was wrong, based on an incomplete look
//! at the upstream tree; corrected 2026-08-18, see
//! `.claude/board/EPIPHANIES.md`.

use std::collections::BTreeMap;

use serde::{Deserialize, Deserializer};
use serde_yaml::Value;

/// A raw disorder document. Untyped at the top level, mirroring the corpus's
/// own shape -- unknown keys tolerated, since this slice only reads a few
/// fields and must not choke on the rest of the schema.
pub type Disorder = BTreeMap<String, Value>;

/// `disease_term: {preferred_term, term: {id, label}}` -- the CURIE-bearing
/// identity wrapper every disorder file carries. `preferred_term` is a
/// sibling of `term`, present on every `*_term` wrapper in the real corpus
/// (`disease_term`, `phenotype_term`, `biomarker_term`, `exposure_term`,
/// `treatment_term`) -- verified across `kb/disorders/` 2026-08-18.
#[derive(Debug, Clone, Deserialize, Default)]
pub struct TermRef {
    #[serde(default)]
    pub preferred_term: Option<String>,
    pub term: Option<InnerTerm>,
}

#[derive(Debug, Clone, Deserialize)]
pub struct InnerTerm {
    pub id: Option<String>,
    pub label: Option<String>,
}

impl TermRef {
    #[must_use]
    pub fn curie(&self) -> Option<&str> {
        self.term.as_ref()?.id.as_deref()
    }
}

/// The identity-slice fields this bake actually reads.
#[derive(Debug, Clone, Deserialize, Default)]
pub struct DisorderIdentity {
    pub name: Option<String>,
    #[serde(default)]
    pub category: Option<String>,
    #[serde(default)]
    pub description: Option<String>,
    #[serde(default)]
    pub disease_term: Option<TermRef>,
}

/// Parse one disorder YAML file leniently into the identity slice. A
/// `null`/empty document becomes an empty map, mirroring the corpus's own
/// tolerant load posture.
///
/// # Errors
/// Returns the underlying `serde_yaml` error on genuinely malformed YAML
/// (not on missing/extra fields -- those are handled leniently).
pub fn parse_disorder(bytes: &[u8]) -> Result<DisorderIdentity, serde_yaml::Error> {
    let v: Option<DisorderIdentity> = serde_yaml::from_slice(bytes)?;
    Ok(v.unwrap_or_default())
}

/// Parse one disorder YAML file into the raw, untyped [`Disorder`] map --
/// what `graph.rs`'s `build_causal_graph` consumes. A `null`/empty document
/// becomes an empty map, same tolerant posture as [`parse_disorder`].
///
/// # Errors
/// Returns the underlying `serde_yaml` error on genuinely malformed YAML.
pub fn parse_disorder_raw(bytes: &[u8]) -> Result<Disorder, serde_yaml::Error> {
    let v: Option<Disorder> = serde_yaml::from_slice(bytes)?;
    Ok(v.unwrap_or_default())
}

/// Accepts `null` (-> empty), a bare scalar (-> one-element list), or a
/// list (-> stringified non-blank items) where the corpus schema nominally
/// declares a list. **The gotcha this exists for:** some section fields
/// are authored as a scalar in the real corpus even though the field is
/// schema'd as a list elsewhere -- verified 2026-08-18 against real
/// `kb/disorders/*.yaml` data: `subtypes` appears BOTH as a block list
/// (`subtypes:\n- Infantile\n- Juvenile`) AND as a bare scalar
/// (`subtypes: PITX2-related ARS (RIEG1) and FOXC1-related ARS (RIEG3)...`)
/// in different files. A naive `Vec<String>` deserialization hard-fails on
/// the scalar form, silently dropping the WHOLE containing node (the same
/// failure mode `bake.rs`'s honest `parse_errors` counter exists to catch,
/// but at the level of one field rather than one file). Applied to
/// `conforms_to`, `hypothesis_groups`, `intermediate_mechanisms`, and
/// `subtypes` below -- `conforms_to`/`intermediate_mechanisms` were not
/// observed as scalars in this snapshot (always a quoted string in the
/// `conforms_to` case, always a block list in the `intermediate_mechanisms`
/// case), but accepting both costs nothing and the corpus is hand-curated
/// prose, not machine-generated -- a future edit reverting to the scalar
/// form for either field is exactly the kind of drift this guards against.
fn de_string_or_list<'de, D>(deserializer: D) -> Result<Vec<String>, D::Error>
where
    D: Deserializer<'de>,
{
    #[derive(Deserialize)]
    #[serde(untagged)]
    enum ScalarOrList {
        Null,
        One(String),
        Many(Vec<String>),
    }
    Ok(match Option::<ScalarOrList>::deserialize(deserializer)? {
        None | Some(ScalarOrList::Null) => Vec::new(),
        Some(ScalarOrList::One(s)) => {
            if s.trim().is_empty() {
                Vec::new()
            } else {
                vec![s]
            }
        }
        Some(ScalarOrList::Many(v)) => v.into_iter().filter(|s| !s.trim().is_empty()).collect(),
    })
}

/// A causal edge entry as it appears in `downstream[]` / `sequelae[]` --
/// `{target, description, evidence, hypothesis_groups, causal_link_type,
/// intermediate_mechanisms}`. Permissive: a list item missing `target` is
/// still parseable (as `None`), the caller decides whether to skip it
/// (mirrors upstream `graph.py:330-331`'s "skip, don't error" posture).
#[derive(Debug, Clone, Deserialize, Default)]
pub struct CausalEdgeRaw {
    pub target: Option<String>,
    #[serde(default)]
    pub description: Option<String>,
    #[serde(default)]
    pub evidence: Vec<EvidenceItemRaw>,
    #[serde(default, deserialize_with = "de_string_or_list")]
    pub hypothesis_groups: Vec<String>,
    #[serde(default)]
    pub causal_link_type: Option<String>,
    #[serde(default, deserialize_with = "de_string_or_list")]
    pub intermediate_mechanisms: Vec<String>,
}

/// `influences_mechanisms[]` -- `{target, environmental_effect,
/// causal_link_type, description, evidence}`.
#[derive(Debug, Clone, Deserialize, Default)]
pub struct EnvironmentalEdgeRaw {
    pub target: Option<String>,
    #[serde(default)]
    pub environmental_effect: Option<String>,
    #[serde(default)]
    pub causal_link_type: Option<String>,
    #[serde(default)]
    pub description: Option<String>,
    #[serde(default)]
    pub evidence: Vec<EvidenceItemRaw>,
}

/// `target_mechanisms[]` -- `{target, treatment_effect, description,
/// evidence}`.
#[derive(Debug, Clone, Deserialize, Default)]
pub struct TargetEdgeRaw {
    pub target: Option<String>,
    #[serde(default)]
    pub treatment_effect: Option<String>,
    #[serde(default)]
    pub description: Option<String>,
    #[serde(default)]
    pub evidence: Vec<EvidenceItemRaw>,
}

/// `readouts[]` / `reports_on[]` -- `{target, relationship, direction,
/// endpoint_context, description/interpretation, evidence}`. Both extra
/// fields (`endpoint_context`, `interpretation`) are real corpus fields
/// beyond the brief 4-field shape, verified 2026-08-18 against
/// `2-Methylbutyryl-CoA_Dehydrogenase_Deficiency.yaml`.
#[derive(Debug, Clone, Deserialize, Default)]
pub struct ReadoutEdgeRaw {
    pub target: Option<String>,
    #[serde(default)]
    pub relationship: Option<String>,
    #[serde(default)]
    pub direction: Option<String>,
    #[serde(default)]
    pub endpoint_context: Option<String>,
    #[serde(default)]
    pub description: Option<String>,
    #[serde(default)]
    pub interpretation: Option<String>,
    #[serde(default)]
    pub evidence: Vec<EvidenceItemRaw>,
}

/// `modeled_mechanisms[]` -- `{target, relationship, fidelity, limitations,
/// description, evidence}`.
#[derive(Debug, Clone, Deserialize, Default)]
pub struct ModelEdgeRaw {
    pub target: Option<String>,
    #[serde(default)]
    pub relationship: Option<String>,
    #[serde(default)]
    pub fidelity: Option<String>,
    #[serde(default)]
    pub limitations: Option<String>,
    #[serde(default)]
    pub description: Option<String>,
    #[serde(default)]
    pub evidence: Vec<EvidenceItemRaw>,
}

/// One `evidence[]` entry -- the `{reference, reference_title, supports,
/// evidence_source, snippet, explanation}` shape carried at BOTH node
/// level and edge level throughout the corpus.
#[derive(Debug, Clone, Deserialize, Default)]
pub struct EvidenceItemRaw {
    #[serde(default)]
    pub reference: Option<String>,
    #[serde(default)]
    pub reference_title: Option<String>,
    #[serde(default)]
    pub supports: Option<String>,
    #[serde(default)]
    pub evidence_source: Option<String>,
    #[serde(default)]
    pub snippet: Option<String>,
    #[serde(default)]
    pub explanation: Option<String>,
}

/// A permissive, untyped-first entry from any of the 8 causal-mechanism
/// section lists (`pathophysiology[]`, `phenotypes[]`, `environmental[]`,
/// `genetic[]`, `treatments[]`, `biochemical[]`, `experimental_models[]`,
/// `computational_models[]`). Every field is optional/defaulted so a
/// section-specific field absent from a given section's items (e.g.
/// `phenotype_term` on a `treatments[]` entry) never fails the parse --
/// mirrors upstream `graph.py:330-331`'s own "skip if not a dict, skip if
/// no name" leniency at the list level; `#[serde(flatten)] other` catches
/// everything this struct doesn't name so nothing is silently discarded.
#[derive(Debug, Clone, Deserialize, Default)]
pub struct NodeItem {
    pub name: Option<String>,
    #[serde(default)]
    pub description: Option<String>,

    // causal edge lists:
    #[serde(default)]
    pub downstream: Vec<CausalEdgeRaw>,
    #[serde(default)]
    pub sequelae: Vec<CausalEdgeRaw>,
    #[serde(default)]
    pub influences_mechanisms: Vec<EnvironmentalEdgeRaw>,
    #[serde(default)]
    pub target_mechanisms: Vec<TargetEdgeRaw>,
    #[serde(default)]
    pub readouts: Vec<ReadoutEdgeRaw>,
    #[serde(default)]
    pub reports_on: Vec<ReadoutEdgeRaw>,
    #[serde(default)]
    pub modeled_mechanisms: Vec<ModelEdgeRaw>,

    // identity/grounding:
    #[serde(default, deserialize_with = "de_string_or_list")]
    pub conforms_to: Vec<String>,
    #[serde(default)]
    pub phenotype_term: Option<TermRef>,
    #[serde(default)]
    pub disease_term: Option<TermRef>,
    #[serde(default)]
    pub biomarker_term: Option<TermRef>,
    #[serde(default)]
    pub treatment_term: Option<TermRef>,
    #[serde(default)]
    pub exposure_term: Option<TermRef>,
    /// Raw, not `TermRef` -- `gene_term` occasionally carries extra
    /// sibling fields (`modifier`, `description`) beyond `preferred_term`/
    /// `term`, verified against a corpus-wide scan 2026-08-18 (3,686
    /// instances: `preferred_term` 3686, `term` 3685, `modifier` 7,
    /// `description` 6). Keeping it raw preserves those without widening
    /// `TermRef` for a handful of occurrences.
    #[serde(default)]
    pub gene_term: Option<Value>,
    #[serde(default)]
    pub preferred_term: Option<String>,

    // gene-key inference inputs (see `graph.rs::gene_lookup_keys`):
    #[serde(default)]
    pub gene: Option<Value>,
    #[serde(default)]
    pub genes: Vec<Value>,
    #[serde(default)]
    pub relationship_type: Option<String>,
    #[serde(default)]
    pub association: Option<String>,
    /// Not observed anywhere in the real corpus as of 2026-08-18 (zero
    /// `variant_of:` occurrences across `kb/disorders/`) -- kept for
    /// forward compatibility with the upstream schema, never populated by
    /// current data.
    #[serde(default)]
    pub variant_of: Option<String>,

    // metadata carried verbatim, never acted on:
    #[serde(default)]
    pub mechanism_confidence: Option<String>,
    #[serde(default)]
    pub biological_scale: Option<String>,
    #[serde(default)]
    pub subtype: Option<String>,
    #[serde(default, deserialize_with = "de_string_or_list")]
    pub subtypes: Vec<String>,
    #[serde(default)]
    pub evidence: Vec<EvidenceItemRaw>,

    #[serde(flatten)]
    pub other: BTreeMap<String, Value>,
}

/// Parse one section-list entry leniently. A non-map item, or a map
/// without `name`, is not a hard error here (the caller decides admission
/// -- mirrors `graph.py`'s own list-item leniency); this only fails on
/// genuinely malformed YAML for the item itself.
///
/// # Errors
/// Returns the underlying `serde_yaml` error on malformed YAML.
pub fn parse_node_item(value: Value) -> Result<Option<NodeItem>, serde_yaml::Error> {
    if value.as_mapping().is_none() {
        return Ok(None);
    }
    serde_yaml::from_value(value).map(Some)
}

#[cfg(test)]
mod tests {
    use super::*;

    /// A real corpus shape (22q11.2 Deletion Syndrome, trimmed to the
    /// identity-slice fields), byte-for-byte as read from
    /// `monarch-initiative/dismech kb/disorders/22q11.2_Deletion_Syndrome.yaml`
    /// on 2026-08-17 -- not a synthetic fixture invented for the test.
    const REAL_SAMPLE: &str = r#"
name: 22q11.2 Deletion Syndrome
creation_date: '2026-02-06T03:39:54Z'
category: Genetic
synonyms:
- DiGeorge syndrome
- Velocardiofacial syndrome
description: >-
  22q11.2 deletion syndrome is a variably expressive chromosomal disorder.
disease_term:
  preferred_term: 22q11.2 deletion syndrome
  term:
    id: MONDO:0018923
    label: 22q11.2 deletion syndrome
"#;

    #[test]
    fn parses_the_real_corpus_shape() {
        let d = parse_disorder(REAL_SAMPLE.as_bytes()).expect("valid yaml");
        assert_eq!(d.name.as_deref(), Some("22q11.2 Deletion Syndrome"));
        assert_eq!(d.category.as_deref(), Some("Genetic"));
        assert!(d.description.as_deref().unwrap().starts_with("22q11.2"));
        assert_eq!(
            d.disease_term.as_ref().and_then(TermRef::curie),
            Some("MONDO:0018923")
        );
    }

    /// Anti-vacuity: a document missing the MONDO term must read back as
    /// `None`, never a fabricated address -- this is the honest-gap path
    /// `bake.rs` counts as "unresolved", not silently addressed at 0.
    #[test]
    fn a_missing_disease_term_is_none_not_fabricated() {
        let d = parse_disorder(b"name: Something Without A Mondo Id\n").unwrap();
        assert_eq!(d.disease_term.as_ref().and_then(TermRef::curie), None);
    }

    /// Malformed input is a real parse error, not silently swallowed to a
    /// default -- distinguishes "file doesn't have the field" (fine) from
    /// "file is broken YAML" (must be surfaced).
    #[test]
    fn malformed_yaml_is_a_real_error() {
        assert!(parse_disorder(b"name: [unterminated\n").is_err());
    }

    /// A null/empty document is tolerated, mirroring the corpus's own
    /// lenient load posture -- never a panic on an empty file.
    #[test]
    fn an_empty_document_is_tolerated() {
        let d = parse_disorder(b"").unwrap();
        assert_eq!(d.name, None);
    }
}
