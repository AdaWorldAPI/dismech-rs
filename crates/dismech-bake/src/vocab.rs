//! `vocab` — the source vocabularies as **data**, not as Rust literals.
//!
//! # Why this module exists
//!
//! The DisMech corpus is LLM-generated YAML. Measured over the real corpus
//! (2,100 files, 2026-08-20), its string-valued fields split into two very
//! different populations:
//!
//! | field | distinct values | shape |
//! |---|---:|---|
//! | `causal_link_type` | 4 | CLOSED — corpus == table |
//! | `environmental_effect` | 5 | CLOSED — corpus == table |
//! | `relationship` | 12 | OPEN — 6 outside the ported table |
//! | `supports` | 5 | OPEN — `WRONG_STATEMENT` outside the ported 4 |
//! | `association` | **1,309** | unbounded (32% of occurrences unique) |
//! | `category` | **440** | unbounded (`Neurologic` 2,457 / `Neurological` 1,567) |
//! | `assertion_type` | 28 | unbounded (`structured_disease_record` 99 / `Structured disease record` 12) |
//!
//! A hand-written `match` over an LLM-authored vocabulary is a silent
//! misclassifier the moment the generator emits a synonym. So the tables live
//! in `../data/*.tsv` and this module is the one reusable parser over them —
//! changing a vocabulary is a data edit, never a code edit.
//!
//! # What this module does NOT change
//!
//! **Transcode parity is preserved exactly.** Upstream `graph.py` fails OPEN
//! (`_ => "influences"`, `_ => "models"`), and so does this crate — an
//! unrecognized token still resolves to the same fallback predicate Python
//! would emit. This module does not alter a single emitted predicate.
//!
//! What it adds is VISIBILITY: [`unknown_tokens`] scans a document and names
//! every value that fell through, so the noise is reported through
//! `CausalGraph::integrity_issues` instead of being absorbed in silence. The
//! defect being fixed is silence, not the fallback.
//!
//! The one genuinely closed vocabulary, `causal_link_type`, gets a typed
//! [`CausalLinkType`] with a fail-CLOSED parser — because there a fallback
//! would be an invention, and `UNKNOWN` is an *asserted source value*, never a
//! parse failure. That distinction is load-bearing downstream: bits 59..60 of
//! `CausalEdge64` are source-authoritative.

use std::collections::BTreeMap;

/// One `(source_token, mapped_value)` table, parsed from a TSV.
///
/// The reusable pattern the data-as-config doctrine asks for: one parser, N
/// data files. Comment lines (`#`) and blank lines are skipped; every other
/// line must have at least two tab-separated columns or it is a hard error —
/// a malformed table is a build-time defect, never a silently short table.
#[derive(Debug, Clone)]
pub struct VocabTable {
    /// `source_token` (as written in the YAML) → the mapped value.
    entries: BTreeMap<String, String>,
    /// Extra columns beyond the second, keyed by source token.
    extra: BTreeMap<String, Vec<String>>,
    /// Whether lookups uppercase the probe first (upstream `model_edge_predicate` does).
    case_folded: bool,
}

/// A malformed vocabulary table.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct VocabErr {
    /// 1-based line number in the source TSV.
    pub line: usize,
    /// What was wrong.
    pub message: String,
}

impl core::fmt::Display for VocabErr {
    fn fmt(&self, f: &mut core::fmt::Formatter<'_>) -> core::fmt::Result {
        write!(f, "vocab table line {}: {}", self.line, self.message)
    }
}

impl std::error::Error for VocabErr {}

impl VocabTable {
    /// Parse a TSV table. `case_folded` uppercases both stored tokens and
    /// probes, matching upstream's `relationship.map(str::to_uppercase)`.
    ///
    /// # Errors
    /// Returns [`VocabErr`] on any non-comment line with fewer than two columns.
    pub fn parse(text: &str, case_folded: bool) -> Result<Self, VocabErr> {
        let mut entries = BTreeMap::new();
        let mut extra = BTreeMap::new();
        for (i, raw) in text.lines().enumerate() {
            let line = raw.trim_end_matches(['\r', '\n']);
            if line.trim().is_empty() || line.trim_start().starts_with('#') {
                continue;
            }
            let mut cols = line.split('\t');
            let token = cols.next().unwrap_or_default().trim();
            let Some(mapped) = cols.next().map(str::trim) else {
                return Err(VocabErr {
                    line: i + 1,
                    message: format!("expected at least 2 tab-separated columns, got: {line:?}"),
                });
            };
            if token.is_empty() || mapped.is_empty() {
                return Err(VocabErr {
                    line: i + 1,
                    message: "empty token or mapped value".to_string(),
                });
            }
            let key = if case_folded {
                token.to_uppercase()
            } else {
                token.to_string()
            };
            let rest: Vec<String> = cols.map(|c| c.trim().to_string()).collect();
            if !rest.is_empty() {
                extra.insert(key.clone(), rest);
            }
            entries.insert(key, mapped.to_string());
        }
        Ok(Self {
            entries,
            extra,
            case_folded,
        })
    }

    /// The mapped value for a source token, or `None` if the token is not in
    /// the table. Callers decide what a miss means — this type never invents
    /// a fallback.
    #[must_use]
    pub fn get(&self, token: &str) -> Option<&str> {
        let probe = if self.case_folded {
            token.to_uppercase()
        } else {
            token.to_string()
        };
        self.entries.get(&probe).map(String::as_str)
    }

    /// A third-or-later column for a token (e.g. the `bits2` column).
    #[must_use]
    pub fn extra(&self, token: &str, idx: usize) -> Option<&str> {
        let probe = if self.case_folded {
            token.to_uppercase()
        } else {
            token.to_string()
        };
        self.extra
            .get(&probe)
            .and_then(|v| v.get(idx))
            .map(String::as_str)
    }

    /// Number of rows.
    #[must_use]
    pub fn len(&self) -> usize {
        self.entries.len()
    }

    /// Whether the table has no rows.
    #[must_use]
    pub fn is_empty(&self) -> bool {
        self.entries.is_empty()
    }

    /// Every source token, sorted.
    pub fn tokens(&self) -> impl Iterator<Item = &str> {
        self.entries.keys().map(String::as_str)
    }
}

/// The `causal_link_type.tsv` table text, committed alongside the code.
pub const CAUSAL_LINK_TYPE_TSV: &str = include_str!("../data/causal_link_type.tsv");
/// The `environmental_effect.tsv` table text.
pub const ENVIRONMENTAL_EFFECT_TSV: &str = include_str!("../data/environmental_effect.tsv");
/// The `model_relationship.tsv` table text.
pub const MODEL_RELATIONSHIP_TSV: &str = include_str!("../data/model_relationship.tsv");

/// The four source-authoritative causal topologies.
///
/// This vocabulary is CLOSED — measured corpus-wide as exactly these four
/// values over 17,998 occurrences, with zero unmatched and zero never-seen.
/// Parsing is therefore fail-CLOSED.
///
/// **`Unknown` is an asserted source value, not a parse failure.** A token the
/// source never wrote must never become `Unknown`; that would manufacture an
/// assertion the corpus does not make. [`CausalLinkType::from_source`] returns
/// `None` for an unrecognized token precisely so the two stay distinguishable.
#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub enum CausalLinkType {
    /// `DIRECT` — measured 9,073.
    Direct,
    /// `INDIRECT_KNOWN_INTERMEDIATES` — measured 3,978. The source asserts
    /// indirectness AND claims to know the intermediates (though 1,466 of
    /// these name none).
    IndirectKnownIntermediates,
    /// `INDIRECT_UNKNOWN_INTERMEDIATES` — measured 4,539. The source asserts
    /// indirectness and that the mediator identity is unknown.
    IndirectUnknownIntermediates,
    /// `UNKNOWN` — measured 408. The source asserts that the TOPOLOGY itself
    /// is unknown. Distinct from [`Self::IndirectUnknownIntermediates`]: that
    /// one establishes a mediator ROLE, this one establishes nothing about
    /// direct-vs-indirect.
    Unknown,
}

impl CausalLinkType {
    /// Every variant, in bit order.
    pub const ALL: [Self; 4] = [
        Self::Direct,
        Self::IndirectKnownIntermediates,
        Self::IndirectUnknownIntermediates,
        Self::Unknown,
    ];

    /// Parse a source token, fail-CLOSED.
    ///
    /// Returns `None` for any token not in `causal_link_type.tsv` — including
    /// near-misses and case variants. An unrecognized token is a corpus-drift
    /// signal for the caller to record, NEVER a silent [`Self::Unknown`].
    #[must_use]
    pub fn from_source(token: &str) -> Option<Self> {
        match token.trim() {
            "DIRECT" => Some(Self::Direct),
            "INDIRECT_KNOWN_INTERMEDIATES" => Some(Self::IndirectKnownIntermediates),
            "INDIRECT_UNKNOWN_INTERMEDIATES" => Some(Self::IndirectUnknownIntermediates),
            "UNKNOWN" => Some(Self::Unknown),
            _ => None,
        }
    }

    /// The exact source token this variant came from.
    #[must_use]
    pub const fn as_source(self) -> &'static str {
        match self {
            Self::Direct => "DIRECT",
            Self::IndirectKnownIntermediates => "INDIRECT_KNOWN_INTERMEDIATES",
            Self::IndirectUnknownIntermediates => "INDIRECT_UNKNOWN_INTERMEDIATES",
            Self::Unknown => "UNKNOWN",
        }
    }

    /// The 2-bit encoding a downstream consumer writes into `CausalEdge64`
    /// bits 59..60. Provided as a pure function of the source value;
    /// dismech-rs itself never packs a hot reasoning register.
    #[must_use]
    pub const fn to_bits2(self) -> u8 {
        match self {
            Self::Direct => 0,
            Self::IndirectKnownIntermediates => 1,
            Self::IndirectUnknownIntermediates => 2,
            Self::Unknown => 3,
        }
    }

    /// Inverse of [`Self::to_bits2`]; `None` for a value above 3.
    #[must_use]
    pub const fn from_bits2(b: u8) -> Option<Self> {
        match b {
            0 => Some(Self::Direct),
            1 => Some(Self::IndirectKnownIntermediates),
            2 => Some(Self::IndirectUnknownIntermediates),
            3 => Some(Self::Unknown),
            _ => None,
        }
    }

    /// True only when the source asserted indirectness AND that the mediator
    /// identity is unknown — i.e. `A -> ? -> B` with the mediator ROLE
    /// established.
    ///
    /// Deliberately FALSE for [`Self::Unknown`]: unknown topology does not
    /// license creating a mediator slot.
    #[must_use]
    pub const fn mediator_unresolved(self) -> bool {
        matches!(self, Self::IndirectUnknownIntermediates)
    }

    /// True only when the topology ITSELF is unresolved.
    #[must_use]
    pub const fn topology_unresolved(self) -> bool {
        matches!(self, Self::Unknown)
    }
}

// ---------------------------------------------------------------------
// The visibility pass — report what the fail-open fallbacks absorb.
// ---------------------------------------------------------------------

/// One source value that fell through its vocabulary table.
#[derive(Debug, Clone, PartialEq, Eq, PartialOrd, Ord)]
pub struct UnknownToken {
    /// The YAML key it was read from (`causal_link_type`, `relationship`, …).
    pub field: &'static str,
    /// The value as written in the source.
    pub value: String,
}

impl core::fmt::Display for UnknownToken {
    fn fmt(&self, f: &mut core::fmt::Formatter<'_>) -> core::fmt::Result {
        write!(
            f,
            "Unrecognized {} value '{}' — not in the committed vocabulary table",
            self.field, self.value
        )
    }
}

/// Scan a parsed disorder document for every vocabulary value that is NOT in
/// its committed table.
///
/// This is a REPORTING pass, deliberately separate from edge construction: it
/// changes no predicate and drops no edge, so transcode parity is untouched.
/// `build_causal_graph` prepends the result to `CausalGraph::integrity_issues`
/// so drift surfaces where a reader already looks for defects.
///
/// Values are de-duplicated and sorted, so the output is deterministic and a
/// document repeating one bad token 900 times reports it once.
#[must_use]
pub fn unknown_tokens(doc: &crate::model::Disorder) -> Vec<UnknownToken> {
    let clt = VocabTable::parse(CAUSAL_LINK_TYPE_TSV, false).expect("committed table parses");
    let env = VocabTable::parse(ENVIRONMENTAL_EFFECT_TSV, false).expect("committed table parses");
    let rel = VocabTable::parse(MODEL_RELATIONSHIP_TSV, true).expect("committed table parses");
    let mut found = std::collections::BTreeSet::new();
    for (k, v) in doc {
        scan_pair(k, v, &clt, &env, &rel, &mut found);
        walk(v, &clt, &env, &rel, &mut found);
    }
    found.into_iter().collect()
}

fn scan_pair(
    key: &str,
    v: &serde_yaml::Value,
    clt: &VocabTable,
    env: &VocabTable,
    rel: &VocabTable,
    out: &mut std::collections::BTreeSet<UnknownToken>,
) {
    let Some(val) = v.as_str() else { return };
    let table = match key {
        "causal_link_type" => Some(("causal_link_type", clt)),
        "environmental_effect" => Some(("environmental_effect", env)),
        "relationship" => Some(("relationship", rel)),
        _ => None,
    };
    if let Some((field, t)) = table {
        if t.get(val).is_none() {
            out.insert(UnknownToken {
                field,
                value: val.to_string(),
            });
        }
    }
}

fn walk(
    node: &serde_yaml::Value,
    clt: &VocabTable,
    env: &VocabTable,
    rel: &VocabTable,
    out: &mut std::collections::BTreeSet<UnknownToken>,
) {
    match node {
        serde_yaml::Value::Mapping(m) => {
            for (k, v) in m {
                if let Some(key) = k.as_str() {
                    scan_pair(key, v, clt, env, rel, out);
                }
                walk(v, clt, env, rel, out);
            }
        }
        serde_yaml::Value::Sequence(s) => {
            for it in s {
                walk(it, clt, env, rel, out);
            }
        }
        _ => {}
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    /// Every committed table parses and is NON-EMPTY. Anti-vacuity: a table
    /// that silently parsed to zero rows would make every `get` miss and every
    /// drift test pass for the wrong reason.
    #[test]
    fn committed_tables_parse_and_are_non_empty() {
        let clt = VocabTable::parse(CAUSAL_LINK_TYPE_TSV, false).unwrap();
        let env = VocabTable::parse(ENVIRONMENTAL_EFFECT_TSV, false).unwrap();
        let rel = VocabTable::parse(MODEL_RELATIONSHIP_TSV, true).unwrap();
        assert_eq!(
            clt.len(),
            4,
            "causal_link_type is a closed 4-value vocabulary"
        );
        assert_eq!(
            env.len(),
            5,
            "environmental_effect measured 5 distinct values"
        );
        assert_eq!(
            rel.len(),
            6,
            "model_relationship ports 6 of the 12 corpus values"
        );
    }

    /// The tables are DATA, not literals: a table parsed from a different
    /// string yields different lookups through the SAME code path. Proves the
    /// parser is actually consulted rather than the values being inlined.
    #[test]
    fn the_table_is_data_so_a_different_table_gives_a_different_answer() {
        let real = VocabTable::parse(ENVIRONMENTAL_EFFECT_TSV, false).unwrap();
        assert_eq!(real.get("TRIGGERS"), Some("triggers"));
        let swapped = VocabTable::parse("TRIGGERS\tsomething_else\n", false).unwrap();
        assert_eq!(swapped.get("TRIGGERS"), Some("something_else"));
        assert_ne!(real.get("TRIGGERS"), swapped.get("TRIGGERS"));
    }

    /// A malformed table is a hard error with a line number, never a short table.
    #[test]
    fn a_one_column_line_is_an_error_not_a_skipped_row() {
        let err = VocabTable::parse("GOOD\tmapped\nBROKEN_NO_TAB\n", false).unwrap_err();
        assert_eq!(err.line, 2);
        assert!(
            err.message.contains("2 tab-separated columns"),
            "{}",
            err.message
        );
    }

    /// Comments and blanks are skipped, and the committed tables lead with a
    /// comment block — so this is exercised by real data, not only a fixture.
    #[test]
    fn comments_and_blank_lines_are_skipped() {
        let t = VocabTable::parse("# header\n\nA\tb\n\n# trailing\nC\td\n", false).unwrap();
        assert_eq!(t.len(), 2);
        assert_eq!(t.get("A"), Some("b"));
        assert_eq!(t.get("C"), Some("d"));
    }

    /// `model_relationship` matches case-insensitively (upstream uppercases);
    /// `causal_link_type` does NOT (its tokens are already canonical).
    #[test]
    fn case_folding_is_per_table_not_global() {
        let rel = VocabTable::parse(MODEL_RELATIONSHIP_TSV, true).unwrap();
        assert_eq!(rel.get("perturbs"), Some("perturbs"));
        assert_eq!(rel.get("PERTURBS"), Some("perturbs"));
        let clt = VocabTable::parse(CAUSAL_LINK_TYPE_TSV, false).unwrap();
        assert_eq!(clt.get("DIRECT"), Some("Direct"));
        assert_eq!(
            clt.get("direct"),
            None,
            "closed vocabulary does not case-fold"
        );
    }

    /// The `bits2` third column is read from the table, not hardcoded.
    #[test]
    fn the_bits_column_comes_from_the_table() {
        let clt = VocabTable::parse(CAUSAL_LINK_TYPE_TSV, false).unwrap();
        for v in CausalLinkType::ALL {
            let from_table: u8 = clt.extra(v.as_source(), 0).unwrap().parse().unwrap();
            assert_eq!(
                from_table,
                v.to_bits2(),
                "{} bits2 disagrees",
                v.as_source()
            );
        }
    }

    /// F1 — all four topologies round-trip source -> variant -> bits -> variant.
    #[test]
    fn every_topology_round_trips_through_bits() {
        for v in CausalLinkType::ALL {
            assert_eq!(CausalLinkType::from_source(v.as_source()), Some(v));
            assert_eq!(CausalLinkType::from_bits2(v.to_bits2()), Some(v));
        }
        assert_eq!(CausalLinkType::from_bits2(4), None);
    }

    /// F2 — an unrecognized token FAILS CLOSED. It must never become
    /// `Unknown`, because `UNKNOWN` is an asserted source value.
    #[test]
    fn an_unrecognized_token_is_none_never_unknown() {
        for bad in [
            "",
            "  ",
            "direct",
            "Direct",
            "INDIRECT",
            "MAYBE",
            "INDIRECT_KNOWN",
        ] {
            assert_eq!(
                CausalLinkType::from_source(bad),
                None,
                "{bad:?} must not parse"
            );
        }
        // ...and the real token still does, so the test is not vacuous.
        assert_eq!(
            CausalLinkType::from_source("UNKNOWN"),
            Some(CausalLinkType::Unknown)
        );
    }

    /// F3 — the two unresolved states are NOT the same proposition.
    #[test]
    fn mediator_unresolved_and_topology_unresolved_are_disjoint() {
        assert!(CausalLinkType::IndirectUnknownIntermediates.mediator_unresolved());
        assert!(!CausalLinkType::Unknown.mediator_unresolved());
        assert!(CausalLinkType::Unknown.topology_unresolved());
        assert!(!CausalLinkType::IndirectUnknownIntermediates.topology_unresolved());
        // Neither predicate fires on the two resolved states.
        for v in [
            CausalLinkType::Direct,
            CausalLinkType::IndirectKnownIntermediates,
        ] {
            assert!(!v.mediator_unresolved());
            assert!(!v.topology_unresolved());
        }
    }

    /// The scanner reports a drifted token — and stays SILENT on a clean
    /// document. Two-sided: a scanner that reported everything would carry as
    /// much information as one that reported nothing.
    #[test]
    fn the_scanner_reports_drift_and_stays_silent_on_clean_input() {
        let clean: crate::model::Disorder = serde_yaml::from_str(
            "pathophysiology:\n- downstream:\n  - causal_link_type: DIRECT\n    relationship: PERTURBS\n",
        )
        .unwrap();
        assert_eq!(
            unknown_tokens(&clean),
            vec![],
            "clean document must report nothing"
        );

        let drifted: crate::model::Disorder = serde_yaml::from_str(
            "pathophysiology:\n- downstream:\n  - causal_link_type: PROBABLY_DIRECT\n    relationship: READOUT_OF\n",
        )
        .unwrap();
        let got = unknown_tokens(&drifted);
        assert_eq!(
            got.len(),
            2,
            "both drifted values must be reported: {got:?}"
        );
        assert!(got
            .iter()
            .any(|t| t.field == "causal_link_type" && t.value == "PROBABLY_DIRECT"));
        assert!(got
            .iter()
            .any(|t| t.field == "relationship" && t.value == "READOUT_OF"));
    }

    /// A document repeating one bad token many times reports it ONCE.
    #[test]
    fn repeated_drift_is_deduplicated() {
        let mut y = String::from("pathophysiology:\n");
        for _ in 0..50 {
            y.push_str("- downstream:\n  - relationship: READOUT_OF\n");
        }
        let doc: crate::model::Disorder = serde_yaml::from_str(&y).unwrap();
        let got = unknown_tokens(&doc);
        assert_eq!(got.len(), 1, "50 repeats must report once, got {got:?}");
    }
}
