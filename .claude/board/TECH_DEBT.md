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

## ~~OPEN — `MedCare-rs`'s `medcare-dismech` crate: move-or-not undecided~~ CORRECTED 2026-08-17

**Dated:** 2026-08-17 (opened and corrected same day)

**What this entry claimed:** that `MedCare-rs` has an earlier
hand-transcribed causal-graph resolver crate (`crates/medcare-dismech`)
that might need moving here.

**Correction:** re-checked directly against the live `MedCare-rs`
checkout — `find /home/user/MedCare-rs -iname "*dismech*"` returns zero
hits, in both `crates/` and the whole tree. No such crate exists. This
entry (and the matching line in `CLAUDE.md`'s "Open questions") was
carried forward from an earlier, apparently mistaken, session claim —
struck here rather than deleted per this repo's TECH_DEBT convention.
There is nothing to move and no decision pending.
