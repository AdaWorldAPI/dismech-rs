//! `vocab_drift` — run the committed vocabulary tables against a real corpus
//! and report every value that falls through.
//!
//! This is the probe behind `dismech_bake::vocab`'s measured numbers. A drift
//! detector that never fires on real data is decoration, so this exists to be
//! run against an actual `monarch-initiative/dismech` checkout:
//!
//! ```text
//! cargo run --release --example vocab_drift -- /path/to/dismech
//! ```
use dismech_bake::{model::parse_disorder_raw, vocab};
use std::collections::BTreeMap;

fn main() -> Result<(), Box<dyn std::error::Error>> {
    let root = std::env::args()
        .nth(1)
        .ok_or("usage: vocab_drift <dismech-checkout>")?;
    let dir = std::path::Path::new(&root).join("kb/disorders");
    let mut files = 0usize;
    let mut parse_errors = 0usize;
    let mut drift: BTreeMap<(&'static str, String), usize> = BTreeMap::new();
    let mut docs_with_drift = 0usize;

    let mut entries: Vec<_> = std::fs::read_dir(&dir)?.filter_map(Result::ok).collect();
    entries.sort_by_key(std::fs::DirEntry::path);
    for e in entries {
        let p = e.path();
        if p.extension().and_then(|s| s.to_str()) != Some("yaml") {
            continue;
        }
        files += 1;
        let text = std::fs::read(&p)?;
        let Ok(doc) = parse_disorder_raw(&text) else {
            parse_errors += 1;
            continue;
        };
        let unknown = vocab::unknown_tokens(&doc);
        if !unknown.is_empty() {
            docs_with_drift += 1;
        }
        for t in unknown {
            *drift.entry((t.field, t.value)).or_default() += 1;
        }
    }

    println!("files            {files}");
    println!("parse errors     {parse_errors}");
    println!("docs with drift  {docs_with_drift}");
    println!("distinct drifted values {}", drift.len());
    println!("\nfield                 documents  value");
    for ((field, value), docs) in &drift {
        println!("  {field:<20} {docs:>6}  {value}");
    }
    Ok(())
}
