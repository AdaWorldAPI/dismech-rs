# AGENT_LOG.md — dismech-rs

> **APPEND-ONLY. ONE WRITER.** Only the orchestrating session's main
> thread appends to this file — sub-agents/workers spawned within a
> session do NOT write here directly (concurrent append to one shared
> file is a lost-write race). If sub-agents need to leave a record, they
> write their own tag-file and the orchestrator consolidates into a
> single entry here after the work lands. This mirrors the "ONE WRITER
> PER FILE" rule documented in the sibling `lance-graph` repo's
> `CLAUDE.md` (§ Layer 2 — Session A2A), adopted here for consistency
> even though this repo does not yet run a multi-agent fleet.
>
> Each entry: what was done, what was verified, outcome. Prepend new
> entries at the top.

---

## 2026-08-19 — ownership correction applied to causal-graph-soa-integration-v1 (orchestrator, no agents)

Operator ruling: this repo is the ORACLE, not the integration surface.
Applied main-thread as an append-only correction — banner + retraction
markers on §PROPOSED RESOLUTION + a per-deliverable layer split table on
§DELIVERABLES. No text deleted; the row/lane/mint design is retained
verbatim as the record the bridge layer inherits. No code touched, no
resolver change, zero-dep posture untouched and now load-bearing.

Evidence that forced the predicate half: `ogar-dismech` already mints the
same 19 predicates as `FnIndex 0x90..0xA2` behind a `DisMechVocabulary`
(OGAR read-only sweep, 2026-08-19), so D-DCG-2's parallel `1..19` freeze
would have created a second canonical numbering. PR #7 stays DRAFT.

## 2026-08-19 — causal-graph SoA integration plan authored (1 plan agent, orchestrator-consolidated)

One accumulation-tier plan agent, brief carried no-cargo/no-git/no-board-
writes/one-file-only; wrote `.claude/plans/causal-graph-soa-integration-v1.md`
(488 lines, D-DCG-1..10, O1–O8). This entry is the orchestrator
consolidating per the one-writer rule. Notable authoring-time measurements:
the four-valued `causal_link_type` census (incl. UNKNOWN=361), the upstream
`perturb/graph.py` substring bug (all link types classify DIRECT — its
`elif` is unreachable), the 1,968-vs-1,990/1,996 snapshot drift, and the
single-entry `intermediate_mechanisms` fixture that cannot falsify order
loss. Cross-repo companions landed the same day in lance-graph (PR #969
plan wave); the operator's standing model-allocation rule for this session
applied.

## 2026-08-18 — F4: causal-mechanism graph resolver ported (`graph.rs`)

**Did:** Extended `dismech-bake` with `crates/dismech-bake/src/graph.rs`
(`build_causal_graph`, a direct port of the real upstream
`src/dismech/graph.py::build_causal_graph`, read in full from
`/tmp/dismech-up/src/dismech/graph.py`), `bin/census.rs`
(`dismech_census`, the falsifier binary), and a `NodeItem` typed-model
extension in `model.rs` (+ `de_string_or_list` for the corpus's real
scalar-or-list gotcha). `lib.rs` doc comment corrected to match the
already-resolved 2026-08-18 EPIPHANIES finding.

**Verified:**
- `cargo test -p dismech-bake`: 22/22 passing (13 pre-existing + 9 new,
  all against real-corpus-derived fixtures, none synthetic).
- `cargo clippy -p dismech-bake --all-targets -- -D warnings`: clean.
- `cargo fmt -p dismech-bake`: clean, no diff.
- `dismech_census /tmp/dismech-up` (full real corpus, 1,996 files, 0
  parse errors): **1995 diseases**, **33,458 total edges** — vs.
  `MedCare-rs`'s own 1995/33,328 measurement on the same corpus family:
  exact match on disease count, 0.4% apart on edge count.

**Outcome:** Resolver + typed model layer landed. SoA packing (`pack.rs`)
deliberately NOT touched — flagged as a genuine byte-layout decision in
`TECH_DEBT.md`, not guessed at. Full account:
`.claude/board/EPIPHANIES.md` (2026-08-18, "The causal-mechanism graph
resolver, ported directly against the real Python (F4)").

---

## 2026-08-17 — Repo creation + full bake verification

**What:** Created `dismech-rs` as a new public repo — a pure-Rust
transcode of `monarch-initiative/dismech` disorder identity into the
shared `lance-graph` `NodeRow` byte layout. Built `crates/dismech-bake`
(`lib.rs`, `model.rs`, `pack.rs`, `bake.rs`) and reserved classid
`0x0333` in the shared OGAR `0x03` Ontology domain. Pushed as the first
commit on `main` (`b91d6a1`).

**Verified:**
- Ran the bake against a real `monarch-initiative/dismech` checkout (not
  a synthetic fixture)
- **1,990 / 1,990** disorder YAML files under `kb/disorders/` parsed
- **0 parse errors**
- **98.3%** MONDO-resolution (1,957/1,990 disorders resolve a real
  `MONDO:<num>` xref; 33 fall back to a disjoint ordinal band, flagged
  in-row)
- **13/13** unit tests green
- Confirmed `0x0333` unused anywhere in `AdaWorldAPI/OGAR` at reservation
  time (`grep -rn "0x0333"` across every crate, zero hits)
- Confirmed source correction (see `EPIPHANIES.md` 2026-08-17 entry): the
  `AdaWorldAPI/dismech` fork was stale with no `kb/` corpus; the real
  upstream is `monarch-initiative/dismech`

**Outcome:** First commit landed clean on `main`. `.claude/board/`
scaffold (this file and its siblings) added in a follow-up docs-only PR
— see `PR_ARC_INVENTORY.md` for that entry once merged.
