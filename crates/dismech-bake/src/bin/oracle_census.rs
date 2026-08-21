//! The oracle-population census: how many `INDIRECT_KNOWN_INTERMEDIATES`
//! edges actually NAME a mediator, and how many carry the label alone.
//!
//! **Why this is a Rust binary and not a script.** The first version of this
//! measurement was a throwaway Python line-scanner, and it was WRONG: it
//! reported 2,489 oracle edges where the real figure is 2,512, because a
//! line-oriented sibling-key walk cannot see every YAML shape the corpus
//! uses. The repository already had a structural parser for exactly this
//! corpus -- `model::parse_disorder_raw` + `graph::build_causal_graph`, which
//! carry `intermediate_mechanisms: Vec<String>` on the edge -- so the correct
//! answer was one `cargo run` away the whole time. A measurement that a
//! committed parser can make must not be made by an ad-hoc script: the script
//! is unreviewed, unversioned, and its errors are invisible.
//!
//! Usage: `dismech_oracle_census <path-to-dismech-checkout>`

use std::collections::{BTreeMap, BTreeSet};
use std::path::PathBuf;

use dismech_bake::graph;
use dismech_bake::model;

const KNOWN: &str = "INDIRECT_KNOWN_INTERMEDIATES";
const UNKNOWN_INT: &str = "INDIRECT_UNKNOWN_INTERMEDIATES";

fn main() {
    let mut args = std::env::args().skip(1);
    let Some(repo) = args.next() else {
        eprintln!("usage: dismech_oracle_census <path-to-dismech-checkout>");
        std::process::exit(2);
    };
    let corpus_dir = PathBuf::from(&repo).join("kb").join("disorders");
    if !corpus_dir.is_dir() {
        eprintln!("dismech_oracle_census: {corpus_dir:?} is not a directory");
        std::process::exit(1);
    }

    let mut entries: Vec<_> = std::fs::read_dir(&corpus_dir)
        .unwrap_or_else(|e| panic!("read_dir {corpus_dir:?}: {e}"))
        .filter_map(Result::ok)
        .map(|e| e.path())
        .filter(|p| p.extension().is_some_and(|e| e == "yaml"))
        .collect();
    entries.sort();

    let mut files = 0usize;
    let mut parse_errors = 0usize;
    let mut topology: BTreeMap<String, usize> = BTreeMap::new();
    let mut known_with = 0usize;
    let mut known_without = 0usize;
    let mut contradictory = 0usize;
    let mut oracle_diseases: BTreeSet<String> = BTreeSet::new();
    let mut mediator_strings = 0usize;
    let mut distinct: BTreeSet<String> = BTreeSet::new();

    for path in &entries {
        files += 1;
        let Ok(bytes) = std::fs::read(path) else {
            parse_errors += 1;
            continue;
        };
        let Ok(doc) = model::parse_disorder_raw(&bytes) else {
            parse_errors += 1;
            continue;
        };
        let g = graph::build_causal_graph(&doc);
        let stem = path
            .file_stem()
            .map(|s| s.to_string_lossy().into_owned())
            .unwrap_or_default();
        for e in &g.edges {
            let Some(t) = e.causal_link_type.as_deref() else {
                continue;
            };
            *topology.entry(t.to_string()).or_default() += 1;
            let named = !e.intermediate_mechanisms.is_empty();
            match (t, named) {
                (KNOWN, true) => {
                    known_with += 1;
                    oracle_diseases.insert(stem.clone());
                    for m in &e.intermediate_mechanisms {
                        mediator_strings += 1;
                        distinct.insert(m.trim().to_string());
                    }
                }
                (KNOWN, false) => known_without += 1,
                (UNKNOWN_INT, true) => contradictory += 1,
                _ => {}
            }
        }
    }

    let known_total = known_with + known_without;
    println!("files {files}  parse_errors {parse_errors}");
    println!("\n-- causal_link_type census --");
    let mut total = 0usize;
    for (k, v) in &topology {
        println!("  {k:<32} {v}");
        total += v;
    }
    println!("  {:<32} {total}", "TOTAL");

    println!("\n-- the oracle population --");
    println!("  label-KNOWN edges                {known_total}");
    println!(
        "    with >=1 named mediator        {known_with}  ({:.1}%)",
        pct(known_with, known_total)
    );
    println!(
        "    label only, NO mediator        {known_without}  ({:.1}%)",
        pct(known_without, known_total)
    );
    println!(
        "  distinct diseases contributing    {}",
        oracle_diseases.len()
    );
    println!("  mediator strings                  {mediator_strings}");
    println!("  distinct mediator strings         {}", distinct.len());
    println!("\n-- the source contradiction --");
    println!("  UNKNOWN_INTERMEDIATES that DO name mediators  {contradictory}");
    println!("  (these are neither oracle nor restraint control)");
}

fn pct(n: usize, d: usize) -> f64 {
    if d == 0 {
        0.0
    } else {
        n as f64 * 100.0 / d as f64
    }
}
