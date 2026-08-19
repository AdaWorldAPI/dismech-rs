# INTEGRATION_PLANS.md — dismech-rs

> **APPEND-ONLY** index of versioned plans. The active version of a plan
> lives at `.claude/plans/<name>-v<N>.md`; prior versions are retained
> with a `Status` annotation rather than deleted. Consult this file
> before proposing a new plan — most integration concerns for this repo
> may already have a plan on record. Mirrors the convention in
> `lance-graph/.claude/board/INTEGRATION_PLANS.md`.

---

## `causal-graph-soa-integration-v1` — 2026-08-19

**File:** `.claude/plans/causal-graph-soa-integration-v1.md`

**Status:** PROPOSED — awaits operator ruling on Options A/B/C (O1) before
any code.

**Summary:** The SoA landing for `graph.rs`'s falsified causal-mechanism
resolver (the OPEN half of TECH_DEBT's "causal-mechanism graph not yet
baked"). Recommends **Option A (relations as rows), sub-variants A2+N1**:
four 512-byte row kinds (disorder `0x0333` / node / edge / predicate),
references as full `(classid u32, identity u32)` pairs, unbounded lists on
a side lane extending the `bake.rs` label-lane precedent; Option B
(edge-block overflow chaining) rejected on five grounds; C = fallback.
D-DCG-1..10 each carry a pre-registered gate, headlined by the full-corpus
round-trip falsifier (bake → read back → reconstruct `CausalGraph` → diff
against the 1,995/33,458 census) with a mandatory disable-run, and
D-DCG-6's three-sided gate that fails if the two `INDIRECT_*` kinds merge.
New measurements banked in the plan: `causal_link_type` has FOUR values
(DIRECT 8,058 / INDIRECT_UNKNOWN 4,150 / INDIRECT_KNOWN 3,825 / UNKNOWN
361); upstream `perturb/graph.py:146-151` is buggier than a two-way merge
— `"DIRECT" in "INDIRECT_KNOWN_INTERMEDIATES"` is True (substring), so its
`elif` is unreachable and ALL link types classify as DIRECT; local
checkout carries 1,968 disorder files vs 1,990/1,996 on record (snapshot
drift the round-trip gate must pin). Zero-dep posture preserved (revisit =
O4); NO CausalEdge64 emission promised (sibling representation). Deferred:
trajectory artifact, severity→NARS-truth, 4-of-22 runnable models, MONDO
cross-walk, five unconsumed `kb/` siblings.

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
