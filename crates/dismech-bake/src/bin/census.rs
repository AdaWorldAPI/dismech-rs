//! Run `graph::build_causal_graph` over every real `kb/disorders/*.yaml`
//! file and report honest node/edge totals — the falsifier for the
//! causal-mechanism-graph port. The private sibling `MedCare-rs`'s own
//! measurement on the same upstream corpus: diseases 1995, total edges
//! 33,328 (see the F4 brief this binary was built from).
//!
//! Usage: `dismech_census <path-to-dismech-checkout>`

use std::path::PathBuf;

use dismech_bake::graph::{self, NodeType};
use dismech_bake::model;

fn main() {
    let mut args = std::env::args().skip(1);
    let Some(repo) = args.next() else {
        eprintln!("usage: dismech_census <path-to-dismech-checkout>");
        std::process::exit(2);
    };

    let corpus_dir = PathBuf::from(&repo).join("kb").join("disorders");
    if !corpus_dir.is_dir() {
        eprintln!(
            "dismech_census: {corpus_dir:?} is not a directory -- did you point this at a \
             monarch-initiative/dismech checkout?"
        );
        std::process::exit(1);
    }

    let mut entries: Vec<_> = std::fs::read_dir(&corpus_dir)
        .unwrap_or_else(|e| panic!("read_dir {corpus_dir:?}: {e}"))
        .filter_map(Result::ok)
        .map(|e| e.path())
        .filter(|p| p.extension().is_some_and(|e| e == "yaml"))
        .collect();
    entries.sort();

    let mut files_seen = 0usize;
    let mut parse_errors = 0usize;
    let mut diseases_with_nodes = 0usize;
    let mut total_nodes = 0usize;
    let mut total_edges = 0usize;
    let mut total_orphan_targets = 0usize;
    let mut node_type_counts: std::collections::BTreeMap<&'static str, usize> =
        std::collections::BTreeMap::new();
    let mut predicate_counts: std::collections::BTreeMap<String, usize> =
        std::collections::BTreeMap::new();

    for path in &entries {
        files_seen += 1;
        let bytes = match std::fs::read(path) {
            Ok(b) => b,
            Err(_) => {
                parse_errors += 1;
                continue;
            }
        };
        let doc = match model::parse_disorder_raw(&bytes) {
            Ok(d) => d,
            Err(_) => {
                parse_errors += 1;
                continue;
            }
        };

        let g = graph::build_causal_graph(&doc);
        if !g.nodes.is_empty() {
            diseases_with_nodes += 1;
        }
        total_nodes += g.nodes.len();
        total_edges += g.edges.len();
        total_orphan_targets += g.orphan_targets.len();

        for node in g.nodes.values() {
            *node_type_counts.entry(node.node_type.as_str()).or_insert(0) += 1;
        }
        for edge in &g.edges {
            *predicate_counts.entry(edge.predicate.clone()).or_insert(0) += 1;
        }
    }

    println!("dismech_census: {corpus_dir:?}");
    println!("  files seen              : {files_seen}");
    println!("  parse errors             : {parse_errors}");
    println!("  diseases (>=1 node)      : {diseases_with_nodes}");
    println!("  total nodes              : {total_nodes}");
    println!("  total edges              : {total_edges}");
    println!("  total orphan targets     : {total_orphan_targets}");
    println!();
    println!("  nodes by type:");
    for (t, n) in &node_type_counts {
        println!("    {t:<20}: {n}");
    }
    println!();
    println!("  edges by predicate:");
    for (p, n) in &predicate_counts {
        println!("    {p:<20}: {n}");
    }

    // Reference from the private MedCare-rs's own measurement on the same
    // upstream corpus snapshot family (see the F4 brief): diseases 1995,
    // total edges 33,328. Printed for a human diff, not asserted here —
    // corpus snapshots drift over time and this binary must still run
    // cleanly against whatever snapshot it's pointed at.
    println!();
    println!("  reference (MedCare-rs medcare-dismech, same corpus family): diseases 1995, total edges 33328");
    if !node_type_counts.contains_key(NodeType::Pathophysiology.as_str()) {
        eprintln!("dismech_census: warning: zero pathophysiology nodes across the whole corpus -- suspicious");
    }
}
