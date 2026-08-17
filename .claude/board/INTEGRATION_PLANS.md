# INTEGRATION_PLANS.md — dismech-rs

> **APPEND-ONLY** index of versioned plans. The active version of a plan
> lives at `.claude/plans/<name>-v<N>.md`; prior versions are retained
> with a `Status` annotation rather than deleted. Consult this file
> before proposing a new plan — most integration concerns for this repo
> may already have a plan on record. Mirrors the convention in
> `lance-graph/.claude/board/INTEGRATION_PLANS.md`.

---

## `ogar-classid-registration-v1` — 2026-08-17

**File:** `.claude/plans/ogar-classid-registration-v1.md`

**Status:** ACTIVE — no checklist items completed yet.

**Summary:** Registers the `0x0333` DisMech classid as a compile-time
collision-guarded reservation in the OGAR repo (authority side), mirroring
the pattern `ogar-ro` already uses for its `RELATION_BODY_CONCEPT_ID`
(`0x0306`). This repo's own `dismech-bake` crate stays zero-dependency —
the guard lives entirely in a new `ogar-dismech` crate on the OGAR side,
the same way `ogar-obo`'s own OBO namespaces don't require every consumer
to depend on `ogar-obo`.

**Why this is tracked here and not just in `TECH_DEBT.md`:** the work
crosses repo boundaries (this repo documents the reservation; OGAR must
enforce it), so it gets a real plan doc with a checklist rather than a
one-line debt entry alone.
