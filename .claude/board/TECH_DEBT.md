# TECH_DEBT.md — dismech-rs

> Known gaps, deliberately not fixed yet, recorded so they aren't
> rediscovered from scratch. Not append-only in the strict sense, but
> entries should stay dated and should be struck through (not deleted)
> once resolved, with a pointer to the PR that resolved them.

---

## OPEN — `0x0333` classid is not yet collision-guarded on the OGAR side

**Dated:** 2026-08-17

**What:** `DISMECH_CONCEPT: u16 = 0x0333` (`crates/dismech-bake/src/
pack.rs:36`) is currently documented ONLY in this repo (`CLAUDE.md` +
`LATEST_STATE.md`). It was measured clear of every existing OGAR `0x03XX`
registration at the time of reservation (`grep -rn "0x0333"` across every
crate in `AdaWorldAPI/OGAR`, zero hits before this reservation), but
nothing on the OGAR side actually *enforces* that — there is no
compile-time test that would fail if some future OGAR change (e.g. a
`META_STUDY_SPINE`-style expansion) accidentally claimed `0x0333`.

**Why it matters:** OGAR's own `0x03XX` domain has a real collision
history — `OBO_CORE` (`0x0301`-`0x0305`), `ogar-ro`'s
`RELATION_BODY_CONCEPT_ID` (`0x0306`), a private downstream consumer's
odd-stride run (`0x0307`-`0x031D` live, `0x031F`/`0x0321`
retired-not-reused), and `META_STUDY_SPINE` (`0x0340`-`0x0347`, which
itself collided twice before landing there). A silent future collision
at `0x0333` would corrupt cross-domain joins (the mirrored-addressing
scheme this repo depends on for the 98.3% MONDO-resolved disorders).

**The fix (not yet built):** `ogar-ro` already has the pattern —
`RELATION_BODY_CONCEPT_ID = 0x0306` plus a `#[cfg(test)] mod
concept_id_collision_guard` asserting no `ogar_obo::registry::{OBO_CORE,
META_STUDY_SPINE}` row claims the same id. Mirror it exactly: a new tiny
OGAR crate `ogar-dismech` with `DISMECH_CONCEPT_ID: u16 = 0x0333` plus an
equivalent guard test. See `.claude/plans/ogar-classid-registration-v1.md`
for the full checklist.

**Scope note:** this repo's `dismech-bake` crate does NOT gain an OGAR
dependency as part of this fix — the guard lives entirely on the OGAR
(authority) side. `dismech-bake` stays zero-dependency for the byte
layout, same as today.

---

## OPEN — causal-mechanism graph not yet baked

**Dated:** 2026-08-17

**What:** Only disorder identity (name/description/category/MONDO xref)
is baked. Treatments, phenotype edges, gene associations, and the
sibling `kb/{comorbidities,groupings,hypotheses,modules,
surrogate_endpoints}/` directories are unconsumed.

**Why it's deferred, not a bug:** deliberate first-pass scoping per
`CLAUDE.md` "Current scope" — the identity bake was verified complete
(1,990/1,990 parsed, 0 errors) before attempting the larger graph.

---

## OPEN — S3 sink-in / hot-reload / SPOG routing not built

**Dated:** 2026-08-17

**What:** Today's output is a flat `.soa` blob plus a round-trip-verified
S3 upload script. The shared `lance-graph` SoA table sink-in, "volume01"
hot reload, q2-repository-pattern re-embedding, and paging/SPOG routing
are named goals only.

---

## OPEN — `MedCare-rs`'s `medcare-dismech` crate: move-or-not undecided

**Dated:** 2026-08-17

**What:** `MedCare-rs` has an earlier hand-transcribed causal-graph
resolver (`crates/medcare-dismech`) built against a Python source
citation that turned out to reference a file that does not exist in the
real `monarch-initiative/dismech` upstream. Recommended path (not yet
executed): re-ground that resolver's behavior against this repo's real
corpus bake once it covers enough of the graph to be a real replacement,
then decide on the move. See `CLAUDE.md` "Open questions" for the full
statement — this is a private-repo-facing decision, tracked here only as
a pointer.
