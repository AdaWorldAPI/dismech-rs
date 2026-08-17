//! Walk the real `kb/disorders/*.yaml` corpus and bake identity rows.

use std::fs;
use std::path::Path;

use crate::model::{self, DisorderIdentity};
use crate::pack::{self, Row512, DISMECH_CONCEPT};

/// One baked row plus the label text that rides the out-of-line lane
/// (mirrors the golden bake's `obo_full_labels.tsv` split: the row carries
/// lengths and flags, the text lives beside it, never packed in-row).
pub struct BakedDisorder {
    pub row: Row512,
    pub name: String,
    pub description: String,
    pub mondo_curie: Option<String>,
}

/// Honest counters — every one of these is a fact about the real corpus,
/// never a guess. `unresolved_mondo` is not a failure: some corpus records
/// (comorbidities, groupings, hypotheses) legitimately carry no single
/// MONDO xref, and this bake only tiles `kb/disorders/`.
#[derive(Debug, Default, Clone, Copy, PartialEq, Eq)]
pub struct BakeStats {
    pub files_seen: usize,
    pub parse_errors: usize,
    pub mondo_mirrored: usize,
    pub fallback_addressed: usize,
}

/// Bake every `kb/disorders/*.yaml` file under `corpus_dir` into a
/// `(rows, stats)` pair. `app_prefix` is the low-u16 half of canon-high
/// classid composition (`(DISMECH_CONCEPT << 16) | app_prefix`) — the
/// caller's own render-app identity, never invented here.
///
/// **Addressing rule:** when a file's `disease_term.term.id` parses as
/// `MONDO:<digits>`, this row's identity is that SAME numeric (mirrored
/// addressing — see `pack.rs`'s module doc). When it does not (absent,
/// malformed, or a non-MONDO prefix), the row is addressed by a **fallback
/// ordinal starting at `0x0080_0000`** — a fixed, disjoint band chosen so a
/// fallback identity can never collide with a real MONDO numeric (MONDO's
/// own ids are all `< 0x0080_0000`; measured max magnitude in
/// `docs/ONTOLOGY_BAKE_STATE.md`-adjacent work is under 800,000, and
/// `0x0080_0000` = 8,388,608). The distinction survives in-row (`pack.rs`'s
/// `mondo_mirrored` byte) so a reader never mistakes a fallback address for
/// a real cross-domain join key.
///
/// # Errors
/// Returns an I/O error only for a directory-level failure (missing
/// `corpus_dir`, permission denied) — a single malformed file is counted
/// in `BakeStats::parse_errors` and skipped, never aborts the whole bake.
pub fn bake_disorders(
    corpus_dir: &Path,
    app_prefix: u16,
) -> std::io::Result<(Vec<BakedDisorder>, BakeStats)> {
    const FALLBACK_BASE: u32 = 0x0080_0000;
    let classid = (u32::from(DISMECH_CONCEPT) << 16) | u32::from(app_prefix);
    let mut rows = Vec::new();
    let mut stats = BakeStats::default();
    let mut fallback_next = FALLBACK_BASE;

    let mut entries: Vec<_> = fs::read_dir(corpus_dir)?
        .filter_map(Result::ok)
        .map(|e| e.path())
        .filter(|p| p.extension().is_some_and(|e| e == "yaml"))
        .collect();
    entries.sort(); // deterministic bake order, run to run

    for path in entries {
        stats.files_seen += 1;
        let bytes = match fs::read(&path) {
            Ok(b) => b,
            Err(_) => {
                stats.parse_errors += 1;
                continue;
            }
        };
        let d: DisorderIdentity = match model::parse_disorder(&bytes) {
            Ok(d) => d,
            Err(_) => {
                stats.parse_errors += 1;
                continue;
            }
        };

        let mondo_num = d
            .disease_term
            .as_ref()
            .and_then(model::TermRef::curie)
            .and_then(parse_mondo_numeric);

        let (identity, mirrored) = match mondo_num {
            Some(n) => {
                stats.mondo_mirrored += 1;
                (n, true)
            }
            None => {
                let n = fallback_next;
                fallback_next += 1;
                stats.fallback_addressed += 1;
                (n, false)
            }
        };

        let row = pack::pack_row(classid, identity, &d, mirrored);
        let mondo_curie = d
            .disease_term
            .as_ref()
            .and_then(model::TermRef::curie)
            .map(str::to_string);
        rows.push(BakedDisorder {
            row,
            name: d.name.clone().unwrap_or_default(),
            description: d.description.clone().unwrap_or_default(),
            mondo_curie,
        });
    }

    Ok((rows, stats))
}

/// Parse `MONDO:0018923` -> `18923`. `None` for any other prefix or a
/// malformed numeric tail — never a guess, per this bake's own honesty
/// discipline (identical posture to `medcare-dismech`'s identity
/// resolution earlier in this workspace's history).
fn parse_mondo_numeric(curie: &str) -> Option<u32> {
    curie.strip_prefix("MONDO:")?.parse().ok()
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::io::Write;

    fn write_fixture(dir: &Path, name: &str, contents: &str) {
        let mut f = fs::File::create(dir.join(name)).unwrap();
        f.write_all(contents.as_bytes()).unwrap();
    }

    /// End-to-end over a small real-shaped fixture set: one file with a
    /// resolvable MONDO xref, one without, one malformed. All three
    /// outcomes must be counted, and the two valid rows must carry
    /// DISTINCT identities from DISJOINT address bands.
    #[test]
    fn bakes_a_mixed_fixture_set_honestly() {
        let dir = std::env::temp_dir().join(format!("dismech_bake_test_{}", std::process::id()));
        fs::create_dir_all(&dir).unwrap();

        write_fixture(
            &dir,
            "a.yaml",
            "name: A\ndisease_term:\n  term:\n    id: MONDO:0018923\n",
        );
        write_fixture(&dir, "b.yaml", "name: B\ncategory: Genetic\n");
        write_fixture(&dir, "c.yaml", "name: [broken\n");

        let (rows, stats) = bake_disorders(&dir, 0x1000).unwrap();
        fs::remove_dir_all(&dir).unwrap();

        assert_eq!(stats.files_seen, 3);
        assert_eq!(
            stats.parse_errors, 1,
            "the broken file must be counted, not silently dropped"
        );
        assert_eq!(stats.mondo_mirrored, 1);
        assert_eq!(stats.fallback_addressed, 1);
        assert_eq!(rows.len(), 2, "two valid files -> two rows");

        let (_, id_a) = pack::unpack_key(&rows[0].row.0);
        let (_, id_b) = pack::unpack_key(&rows[1].row.0);
        assert_eq!(id_a, 18_923, "file a mirrors its real MONDO numeric");
        assert!(
            id_b >= 0x0080_0000,
            "file b falls in the disjoint fallback band"
        );
        assert_ne!(id_a, id_b);
    }

    /// Anti-vacuity: a directory of ONLY malformed files must still report
    /// every file as a parse error, not silently produce zero rows and
    /// zero errors (which would look identical to "no work to do").
    #[test]
    fn an_all_broken_directory_reports_every_failure() {
        let dir =
            std::env::temp_dir().join(format!("dismech_bake_test_broken_{}", std::process::id()));
        fs::create_dir_all(&dir).unwrap();
        write_fixture(&dir, "x.yaml", "name: [unterminated\n");
        write_fixture(&dir, "y.yaml", "name: [also broken\n");

        let (rows, stats) = bake_disorders(&dir, 0).unwrap();
        fs::remove_dir_all(&dir).unwrap();

        assert!(rows.is_empty());
        assert_eq!(stats.parse_errors, 2);
        assert_eq!(stats.files_seen, 2);
    }

    /// Determinism: the same corpus baked twice must produce identical
    /// fallback identities, run to run — a bake that isn't reproducible
    /// can't be re-verified against a pinned artifact.
    #[test]
    fn fallback_addressing_is_deterministic_across_runs() {
        let dir = std::env::temp_dir().join(format!(
            "dismech_bake_test_determinism_{}",
            std::process::id()
        ));
        fs::create_dir_all(&dir).unwrap();
        write_fixture(&dir, "z1.yaml", "name: Z1\n");
        write_fixture(&dir, "z2.yaml", "name: Z2\n");

        let (rows1, _) = bake_disorders(&dir, 0).unwrap();
        let (rows2, _) = bake_disorders(&dir, 0).unwrap();
        fs::remove_dir_all(&dir).unwrap();

        let ids1: Vec<u32> = rows1.iter().map(|r| pack::unpack_key(&r.row.0).1).collect();
        let ids2: Vec<u32> = rows2.iter().map(|r| pack::unpack_key(&r.row.0).1).collect();
        assert_eq!(ids1, ids2);
    }

    #[test]
    fn mondo_numeric_parsing_refuses_non_mondo_and_malformed() {
        assert_eq!(parse_mondo_numeric("MONDO:0018923"), Some(18_923));
        assert_eq!(parse_mondo_numeric("HP:0000001"), None);
        assert_eq!(parse_mondo_numeric("MONDO:not_a_number"), None);
        assert_eq!(parse_mondo_numeric("MONDO:"), None);
    }
}
