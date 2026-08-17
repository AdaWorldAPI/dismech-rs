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
