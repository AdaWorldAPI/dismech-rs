# TECH_DEBT.md — dismech-rs

> Known gaps, deliberately not fixed yet, recorded so they aren't
> rediscovered from scratch. Not append-only in the strict sense, but
> entries should stay dated and should be struck through (not deleted)
> once resolved, with a pointer to the PR that resolved them.

---

## ~~OPEN — `0x0333` classid is not yet collision-guarded on the OGAR side~~ RESOLVED 2026-08-17

**Dated:** 2026-08-17 (opened and resolved same day)

**What it was:** `DISMECH_CONCEPT: u16 = 0x0333` (`crates/dismech-bake/
src/pack.rs:36`) was documented only in this repo, with no compile-time
enforcement on the OGAR side against a future collision.

**Resolution:** `ogar-dismech` — a new tiny crate in `AdaWorldAPI/OGAR`
mirroring `ogar-ro`'s `RELATION_BODY_CONCEPT_ID` pattern exactly:
`DISMECH_CONCEPT_ID: u16 = 0x0333` plus a `concept_id_collision_guard`
test module asserting no `ogar_obo::registry::{OBO_CORE,
META_STUDY_SPINE}` row claims it, and a band-clearance test against the
documented odd-stride run (`0x0307`-`0x031D` live, `0x031F`/`0x0321`
retired) and `META_STUDY_SPINE` (`0x0340`-`0x0347`). Merged:
https://github.com/AdaWorldAPI/OGAR/pull/274 (commit `15c5dcb`).
`dismech-bake` stayed zero-dependency, as scoped — the guard lives
entirely on the OGAR (authority) side. Full checklist:
`.claude/plans/ogar-classid-registration-v1.md`.

---

## ~~OPEN — causal-mechanism graph not yet baked~~ RESOLVED (resolver) / OPEN (SoA packing), 2026-08-18

**Dated:** 2026-08-17, updated 2026-08-18

**What it was:** Only disorder identity (name/description/category/MONDO
xref) was baked. Treatments, phenotype edges, gene associations, and the
sibling `kb/{comorbidities,groupings,hypotheses,modules,
surrogate_endpoints}/` directories were unconsumed.

**Resolved 2026-08-18 — the resolver.** `crates/dismech-bake/src/graph.rs`
is a direct, line-for-line port of the real upstream
`src/dismech/graph.py::build_causal_graph` (read in full from a fresh
`/tmp/dismech-up` checkout, not paraphrased) — every node-admission rule
across the 8 sections + `animal_models` + disease/gene-nested `variants`,
every one of the 8 edge-collecting passes (pathophysiology `downstream`,
phenotype `sequelae`, environmental `influences_mechanisms`, treatment
`target_mechanisms`+`target_phenotypes`, the 3 `modeled_mechanisms`
passes, biochemical `readouts`, phenotype `reports_on`, genetic
gene-key-matched `contributes_to`, and variant `variant_of`/
`contributes_to`), and the full gene-key-matching helper family
(`_gene_lookup_keys`, `_descriptor_lookup_keys`, `_name_lookup_key`,
`_genetic_item_infers_mechanism_edges`, `_build_section_lookup`,
`_resolve_descriptor_target`, `iter_variant_items`,
`animal_model_label`). Falsified via `bin/census.rs` against the full
real corpus (`/tmp/dismech-up`, 1,996 files): **diseases 1995** (exact
match to `MedCare-rs`'s own measurement on the same corpus family),
**total edges 33,458** (vs. MedCare-rs's 33,328 — 0.4% apart, plausibly
snapshot drift between the two corpus pulls, not a resolver
discrepancy). `model.rs` also gained a permissive, untyped-first
`NodeItem` layer (+ `CausalEdgeRaw`/`EnvironmentalEdgeRaw`/
`TargetEdgeRaw`/`ReadoutEdgeRaw`/`ModelEdgeRaw`/`EvidenceItemRaw`, and a
`de_string_or_list` deserializer for the corpus's real scalar-or-list
gotcha, verified against `subtypes:` occurring both ways in real data) —
additive, not consumed by `graph.rs` itself (see that module's doc
comment for why: `graph.py`'s own dynamic dict-walk doesn't reduce
cleanly to a fixed struct tree without losing fidelity).

**Still OPEN — SoA packing.** `graph::build_causal_graph`'s output is
NOT wired into `pack.rs`'s 512-byte `NodeRow`. This was a deliberate
stop, not an oversight: `pack.rs`'s value slab is a fixed 480 bytes and
the edge block is a fixed 16 bytes (`pack.rs`'s own module doc: "1
byte/predicate slot", i.e. up to 16 predicate slots) — but one disorder
can carry an UNBOUNDED number of causal edges across 7 edge-list fields
(a single real file can have dozens of `downstream`/`readouts`/
`target_mechanisms` entries; the corpus-wide census above found up to
tens of edges per disorder in the dense cases). Byte-layout decisions
this needs, all requiring an explicit call rather than a guess:

1. **Row-per-disorder vs. row-per-edge.** The current `NodeRow` is one
   row per disorder (identity). Mechanism-graph nodes/edges could be (a)
   additional `NodeRow`s per graph node/edge (own classid slot(s), own
   addressing scheme), or (b) an out-of-line side table keyed by the
   disorder's row identity (mirroring the existing label-lane pattern
   `bake.rs`/`pack.rs` already use for `name`/`description` text).
2. **Edge count is unbounded per disorder** — no fixed in-row byte
   budget can hold it. Whatever the row shape, edges almost certainly
   need to live in an out-of-line lane (a flat edge-list blob, indexed
   by disorder identity + offset/count), not packed into the 16-byte
   edge block that today holds simple predicate-slot bytes for a very
   different (bounded-cardinality) use case.
3. **classid scheme for graph nodes.** Does a `NodeType::Pathophysiology`
   node get its own reserved OGAR concept slot (parallel to `0x0333`
   for the disorder itself), or does it live entirely in the
   out-of-line lane with no independent classid at all (since,
   structurally, these are sub-parts of a disorder, not first-class
   ontology entities with their own cross-domain identity)?

None of these are this session's call — flagged here per this repo's
own "stop and report, don't guess" discipline, same posture as the
resolved classid-collision entry above.

---

## OPEN — S3 sink-in / hot-reload / SPOG routing not built

**Dated:** 2026-08-17

**What:** Today's output is a flat `.soa` blob plus a round-trip-verified
S3 upload script. The shared `lance-graph` SoA table sink-in, "volume01"
hot reload, q2-repository-pattern re-embedding, and paging/SPOG routing
are named goals only.

---

## ~~OPEN — `MedCare-rs`'s `medcare-dismech` crate: move-or-not undecided~~ CORRECTED 2026-08-17, THEN RE-CORRECTED 2026-08-17

**Dated:** 2026-08-17 (opened, "corrected" same day, then re-corrected
same day — see below)

**What this entry originally claimed:** that `MedCare-rs` has an earlier
hand-transcribed causal-graph resolver crate (`crates/medcare-dismech`)
that might need moving here.

**First "correction" (WRONG — do not trust):** claimed the crate does
not exist, based on `find /home/user/MedCare-rs -iname "*dismech*"`
returning zero hits. That check ran against a local `MedCare-rs`
checkout that was, at the time, 193 commits behind `origin/main` — on a
stale branch (`claude/posten-3-stacking`) left over from earlier
investigation in the same session. The check never touched the crate's
real state; it produced a confident-sounding negative from stale data.

**Re-correction (current, verified against a fresh sync to
`origin/main`):** `crates/medcare-dismech` IS real — 5 modules
(`lib.rs`/`model.rs`/`identity.rs`/`graph.rs`/`parity.rs`) + 4 binaries,
~2,000 lines, a genuine from-scratch Rust transcode of the Python
DisMech resolver's `graph.py::build_causal_graph`, with a `parity.rs`
oracle diffing output against 1,870 committed `pathographs/MONDO_*.json`
files. See `CLAUDE.md` "Open questions" for the full re-corrected entry
and its explicit boundary (this repo did not re-verify whether that
parity gate is currently green — that's `MedCare-rs`'s own thing to
confirm).

**Standing lesson, not just for this one fact:** a "zero hits" result
from a filesystem search is only as good as the checkout it ran
against. Verify `git status -sb` / branch-vs-`origin/HEAD` divergence
BEFORE trusting a negative finding, especially one about to be written
into a different, public repository.

**Move-or-not decision:** still genuinely undecided — not this repo's
call to make unilaterally. Tracked as a live open question in
`CLAUDE.md`, not re-opened as a separate TECH_DEBT row (the crate's
existence is now settled; the move decision was never this entry's
subject).
