//! Run the real bake against a checked-out `monarch-initiative/dismech`
//! corpus and report honest stats. Writes nothing by default; pass
//! `--out <path>` to write the packed rows as a flat little-endian blob
//! (512 bytes/row, ready for S3 upload via `scripts/upload-bake.sh`).
//!
//! Usage: `dismech_bake <path-to-dismech-checkout> [--out rows.soa]`

use std::path::PathBuf;

fn main() {
    let mut args = std::env::args().skip(1);
    let Some(repo) = args.next() else {
        eprintln!("usage: dismech_bake <path-to-dismech-checkout> [--out rows.soa]");
        std::process::exit(2);
    };
    let mut out: Option<PathBuf> = None;
    while let Some(a) = args.next() {
        if a == "--out" {
            out = args.next().map(PathBuf::from);
        }
    }

    let corpus_dir = PathBuf::from(&repo).join("kb").join("disorders");
    if !corpus_dir.is_dir() {
        eprintln!(
            "dismech_bake: {corpus_dir:?} is not a directory -- did you point this at a \
             monarch-initiative/dismech checkout? (NOT the AdaWorldAPI/dismech fork's \
             default branch, which carries no kb/ at all)"
        );
        std::process::exit(1);
    }

    let (rows, stats) = dismech_bake::bake::bake_disorders(&corpus_dir, 0x0000)
        .unwrap_or_else(|e| panic!("bake failed: {e}"));

    println!("dismech_bake: {corpus_dir:?}");
    println!("  files seen          : {}", stats.files_seen);
    println!("  parse errors         : {}", stats.parse_errors);
    println!("  mondo-mirrored rows  : {}", stats.mondo_mirrored);
    println!("  fallback-addressed   : {}", stats.fallback_addressed);
    println!("  total rows baked     : {}", rows.len());
    if stats.files_seen > 0 {
        let pct = 100.0 * stats.mondo_mirrored as f64 / stats.files_seen as f64;
        println!("  mondo-resolution rate: {pct:.1}%");
    }

    if let Some(path) = out {
        let mut buf = Vec::with_capacity(rows.len() * dismech_bake::pack::NODE_ROW_STRIDE);
        for r in &rows {
            buf.extend_from_slice(&r.row.0);
        }
        std::fs::write(&path, &buf).unwrap_or_else(|e| panic!("write {path:?}: {e}"));
        println!("  wrote {} bytes to {path:?}", buf.len());
    }
}
