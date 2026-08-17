//! Permissive YAML model for a `monarch-initiative/dismech` disorder file.
//!
//! Deliberately narrow: this first vertical slice bakes DISORDER IDENTITY
//! only (`name`, `description`, `category`, the MONDO xref) -- not the
//! causal-mechanism graph. `monarch-initiative/dismech` (the real upstream;
//! the `AdaWorldAPI/dismech` fork's default branch is STALE and carries no
//! `kb/` corpus at all -- confirmed 2026-08-17, do not clone the fork) ships
//! no separate resolver application, only this LinkML-schema'd YAML corpus.
//! So "100% truthful transcode" here means: read exactly what the YAML
//! declares, against the schema's own field names, inventing nothing.

use serde::Deserialize;

/// A raw disorder document. Untyped at the top level, mirroring the corpus's
/// own shape -- unknown keys tolerated, since this slice only reads a few
/// fields and must not choke on the rest of the schema.
pub type Disorder = std::collections::BTreeMap<String, serde_yaml::Value>;

/// `disease_term: {preferred_term, term: {id, label}}` -- the CURIE-bearing
/// identity wrapper every disorder file carries.
#[derive(Debug, Clone, Deserialize)]
pub struct TermRef {
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

/// Parse one disorder YAML file leniently. A `null`/empty document becomes
/// an empty map, mirroring the corpus's own tolerant load posture.
///
/// # Errors
/// Returns the underlying `serde_yaml` error on genuinely malformed YAML
/// (not on missing/extra fields -- those are handled leniently).
pub fn parse_disorder(bytes: &[u8]) -> Result<DisorderIdentity, serde_yaml::Error> {
    let v: Option<DisorderIdentity> = serde_yaml::from_slice(bytes)?;
    Ok(v.unwrap_or_default())
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
