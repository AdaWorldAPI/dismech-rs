# PR_ARC_INVENTORY.md — dismech-rs

> **APPEND-ONLY.** One row per merged PR, Added / Locked / Deferred / Docs
> / Confidence, newest-first (PREPEND new entries at the top). Only the
> Confidence line of an existing entry may be updated after the fact;
> corrections append as new dated lines, reversals get their own entry.
> Mirrors the convention in `lance-graph/.claude/board/PR_ARC_INVENTORY.md`
> and `MedCare-rs`'s board.

---

## 2026-08-21 — dismech-rs #8 (MERGED, merge commit `fd64453`, head `e44ab48`) — vocab-as-config, the oracle census, and the grounding dumps

**Title:** vocab-as-config + `dismech_oracle_census` (+ mediator / node-name
dumps)

**Added:**
- `crates/dismech-bake/src/vocab.rs` — the data-as-config module.
  `VocabTable::parse` over three committed TSVs
  (`causal_link_type` / `environmental_effect` / `model_relationship`);
  `CausalLinkType` with a **fail-closed** `from_source`, `to_bits2` /
  `from_bits2`, `mediator_unresolved()` (true ONLY for
  `IndirectUnknownIntermediates`), `topology_unresolved()` (only `Unknown`);
  `UnknownToken` + `unknown_tokens(&Disorder)` surfacing LLM-noise
  fallbacks as integrity issues rather than silent defaults.
- `data/*.tsv` — each header carries its own measured provenance, including
  the note that `model_relationship`'s 12-value count POOLS two sites so no
  misclassification rate is claimed.
- `src/bin/oracle_census.rs` (`dismech_oracle_census`) — the census that
  measures the oracle population, plus `--dump-mediators` (3,465 rows,
  `disease/source/target/mediator`) and `--dump-nodes` (31,436 distinct
  causal-graph node names). Both emitted by the SAME parser that produces
  the census, so a grounding run and its census cannot disagree about which
  edges exist.
- `examples/vocab_drift.rs` — real-corpus drift probe.
- `graph.rs` — `unknown_tokens` folded into `integrity_issues` (extend, not
  assign, so `check_referential_integrity` no longer clobbers it).

**Measured (release, 2,100 files, 0 parse errors, sub-second):**
`DIRECT 9,073 / INDIRECT_KNOWN 3,978 / INDIRECT_UNKNOWN 4,539 / UNKNOWN 408`
= **17,998**; label-KNOWN with ≥1 named mediator **2,512 (63.1%)** over
**549** diseases; label-only **1,466 (36.9%)**; mediator strings 3,465,
distinct 3,095; **92** `INDIRECT_UNKNOWN_INTERMEDIATES` edges DO name
mediators.

**Locked:**
- **Transcode parity is untouched.** `vocab.rs` changes no predicate and
  drops no edge; upstream `graph.py`'s fail-OPEN behaviour is preserved and
  the divergence is surfaced as an integrity issue instead.
- **A measurement a committed parser can make must not be made by an ad-hoc
  script.** The first version of the oracle count came from a Python
  line-scanner and was wrong by 23 edges (2,489 vs 2,512) — caught by a
  structural contradiction, not by a test. Cross-checked against an
  independent pyyaml structural parse: identical on every figure. The
  census line also reproduces the lance-graph board's own
  `E-DISMECH-CORPUS-CENSUS-1` exactly, which is the anti-vacuity check that
  the binary reads the corpus it claims to.
- **`--dump-nodes` is bounded two-sided:** 31,436 sits between the census
  node total (56,447, with duplicates across diseases) and the oracle edge
  endpoints (2,636). A value at either bound would mean the dump is reading
  the wrong collection.

**Deferred:**
- The third bucket for the **1,466** label-only edges and the **92**
  contradictory ones — an operator decision that gates any gold set. Scored
  as positives they are unrecoverable; as negatives they punish a correct
  answer.
- Packing the causal graph into the 512-byte `NodeRow` (the resolver runs;
  `pack.rs` still carries disorder identity only).

**Docs:** `.claude/plans/causality-v3-rebase-report-v1.md` (the §26 rebase
report, including its own retracted §7 — kept in place, not deleted).

**NOT recorded here, deliberately:** the mediator-grounding results. They
depend on reference artifacts owned by a private consumer and this
repository is public — mechanism and magnitudes only.

**Confidence:** High for every count (two independent structural parsers
agree). Medium for the grounding rungs, which are heuristic label matching
with the ungrounded residue reported rather than dropped.

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
