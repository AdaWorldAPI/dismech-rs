# Plan: `ogar-classid-registration-v1`

**Status:** ACTIVE — nothing checked off yet.
**Repo touched:** `AdaWorldAPI/OGAR` (this is a cross-repo plan; the
checklist items below run in OGAR, not in `dismech-rs`).
**Owning repo for this doc:** `dismech-rs` (`.claude/plans/
ogar-classid-registration-v1.md`), indexed in `.claude/board/
INTEGRATION_PLANS.md`.

## Goal

Register the `0x0333` DisMech classid as a compile-time
collision-guarded reservation in OGAR, the authority repo for the shared
`0x03XX` "Ontology" classid domain — mirroring the pattern `ogar-ro`
already uses for its own `RELATION_BODY_CONCEPT_ID` (`0x0306`).

## Why this is needed (context, not a checklist item)

`DISMECH_CONCEPT: u16 = 0x0333` (`dismech-rs`'s `crates/dismech-bake/
src/pack.rs:36`) was measured clear of every existing OGAR `0x03XX`
registration on 2026-08-17, but that measurement is a point-in-time
`grep`, not an enforced invariant. OGAR's `0x03XX` domain has collided
before (`META_STUDY_SPINE` collided twice before landing at
`0x0340`-`0x0347`). Without a guard, a future OGAR change could silently
claim `0x0333` and corrupt the mirrored-addressing join between DisMech
rows and MONDO rows that `dismech-rs` depends on.

`dismech-rs`'s own `dismech-bake` crate is deliberately zero-dependency
(no `lance-graph-contract`, no `ogar-obo`) — the guard must live entirely
on the OGAR side, the same way `ogar-obo`'s own OBO namespaces (MONDO,
HPO, Uberon, PATO, RO) don't require every consumer to depend on
`ogar-obo` either.

## Checklist

- [ ] Create a new tiny crate `ogar-dismech` in the OGAR workspace,
      mirroring `ogar-ro`'s footprint exactly (same crate layout, same
      minimal deps — likely just `ogar-obo` for the registry types to
      check against).
- [ ] Add `pub const DISMECH_CONCEPT_ID: u16 = 0x0333;` to
      `ogar-dismech`.
- [ ] Add a `concept_id_collision_guard` test module (mirroring
      `ogar-ro`'s own guard) that chains
      `OBO_CORE.specs().iter().chain(META_STUDY_SPINE.specs())` (and any
      other registered `0x03XX` spec sets that exist in OGAR at the time
      this is built) and asserts none claim `0x0333`.
- [ ] Add a second test asserting `DISMECH_CONCEPT_ID` stays inside the
      `0x03` domain and clears both the documented private-consumer run
      (`0x0307`-`0x031D` live, `0x031F`/`0x0321` retired-not-reused) and
      the `META_STUDY_SPINE` band (`0x0340`-`0x0347`).
- [ ] Add `ogar-dismech` to the OGAR workspace `Cargo.toml` members list.
- [ ] Run `cargo test -p ogar-dismech -p ogar-obo -p ogar-ro` in the
      OGAR workspace and confirm all guard tests pass green.
- [ ] Open a PR in `AdaWorldAPI/OGAR` for the new crate.
- [ ] Merge the OGAR PR once green.
- [ ] Come back to `dismech-rs` and update `.claude/board/
      LATEST_STATE.md`'s "Active / queued work" section and
      `TECH_DEBT.md`'s "OPEN — `0x0333` classid is not yet
      collision-guarded" entry to mark this done, with a pointer to the
      merged OGAR PR.

## Non-goals

- `dismech-bake` does **not** gain an OGAR dependency as part of this
  work. It stays zero-dependency for the byte layout; the guard is
  purely an OGAR-side compile-time check that a *future* OGAR change
  cannot silently break the reservation `dismech-rs` already made.
- This plan does not cover baking the causal-mechanism graph or the
  S3/Lance sink-in work — those are tracked separately in
  `TECH_DEBT.md`.
