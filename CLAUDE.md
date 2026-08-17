# CLAUDE.md — dismech-rs

## What this is

A **public, agnostic** transcode of `monarch-initiative/dismech` (disease
mechanisms — disorder identity, phenotype, treatment, biochemical, and
genetic knowledge base) into a Rust-native SoA bake, sunk into the same
`lance-graph` NodeRow substrate the OGAR OBO-core bake (`ogar-obo`) already
uses. The goal, verbatim from the operator: *"agnostic public surface for
medical as an open medical patterns proxy for the private MedCare-rs."*

`dismech-rs` never carries patient data. It is the public medical-knowledge
layer; `AdaWorldAPI/MedCare-rs` (private) is the consumer that layers real
patient context on top — the same public/private split this workspace
already uses for `ogar-obo`/`ogar-fma` (public reference) vs `MedCare-rs`
(private patient application).

## Source — read this before re-cloning anything

**Source is `https://github.com/monarch-initiative/dismech`** — the real
upstream, Apache-2.0/CC-BY reference content, no PHI.

**`AdaWorldAPI/dismech` (the fork) is STALE on its default branch and
carries NO `kb/` corpus directory at all** — confirmed 2026-08-17 via a
fresh shallow clone (`app/`, `cache/`, `data/`, `docs/`, `attic/`,
`dashboard/` only, no disorder YAML anywhere). Do not clone the fork
expecting the corpus. If the fork is ever needed again (e.g. to sync a
divergent branch), check its branch list first — the default branch alone
is not the corpus.

The real corpus: `kb/disorders/*.yaml` (1,990 files as of 2026-08-17),
plus sibling `kb/{comorbidities,groupings,hypotheses,modules,
surrogate_endpoints}/` this repo does not yet consume.

**There is no separate Python resolver application anywhere in the
upstream repo** (no `graph.py`, no `build_causal_graph`, no
`pathograph_export.py`) — only 17 Python files, all tooling (GH Action
helpers, LinkML QC, a disease-trajectory extraction skill). So "100%
truthful transcode" here means reading exactly what the YAML declares
against its own field names — there is no behavioral parity oracle to
build toward (no Python app to diff against), only the corpus's own
declared shape.

## Classid — `0x0333`

Reserved concept slot in the shared `0x03` OGAR "Ontology" domain, one
slot family past RO's relation-body concept (`0x0306`). Measured unused
anywhere in `AdaWorldAPI/OGAR` on 2026-08-17 (`grep -rn "0x0333"` across
every crate, zero hits before this reservation). `classid = (0x0333 << 16)
| app_prefix`, the same canon-high idiom every OGAR-addressed domain uses.

**Mirrored addressing is the whole point.** Where a disorder resolves to a
`MONDO:<num>` xref (measured: 1,957/1,990 = 98.3% of the real corpus),
its DisMech-domain `identity` is set to that SAME numeric — not a fresh
ordinal. A DisMech row and its MONDO row then differ ONLY in `classid`;
`unpack_key` on either yields the identical `identity`. This is the
pre-bake join the earlier `MedCare-rs` SPOG-unification discussion
(`docs/DISMECH_BAKE_PLAN.md` §13-15) was working toward by hand — here
it's the address scheme itself, not a lookup table. The remaining 1.7%
(33 disorders with no resolvable MONDO xref) get an honest fallback
ordinal from a disjoint band (`0x0080_0000+`), flagged in-row
(`pack.rs`'s `mondo_mirrored` byte) so a reader never mistakes a fallback
address for a real cross-domain join key.

## Byte layout — a contract, not a code dependency

`crates/dismech-bake` depends on nothing from `lance-graph-contract` or
`ogar-obo` — it mirrors `ogar-obo`'s own posture exactly (see that
crate's own doc comment: "The loader connection is a BYTE-LAYOUT
contract, not a code dep"). The 512-byte `NodeRow` shape (`key(16) |
edges(16) | value(480)`) is replicated locally in `pack.rs`, byte-for-byte
compatible with what `lance-graph`'s `node_rows_from_le_bytes` reads back
as `&[NodeRow]` zero-copy.

## Current scope — disorder IDENTITY only

`crates/dismech-bake` bakes `name` / `description` / `category` / the
MONDO xref. It does NOT yet bake the causal-mechanism graph (treatments,
phenotype edges, gene associations) — that is real, substantial follow-on
work, deliberately not attempted in this first pass. Verified against the
full real corpus: 1,990/1,990 files parsed, **zero parse errors**,
98.3% MONDO-resolution.

## Open questions — not yet decided

- **Should `MedCare-rs`'s `crates/medcare-dismech` move here?**
  **Re-corrected 2026-08-17 (second correction — the first correction
  was itself wrong):** an earlier pass in this repo claimed the crate
  "does not exist", based on a `find` run against a local `MedCare-rs`
  checkout that was 193 commits behind `origin/main` on a stale branch
  from earlier in the same session — the check never touched the real
  state. Re-verified against a fresh `MedCare-rs` clone synced to
  `origin/main`: `crates/medcare-dismech` is real and substantial — 5
  modules (`lib.rs`, `model.rs`, `identity.rs`, `graph.rs`, `parity.rs`)
  plus 4 binaries (`bake_dismech`, `dismech_census`, `dismech_identity`,
  `dismech_parity`), ~2,000 lines, documented in
  `MedCare-rs/docs/CAUSALITY_V3_DISMECH_CONTRACT.md` and
  `docs/DISMECH_BAKE_PLAN.md`. Per its own module doc: it is a
  from-scratch Rust transcode of the Python DisMech resolver's
  `graph.py::build_causal_graph`, validated by `parity.rs`/
  `dismech_parity` diffing the Rust output against 1,870 committed
  `pathographs/MONDO_*.json` files (the Python resolver's own output) —
  never invoking Python at build or test time. Whether that parity gate
  is CURRENTLY green, and whether the Python source + pathographs are
  still present in a checkout, was **not** re-verified in this pass
  (this repo has no access to `MedCare-rs`'s private checkout state
  beyond a point-in-time read) — that is `MedCare-rs`'s own thing to
  confirm, not this repo's. Recommendation on the move-or-not question
  itself is unchanged from before either correction: not yet decided,
  and not this repo's call to make unilaterally — it is a
  `MedCare-rs`-side architectural decision. (See `.claude/board/
  TECH_DEBT.md` for the full correction history — two entries now,
  the first correction and this one, both struck rather than deleted.)
- **`ogar-dismech`** — exists now. Built and merged 2026-08-17:
  https://github.com/AdaWorldAPI/OGAR/pull/274. It is the OGAR-side
  compile-time collision guard for `0x0333` (mirrors `ogar-ro`'s
  `RELATION_BODY_CONCEPT_ID` pattern) — nothing from this repo moved
  into it; `dismech-bake` stays zero-dependency.
- **S3 sink-in to the shared `lance-graph` SoA table, "volume01" hot
  reload, q2-repository-pattern re-embedding, paging/SPOG routing** —
  named goals, not yet built. This repo currently produces a flat
  `.soa` blob (`dismech_bake --out rows.soa`) and a round-trip-verified
  S3 upload script (`scripts/upload-bake.sh`, byte-identical method to
  `MedCare-rs`'s own). The Lance-table sink-in and hot-reload path are
  the next phase.

## Commands

```bash
# Bake against a real monarch-initiative/dismech checkout:
cargo run --release --bin dismech_bake -- /path/to/dismech --out rows.soa

# Upload a bake (needs AWS_ENDPOINT_URL / AWS_S3_BUCKET_NAME /
# AWS_ACCESS_KEY_ID / AWS_SECRET_ACCESS_KEY):
scripts/upload-bake.sh <tag> rows.soa
```

## Reference documents

Board hygiene for this repo lives under `.claude/board/` and
`.claude/plans/`, mirroring the pattern the sibling `lance-graph` and
`MedCare-rs` repos use. Read these before doing substantial work here:

- `.claude/board/LATEST_STATE.md` — mutable "what exists now": crate
  inventory, the classid reservation and its collision-avoidance
  history, current bake stats, active/queued work.
- `.claude/board/PR_ARC_INVENTORY.md` — append-only, one row per merged
  PR (Added/Locked/Deferred/Docs/Confidence).
- `.claude/board/INTEGRATION_PLANS.md` — append-only index of versioned
  plans under `.claude/plans/<name>-v<N>.md`.
- `.claude/board/TECH_DEBT.md` — known gaps, not yet fixed (starting
  with the still-open `ogar-dismech` classid collision guard).
- `.claude/board/EPIPHANIES.md` — append-only, dated findings and
  corrections (starting with the source-correction finding: the
  `AdaWorldAPI/dismech` fork was stale, the real upstream is
  `monarch-initiative/dismech`).
- `.claude/board/AGENT_LOG.md` — append-only, one-writer session record.
- `.claude/plans/ogar-classid-registration-v1.md` — the still-open plan
  to register `0x0333` as a compile-time collision guard in OGAR.
