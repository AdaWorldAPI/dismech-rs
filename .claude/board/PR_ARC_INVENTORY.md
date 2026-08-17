# PR_ARC_INVENTORY.md — dismech-rs

> **APPEND-ONLY.** One row per merged PR, Added / Locked / Deferred / Docs
> / Confidence, newest-first (PREPEND new entries at the top). Only the
> Confidence line of an existing entry may be updated after the fact;
> corrections append as new dated lines, reversals get their own entry.
> Mirrors the convention in `lance-graph/.claude/board/PR_ARC_INVENTORY.md`
> and `MedCare-rs`'s board.

---

## Initial commit `b91d6a1` — 2026-08-17 (not a PR; repo genesis on `main`)

**Title:** dismech-bake: real transcode of monarch-initiative/dismech
disorder identity

**Added:**
- `crates/dismech-bake` — new crate, zero external byte-layout deps
  (`lance-graph-contract`/`ogar-obo` are NOT dependencies; the 512-byte
  `NodeRow` shape is replicated locally as a byte-layout contract)
- `model.rs` — disorder YAML-facing struct (name/description/category/
  MONDO xref)
- `pack.rs` — `NodeRow` byte packer, `DISMECH_CONCEPT: u16 = 0x0333`
  classid reservation, mirrored-addressing logic, `mondo_mirrored` flag
  byte for the 1.7% fallback-ordinal band
- `bake.rs` — directory-walk + parse + pack driver, `dismech_bake` binary
- `scripts/upload-bake.sh` — S3 upload, byte-identical method to
  `MedCare-rs`'s own upload script
- `CLAUDE.md` — full architecture writeup (source-correction history,
  classid reservation, mirrored addressing, bake stats, open questions)

**Locked:**
- Classid `0x0333` chosen and measured clear of every OGAR `0x03XX`
  registration known at the time (see `LATEST_STATE.md` "Classid
  reservation" for the full collision-avoidance history)
- Byte-layout-contract-not-code-dependency posture (mirrors `ogar-obo`)
- Disorder identity fields baked: name / description / category / MONDO
  xref

**Deferred:**
- Causal-mechanism graph (treatments, phenotype edges, gene associations)
- Sibling `kb/{comorbidities,groupings,hypotheses,modules,
  surrogate_endpoints}/` directories
- `ogar-dismech` compile-time collision guard on the OGAR side (not yet
  created anywhere in OGAR as of 2026-08-17)
- S3 sink-in to the shared `lance-graph` SoA table, hot reload,
  q2-repository-pattern re-embedding, paging/SPOG routing
- Decision on moving `MedCare-rs`'s `crates/medcare-dismech` here

**Docs:** `CLAUDE.md` (repo root) — the canonical architecture doc for
this repo. `.claude/board/` scaffold (this ledger set) added in a
follow-up docs-only PR — see the next entry once merged.

**Confidence:** HIGH on bake correctness (1,990/1,990 files parsed, 0
parse errors, 98.3% MONDO-resolution, 13/13 unit tests green — all
measured against a real `monarch-initiative/dismech` checkout, not
synthetic fixtures). LOW-to-none on the downstream sink-in path (S3
Lance-table hot-reload, `ogar-dismech` guard) — those are named goals,
not built.
