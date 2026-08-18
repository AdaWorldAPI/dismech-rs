# LATEST_STATE.md — dismech-rs

> Mutable "what exists now." Updated in place, not append-only (contrast
> `PR_ARC_INVENTORY.md` and `EPIPHANIES.md`, which are append-only ledgers).
> Last updated: 2026-08-17.

## Repository

`AdaWorldAPI/dismech-rs` — public, agnostic pure-Rust transcode of
`monarch-initiative/dismech` (disease-mechanism knowledge base: disorder
identity, phenotype, treatment, biochemical, and genetic content). Carries
**no patient data** — it is the public medical-knowledge layer that
`MedCare-rs` (private) is meant to eventually consume alongside `ogar-obo`.

Created 2026-08-17. First commit `b91d6a1` on `main`: "dismech-bake: real
transcode of monarch-initiative/dismech disorder identity."

## Crate inventory

| Crate | Purpose | Deps |
|---|---|---|
| `crates/dismech-bake` | Parses `kb/disorders/*.yaml`, packs disorder identity rows into the shared 512-byte `NodeRow` byte layout, writes a flat `.soa` blob | **Zero** `lance-graph-contract` / `ogar-obo` dependency — the 512-byte layout is replicated locally in `pack.rs` as a byte-layout contract, not a code dependency (mirrors `ogar-obo`'s own documented posture) |

Source files (as of `b91d6a1`): `lib.rs` (31 lines), `model.rs` (122 lines,
the YAML-facing disorder struct), `pack.rs` (209 lines, the `NodeRow`
byte-packer + `DISMECH_CONCEPT` constant), `bake.rs` (227 lines, the
directory-walk + parse + pack driver + `dismech_bake` binary). 13 unit
tests, all green at `b91d6a1`.

**Updated 2026-08-18:** `model.rs` gained a permissive `NodeItem`
causal-mechanism-graph typed layer (edge raw structs +
`de_string_or_list`); a new `graph.rs` is a direct port of the real
upstream `src/dismech/graph.py::build_causal_graph`; a new
`bin/census.rs` (`dismech_census`) falsifies it against the full corpus.
22 unit tests green. See the resolved half of "OPEN — causal-mechanism
graph not yet baked" in `TECH_DEBT.md` for the full account, including
what's still open (SoA packing).

## Classid reservation — `0x0333`

`DISMECH_CONCEPT: u16 = 0x0333` (`crates/dismech-bake/src/pack.rs:36`).
Reserved in the shared `0x03` OGAR "Ontology" domain, one slot family past
`ogar-ro`'s relation-body concept (`0x0306`). Measured unused anywhere in
`AdaWorldAPI/OGAR` on 2026-08-17 (`grep -rn "0x0333"` across every crate,
zero hits before this reservation).

**Collision-avoidance context** (OGAR's own `0x03XX` history, per
`crates/ogar-obo/src/registry.rs`): `OBO_CORE` occupies `0x0301`-`0x0305`;
`ogar-ro`'s `RELATION_BODY_CONCEPT_ID` is `0x0306`; a private downstream
consumer holds an odd-stride run from `0x0307` through `0x031D` (live,
with `0x031F` and `0x0321` retired-not-reused); `META_STUDY_SPINE` landed
at `0x0340`-`0x0347` after two prior collisions. `0x0333` sits clear of
all of the above — above the private consumer's run ending `0x0321`,
below `META_STUDY_SPINE`'s `0x0340` — **and is now compile-time
collision-guarded on the OGAR side** via `ogar-dismech`
(https://github.com/AdaWorldAPI/OGAR/pull/274, merged 2026-08-17). See
`.claude/plans/ogar-classid-registration-v1.md` (all items checked).

**Mirrored addressing**: where a disorder resolves to a `MONDO:<num>`
xref (measured 1,957/1,990 = 98.3% of the real corpus), its DisMech-domain
`identity` is set to that SAME numeric — not a fresh ordinal — so a
DisMech row and its MONDO row differ only in `classid`. The remaining
1.7% (33 disorders, no resolvable MONDO xref) get a fallback ordinal from
a disjoint band (`0x0080_0000+`), flagged in-row via `pack.rs`'s
`mondo_mirrored` byte.

## Bake verification (measured, 2026-08-17)

Run against a real `monarch-initiative/dismech` checkout:

- **1,990 / 1,990** disorder YAML files under `kb/disorders/` parsed
- **0 parse errors**
- **98.3%** MONDO-resolution (1,957/1,990 disorders carry a resolvable
  `MONDO:<num>` xref; 33 fall back to the disjoint ordinal band)
- **13/13** unit tests green

## Scope — what's baked, what's not

Baked into the SoA `NodeRow`: disorder **identity only** — `name` /
`description` / `category` / the MONDO xref.

**Resolved (in-memory, not yet packed) 2026-08-18:** the causal-mechanism
graph — `graph::build_causal_graph` resolves the full node/edge graph
(pathophysiology, phenotypes, environmental, genetic, treatments,
biochemical, experimental/animal/computational models, variants) from a
parsed `Disorder`, ported directly from the real upstream
`graph.py::build_causal_graph`, census-verified against the full corpus
(diseases 1995, edges 33,458 — within 0.4% of `MedCare-rs`'s own
33,328 on the same corpus family). It is NOT yet wired into `pack.rs`'s
SoA row output — see `TECH_DEBT.md` for the specific byte-layout
decisions that blocks on.

Still entirely unconsumed: the sibling
`kb/{comorbidities,groupings,hypotheses,modules,surrogate_endpoints}/`
directories. See `CLAUDE.md` "Current scope" section for the full
statement.

## Active / queued work

- **Done 2026-08-17:** `ogar-dismech` OGAR-side classid collision guard
  — https://github.com/AdaWorldAPI/OGAR/pull/274, merged. See
  `.claude/plans/ogar-classid-registration-v1.md` (all items checked).
- **Named goal, not yet built:** S3 sink-in to the shared `lance-graph`
  SoA table, "volume01" hot reload, q2-repository-pattern re-embedding,
  paging/SPOG routing. Today's output is a flat `.soa` blob plus a
  round-trip-verified S3 upload script (`scripts/upload-bake.sh`).
- **Open question, not decided:** whether `MedCare-rs`'s
  `crates/medcare-dismech` (an earlier hand-transcribed causal-graph
  resolver, built against an unconfirmed Python source citation) should
  move here once this repo's bake covers enough of the corpus to be a
  real replacement. See `CLAUDE.md` "Open questions" for the full
  reasoning — not executed yet.

## Board files in this directory

- `LATEST_STATE.md` (this file) — mutable, what exists now
- `PR_ARC_INVENTORY.md` — append-only, one row per merged PR
- `INTEGRATION_PLANS.md` — append-only index of `.claude/plans/*.md`
- `TECH_DEBT.md` — known gaps, not yet fixed
- `EPIPHANIES.md` — append-only, dated findings/corrections
- `AGENT_LOG.md` — append-only, one-writer session record
