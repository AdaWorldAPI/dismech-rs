# Plan: `causal-graph-soa-integration-v1`

**Status:** PROPOSED — zero code changes in this document. Every option
below needs an operator call before a line is written (§OPEN OPERATOR
DECISIONS).
**Repo:** `AdaWorldAPI/dismech-rs` (**public**).
**Subject:** land `graph::build_causal_graph`'s output — today resolved
in memory and thrown away — into the 512-byte `NodeRow` SoA, closing the
"OPEN (SoA packing)" half of `.claude/board/TECH_DEBT.md`'s
causal-mechanism-graph entry (TECH_DEBT.md:69-101).

---

## FROZEN DECISIONS

**F1 — The stop was deliberate, and it stays a stop until decided.**
TECH_DEBT.md:69-78 and EPIPHANIES.md:66-72 both record that `pack.rs` was
NOT touched when the resolver landed: a 480-byte value slab and a 16-byte
edge block cannot hold an unbounded per-disorder edge count. The three
byte-layout questions at TECH_DEBT.md:79-97 are answered below as *options
with a recommendation*, never as a decision already taken.

**F2 — Zero-dependency posture.** `crates/dismech-bake/Cargo.toml` declares
exactly `serde` + `serde_yaml`. The 512-byte layout is a **byte-layout
contract, not a code dependency** (`pack.rs:1-6`), mirroring `ogar-obo`'s
documented posture. Default here is to **preserve** it — every new row kind
replicated locally as bytes. Revisiting it (adding `lance-graph-contract`)
is O4: an explicit operator decision, never a silent `Cargo.toml` edit.

**F3 — `0x0333` is reserved and compile-time collision-guarded.**
`DISMECH_CONCEPT: u16 = 0x0333` (`pack.rs:36`), guarded OGAR-side by
`ogar-dismech` (OGAR PR #274, merged, commit `15c5dcb` —
TECH_DEBT.md:18-28). **Every new classid inherits that obligation**: no new
slot is written by a bake before its guard is merged (D-DCG-10). The
`0x03XX` domain has collided before (`META_STUDY_SPINE` twice —
LATEST_STATE.md:47-55).

**F4 — Canon-high classid composition.** `classid = (concept << 16) |
app_prefix` (`pack.rs:34-35`, `bake.rs:57`). The **lo u16 is the app render
prefix and is NEVER a shape ordinal** — no predicate, node type, or edge
kind may be encoded there.

**F5 — Public-repo rules.** No patient data (LATEST_STATE.md:11-14). The
technical shape of DisMech KB content is in scope; the terms, sourcing, or
acquisition of any clinical reference vocabulary are out of scope for every
artifact this repo produces — code, comment, plan, PR body, commit message.
That is a hard boundary, not a style note. No model identifier in any
committed artifact.

**F6 — Additive-only slab discipline** (`pack.rs:88-97`): positions never
reused, tags never renumbered, unrecognized is a distinct code from absent
(proved by `pack.rs:198-208`). Every layout below inherits this verbatim.

---

## INPUT INVENTORY (file:line, each read in this pass)

| What | Where | Fact |
|---|---|---|
| Row stride / offsets | `pack.rs:25,27,29` | `NODE_ROW_STRIDE = 512`, `EDGES_OFFSET = 16`, `VALUE_OFFSET = 32` |
| Key rail | `pack.rs:63-73`, `:78-83` | `classid(4)|HEEL(2)|HIP(2)|TWIG(2)|family(2)|identity(2)`; `family:identity` recombines to a **u32 identity**; HHTL stay `0` in this bake (`pack.rs:52-56`) |
| Edge block, today | `pack.rs:9-13` | 16 bytes, documented "1 byte/predicate slot" — i.e. ~16 bounded-cardinality slots. Written by nothing today (`pack_row`, `pack.rs:100-124`, leaves it zero) |
| Disorder value slab, today | `pack.rs:88-97` | `[0..2)` name len, `[2..4)` desc len, `[4..5)` category tag, `[5..6)` `mondo_mirrored` — 474 bytes still reserved |
| Text is out-of-line already | `pack.rs:95-98`, `bake.rs:9-17` | the row carries lengths+flags; `name`/`description` ride a label lane. **The precedent this plan extends.** |
| Mirrored addressing | `bake.rs:86-103`, `pack.rs:16-22` | MONDO numeric IS the identity; else a fallback ordinal from `FALLBACK_BASE = 0x0080_0000` (`bake.rs:56`), flagged in-row |
| `Edge` (13 fields) | `graph.rs:116-130` | `source`, `target`, `predicate`, `source_type`, `description`, `relationship`, `direction`, `endpoint_context`, `fidelity`, `limitations`, `hypothesis_groups: Vec`, `causal_link_type`, `intermediate_mechanisms: Vec` |
| `NodeInfo` | `graph.rs:100-105` | `node_type`, `description`, `evidence_count`, `curie` (the last two are this crate's additive enrichment, `graph.rs:38-56`) |
| `NodeType` (9) | `graph.rs:68-78`, `:82-94` | pathophysiology, phenotype, environmental, genetic, treatment, biochemical, experimental_model, animal_model, computational_model |
| `CausalGraph` | `graph.rs:164-169` | `nodes: BTreeMap<String,_>` (deterministic), `edges: Vec<_>` (**ordered**), `orphan_targets`, `integrity_issues` |
| Orphan edges are KEPT | `graph.rs:1051-1084` | an edge whose target is not an admitted node stays in `edges`; only the issue lists grow |
| Unbounded lists | `graph.rs:321-333` (`coerce_string_list`), used at `:669,674,701,706` | `hypothesis_groups` / `intermediate_mechanisms` are corpus-ordered `Vec<String>` of arbitrary length |
| Census binary | `src/bin/census.rs:82-98` | already prints nodes-by-type and **edges-by-predicate** — the measurement D-DCG-1 needs exists |
| Census result of record | TECH_DEBT.md:55-59, LATEST_STATE.md:86-91 | **1,995 diseases / 33,458 edges** over 1,996 files (2026-08-18) |

**Predicate set — 19, exhaustively derived from `graph.rs` (verified):**
`causes` (:675) · `leads_to` (:707) · `triggers`/`exacerbates`/
`predisposes_to`/`protects_against`/`modulates`/`influences` (:471-480) ·
`targets` (:770) · `treats` (:784) · `models`/`partially_models`/
`fails_to_model`/`perturbs`/`measures`/`rescues` (:483-493) · `readout`
(:907 and :953 — one predicate, two passes) · `contributes_to` (:987,
:1044) · `variant_of` (:1027).

**`causal_link_type` — FOUR values, measured over `kb/disorders/` this
pass** (not three): `DIRECT` 8,058 · `INDIRECT_UNKNOWN_INTERMEDIATES`
4,150 · `INDIRECT_KNOWN_INTERMEDIATES` 3,825 · `UNKNOWN` 361.
`intermediate_mechanisms` appears in 544 files.

**Corpus drift is real and must be pinned.** The checkout available this
pass carries **1,968** `kb/disorders/*.yaml` files, against 1,990
(2026-08-17, LATEST_STATE.md:72) and 1,996 (census, 2026-08-18). Any
round-trip gate names its snapshot; a census number is only comparable
against the snapshot that produced it.

---

## PROPOSED RESOLUTION

**The constraint, precisely:** one disorder's edges are unbounded across 7
edge-list fields; the row is fixed at 512 bytes with a 16-byte edge block of
1-byte slots. No in-row budget holds them. Three shapes exist — **A**
relations as rows (an edge is addressable), **B** overflow chaining inside
the fixed stride, **C** an out-of-line flat blob keyed by disorder identity
(TECH_DEBT.md:83-85 option (b); edges become data, not addresses).

### Option A — RELATIONS AS ROWS  ★ RECOMMENDED

An edge is its own 512-byte row with its own classid. Its predicate is a
**classid reference**, its endpoints are **key references**. Unbounded edge
counts become unbounded ROWS — which SoA handles natively — rather than
overflow inside a fixed block.

This is the shape the workspace commitment already names: *types exist only
BEFORE the bake; afterwards there are only classes; a relation is a class;
any type that survives the bake is a leak.* A Rust `enum Predicate` consulted
at read time is exactly a type surviving the bake; a predicate **row** is not.

Four row kinds, all 512-byte, all locally packed (F2):

| kind | classid | identity |
|---|---|---|
| disorder (**exists**) | `0x0333` | MONDO numeric, or fallback band |
| graph node | `DISMECH_NODE` (new) | per-(disorder, name) address |
| causal edge | `DISMECH_EDGE` (new) | per-edge address |
| predicate | `DISMECH_PREDICATE` (new) | **frozen ordinal 1..19** |

**A2 (recommended) vs A1.** A1 mints 19 concept slots, one per predicate —
the purest reading of "a relation is a class", but a 19-slot ask in a
domain that has collided twice. A2 mints ONE `DISMECH_PREDICATE` class
whose *rows* are the 19 predicates. Because every reference is stored as a
full **(classid u32, identity u32)** pair, A2 → A1 is later a change of
stored *values*, **not of layout** — the cheap mint that keeps the pure
form reachable without re-baking the row shape.

**N1 (recommended) vs N2.** N1 = one `DISMECH_NODE` classid with
`node_type` as a slab tag; N2 = nine classids. Same argument: cheap and
promotable, since the reference already carries a classid.

#### Proposed edge-row value slab (offsets from `VALUE_OFFSET = 32`)

```
 0..8    source_ref       classid u32 | identity u32
 8..16   target_ref       classid u32 | identity u32   (0,0 when orphan)
16..24   predicate_ref    classid u32 | identity u32
24..32   disorder_ref     classid u32 | identity u32   (the 0x0333 row)
32..33   source_type      u8   0 absent, 1..9 = NodeType (graph.rs:68-78)
33..34   causal_link_type u8   see the table below
34..35   relationship     u8   controlled tag, 255 = unrecognized (prose in lane)
35..36   direction        u8   "
36..37   endpoint_context u8   "
37..38   fidelity         u8   "
38..39   flags            u8   b0 target_resolved · b1 source_resolved
                               b2 description · b3 limitations
                               b4 intermediates_all_resolved
39..40   (reserved, zero)
40..42   description_len               u16
42..44   fidelity_len                  u16   (0 unless fidelity tag == 255)
44..46   limitations_len               u16
46..48   hypothesis_groups_count       u16
48..50   intermediate_mechanisms_count u16
50..52   evidence_count                u16
52..60   lane_offset u64
60..64   lane_len    u32
64..480  RESERVED (zero) — positions never reused (F6)
```

**Node row** slab, same discipline: `[0..8)` `disorder_ref`, `[8..9)`
`node_type` tag, `[9..10)` flags (curie/description present), `[10..12)`
`name_len`, `[12..14)` `description_len`, `[14..16)` `curie_len`,
`[16..18)` `evidence_count` (`graph.rs:100-105`), `[18..26)` `lane_offset`,
`[26..30)` `lane_len`, rest RESERVED. **Predicate row** slab: `[0..2)`
`name_len`, `[2..10)` `lane_offset`, `[10..14)` `lane_len`, rest RESERVED —
its identity is the frozen ordinal and its name is the truth.

#### The side lane (the only variable-length surface)

One append-only blob; a row names its `(offset, len)` window. Fixed field
order inside a window: `description` · `fidelity` (prose only) ·
`limitations` · `hypothesis_groups` (count × `u16 len + bytes`) ·
`intermediate_mechanisms` (count × `u8 kind` + either `u16 len + bytes` for
an unresolved name, or an 8-byte node ref when it resolves to an admitted
node). **Order is preserved** — `coerce_string_list` (`graph.rs:321-333`)
yields corpus order and that order is data. This is the *same* mechanism
`bake.rs:9-17` already uses for name/description text, given a second
customer — not a new one.

#### The disorder row's 16-byte edge block

Under Option A the block stops being a place edges live and becomes what
the V3 `EdgeCodecFlavor` reading says it is: **a class-resolved low-degree
summary**, never the truth. Proposal: **16 saturating u8 counters**, one per
predicate *family*, slots pinned from the D-DCG-1 census (the 19 predicates
fold into ≤16 families: causal, the six environmental, the two treatment,
the six model, readout, genetic). The truth is the edge rows; a consumer
needing an exact degree counts rows. D-DCG-7 gates that saturation is
distinguishable from exactly-255.

### Option B — edge-block overflow chaining  ✗ REJECTED (assessed)

Chain a disorder's edges through continuation rows, head/next pointers in
the 16-byte block. Rejected on five grounds: (1) it reinvents
variable-length storage *inside* a fixed stride — the one thing the
512-byte contract exists to make unnecessary; (2) it consumes the edge
block for pointers, against `pack.rs:9-13`'s documented "1 byte/predicate
slot" meaning and the class-resolved flavor reading; (3) a bulk SoA sweep
becomes a pointer chase, losing the columnar property precisely on the
largest table; (4) fan-out stays unbounded — any fixed continuation arity
is a future re-bake the moment a curated disorder outgrows it, and this
corpus is actively growing (three snapshot sizes on record); (5) round-trip
verification becomes traversal-order-dependent, weakening the D-DCG-5
falsifier that is the point of the exercise. Option A's *lane* is the
honest form of what B attempts in-row.

### Option C — flat edge blob keyed by disorder  △ FALLBACK ONLY

TECH_DEBT.md:83-85's option (b). Simplest, zero new classids. But an edge
then has **no address**: nothing can reference an edge, no ClassView or
field-mask projection applies to one, no predicate can be a class, and the
"types only before the bake" commitment is unmet (the reader re-derives
edge semantics at read time). It is Option A minus the keys. Recommend only
if the operator declines new classid mints (O5).

### Migration steps (Option A, order matters)

1. **Measure first** (D-DCG-1) — no layout pinned before the census.
2. Freeze the address scheme + the 19 predicate ordinals (D-DCG-2),
   permanent from that moment.
3. Land the OGAR-side mints and guards (D-DCG-10) **before** any bake writes
   a new classid (F3).
4. Node → predicate → edge rows (D-DCG-3), then the lane (D-DCG-4).
5. Round-trip falsifier green on the pinned snapshot (D-DCG-5, D-DCG-6).
6. Only then the edge-block summary (D-DCG-7) — a derived hint, never the
   thing verified against. Then driver + `.soa`, determinism, honest stats
   (D-DCG-8).

### `causal_link_type` — the mapping, and the anti-pattern it must not copy

| corpus value | measured | proposed tag |
|---|---|---|
| *absent* | — | `0` |
| `DIRECT` | 8,058 | `1` |
| `INDIRECT_KNOWN_INTERMEDIATES` | 3,825 | `2` |
| `INDIRECT_UNKNOWN_INTERMEDIATES` | 4,150 | `3` |
| `UNKNOWN` | 361 | `4` |
| anything else | 0 today | `255` (unrecognized ≠ absent, per `pack.rs:198-208`) |

`intermediate_mechanisms` rides the lane, in order, whole. Tag `2` without
a populated list, or tag `3` with one, is a bake-time inconsistency worth
counting.

**The anti-pattern, cited precisely.** Upstream's own perturbation CLI
collapses these. `src/dismech/perturb/graph.py:146-151`:

```python
link_type = downstream.get("causal_link_type", "")
relationship = "MEDIATES"
if "DIRECT" in link_type:
    relationship = "DIRECT"
elif "INDIRECT" in link_type:
    relationship = "INDIRECT"
```

Two losses, the second worse than the first and **measured this pass by
evaluating that exact predicate over the four corpus values**: (a) the two
INDIRECT kinds were meant to merge into one `"INDIRECT"`; (b) they do not
even reach it — `"DIRECT" in "INDIRECT_KNOWN_INTERMEDIATES"` is **true**
(substring at index 2), so the `elif` is unreachable and *all three*
DIRECT/INDIRECT_* values classify as `"DIRECT"`. `CausalEdgeEnriched`
(`perturb/graph.py:35-42`) carries no `intermediate_mechanisms` field at
all, so the named intermediates are dropped outright. **Our bake must
preserve all four values and the intermediate list; D-DCG-6 is the gate
that fails if it does not.**

### Evidence fields — where each lands (proposed, not hand-waved)

| field | shape in corpus | lands |
|---|---|---|
| `fidelity` (`graph.rs:822-825`) | short, controlled-ish, model edges only | **tag byte** `[37..38)` when it matches the pinned vocabulary; else `255` + prose in the lane, length at `[42..44)` |
| `limitations` (`graph.rs:826-829`) | free prose | **lane** only; row carries `limitations_len` |
| `evidence_count` (edge level) | `len(evidence[])` | **row**, `[50..52)` u16, saturating |
| `evidence_count` (node level, `graph.rs:100-105`) | same | **node row** slab |
| `evidence[]` item bodies (`model.rs:232-246`: reference, title, supports, source, snippet, explanation) | unbounded prose per edge | **NOT baked in v1** — named in DEFERRED; a v1 bake carries the *count*, never a truncated body, because a truncated citation is worse than an absent one |
| `description` (every pass) | prose | **lane**, `description_len` in-row |
| `relationship`/`direction`/`endpoint_context` | short controlled strings | **tag bytes**, `255` = unrecognized with prose in the lane |
| `hypothesis_groups` | unbounded list | **lane**, count in-row |

Rule behind the split: a **bounded, measured** vocabulary becomes a tag byte
(with a live `255` escape); anything unbounded or prose goes to the lane; a
count a consumer might filter on stays in-row so a scan never touches the
lane. No vocabulary is pinned before D-DCG-1 measures its distinct values.

### MONDO-mirrored addressing, and the 1.7% tail rule

- An edge's **endpoints** reference DisMech NODE rows, not MONDO rows.
  Graph nodes are per-disorder mechanism steps keyed by name
  (`graph.rs:164-169`); they are not MONDO diseases and must not be
  addressed as if they were.
- Mirroring lands on **`disorder_ref`**, which points at the `0x0333` row
  whose identity IS the MONDO numeric for 1,957/1,990 = 98.3% of the corpus
  (`bake.rs:86-103`, LATEST_STATE.md:60-65).
- **The 1.7% tail rule:** an edge row NEVER re-derives or copies the
  mirroring flag — `pack.rs:122` (`VALUE_OFFSET+5`) on the disorder row is
  the single source of truth, and a cross-domain join reads it there. An
  identity `>= 0x0080_0000` (`bake.rs:56`) is a fallback ordinal and **may
  never be published as a cross-domain join key** (`pack.rs:16-22`,
  restated for edges). Bake-time assertion: every `disorder_ref` resolves
  to a row emitted in the same bake; a dangling one is a hard stop.
- Node `curie` (`graph.rs:104`) is HP:/CHEBI:/… — **not** MONDO. Mirroring
  those into `ogar-obo` addresses is DEFERRED below, not assumed.

---

## DELIVERABLES

Each carries a **pre-registered gate** — written before the work, failing
before the work, per this repo's own falsifier discipline.

**D-DCG-1 — Census the packing inputs.** Extend `src/bin/census.rs` (:82-98
already prints most of it): total nodes, max per-disorder edge and node
count, `causal_link_type` values with counts, orphan-target edge count,
per-predicate totals, and the distinct value sets of `relationship` /
`direction` / `endpoint_context` / `fidelity`.
*Gate:* the predicate list is **exactly** the 19 in §INPUT INVENTORY — a
20th is a STOP (resolver or corpus drift), not a widened table. Totals
pinned to a named snapshot alongside the 1,995 / 33,458 of record.

**D-DCG-2 — Freeze the address scheme.** Node identity per (disorder,
name); edge identity per edge; the 19 predicate ordinals, permanent.
*Gate:* (a) a collision test — two distinct (disorder, name) pairs never
share an identity, over the full corpus, measured not asserted; (b) a
determinism test — two bakes produce identical identities, mirroring
`bake.rs:201-218`.

**D-DCG-3 — `pack_node_row` / `pack_edge_row` / `pack_predicate_row`.**
*Gate:* a field-isolation matrix mirroring `pack.rs:181-192` — writing each
slab field perturbs no other field and never the key; plus, for every tag
byte, an `unrecognized ≠ absent` test mirroring `pack.rs:198-208`.

**D-DCG-4 — The side lane, writer + reader.**
*Gate:* an order-preserving round trip — order, count, and bytes identical —
on a **real** corpus fixture carrying **≥2** `intermediate_mechanisms`
entries. Note: `graph.rs`'s existing fixture (:1116-1117, asserted at
:1160-1166) has exactly **one** entry, so it **cannot** falsify an
order-losing implementation; a new multi-entry fixture must be lifted from
the 544 files that carry the field. A one-entry fixture here is the vacuous
case, not the cheap case.

**D-DCG-5 — THE ROUND-TRIP FALSIFIER.** Bake → read the `.soa` + lane back
→ reconstruct a full `CausalGraph` per disorder → diff against a live
`build_causal_graph` over the pinned corpus snapshot.
*Gate:* zero differing disorders; totals equal D-DCG-1 exactly (node count,
edge count, **and every per-predicate bucket**), reconciled against the
1,995 / 33,458 of record for its snapshot.
*Anti-vacuity (mandatory):* a disable-run — zero one edge field in the
packer and show the falsifier goes RED, then restore and show GREEN. A
round trip that compares two empty reconstructions passes for free.

**D-DCG-6 — The INDIRECT_* anti-collapse gate.**
*Gate, three-sided:* (a) a test that FAILS if `INDIRECT_KNOWN_INTERMEDIATES`
and `INDIRECT_UNKNOWN_INTERMEDIATES` pack to the same tag; (b) a
corpus-level assertion that **both** codes are non-zero in the bake
(expected magnitudes 3,825 / 4,150 — a bake reporting 0 for either IS the
collapse, and would otherwise look like a clean run); (c) `UNKNOWN` is its
own code and an unseen value maps to `255`, never into 1..4. Cite
`perturb/graph.py:146-151` in the test's own doc comment so the next reader
sees what is being guarded against.

**D-DCG-7 — The disorder row's 16-byte edge-block summary.** Slots pinned
from D-DCG-1.
*Gate:* saturation is distinguishable from exactly-255; a test asserting
that when any counter saturates the block's sum is **less than** the true
edge-row count — i.e. the block cannot be mistaken for an exact degree.

**D-DCG-8 — Bake driver + `.soa` emission** for the three new row kinds and
the lane, with honest counters mirroring `bake.rs:23-29`.
*Gate:* two runs byte-identical (rows AND lane); every skipped or
unresolvable edge counted, never silently dropped — the failure mode
`bake.rs:183-196` already exists to catch one level up.

**D-DCG-9 — Posture check.**
*Gate:* `Cargo.toml` still declares exactly `serde` + `serde_yaml`; no
`lance-graph-contract` / `ogar-obo` import anywhere; the new modules carry
the byte-layout-contract doc comment (`pack.rs:1-6` pattern).

**D-DCG-10 — OGAR-side mints + collision guards** for `DISMECH_NODE`,
`DISMECH_EDGE`, `DISMECH_PREDICATE`, mirroring
`.claude/plans/ogar-classid-registration-v1.md` exactly.
*Gate:* OGAR-side guard tests green, including band clearance against the
documented `0x03XX` history (LATEST_STATE.md:47-55). **No bake writes a new
classid before its guard is merged** (F3).

---

## NON-GOALS

- **No perturbation / ODE / simulation work.** Not scoped, not started.
- **No `CausalEdge64` emission.** This plan promises none (see below).
- **No upstream PRs** to `monarch-initiative/dismech` — the
  `perturb/graph.py:146-151` finding is cited as an anti-pattern to avoid,
  not as something to file.
- **No change to the resolver's admission logic.** `graph.rs` is a
  line-cited port (EPIPHANIES.md:1-78); a behavioral edit moves the census
  and invalidates D-DCG-5's baseline. If packing appears to need one, STOP.
- **No S3 / Lance-table sink-in, hot reload, or SPOG routing** — separate
  open entry (TECH_DEBT.md:105-113).
- **No consumption of the `kb/` siblings** (see DEFERRED); **no decision on
  moving `MedCare-rs`'s `crates/medcare-dismech`** — explicitly not this
  repo's unilateral call (TECH_DEBT.md:145-154).

---

## OPEN OPERATOR DECISIONS

- **O1** — A2 (one `DISMECH_PREDICATE` class, 19 rows) vs A1 (19 minted
  concept slots). *Recommend A2*; A1 stays reachable without a layout change.
- **O2** — N1 (one `DISMECH_NODE` class + type tag) vs N2 (nine classids).
  *Recommend N1*, same argument.
- **O3** — Edge/node identity: deterministic **ordinal** (collision-free by
  construction, but shifts as the corpus grows) vs **content hash** (stable
  across snapshots, ~1e-4 birthday risk at 33k in 2³², so it needs a
  measured zero-collision gate and a stated failure behaviour). *Recommend
  ordinal for v1*, matching `bake.rs:56`'s precedent; content-hash is a
  later, gated upgrade.
- **O4** — Preserve the zero-dep posture vs revisit it. *Recommend
  preserve* (F2); revisiting is a cross-repo call, never a silent edit.
- **O5** — Classid budget: A2+N1 needs **three** new `0x03XX` slots;
  A1+N2 needs **twenty-nine**.
- **O6** — Edge-block flavor: degree histogram (recommended) vs low-degree
  hot references vs leave zeroed for now.
- **O7** — Do orphan targets (`graph.rs:1051-1084`; count from D-DCG-1) get
  synthetic node rows, or does `target_ref` stay `(0,0)` with the raw name
  in the lane? *Lean: no synthetic rows* — inventing an address for a name
  the corpus never admitted is fabrication.
- **O8** — Does the lane blob ship inside the `.soa` artifact or as a
  sibling file with its own pin?

---

## DEFERRED — missing integration

- **A trajectory-bearing perturbation artifact does not exist upstream.**
  `SimulationResult` is `{variables: dict[str,float], label: str}`
  (`perturb/simulate.py:105-109`) — **steady state only, no time series**.
  The mechanistic path exists only as CLI string-printing over re-parsed
  edges (`perturb/__main__.py:253-265` prints causal chains to stdout), and
  `build_perturbation_graph` (`perturb/graph.py:55-119`) is **dead code on
  the run path**: `__main__.py:13` imports only `extract_causal_edges` and
  `trace_causal_paths`; its only callers are in
  `tests/test_perturb/test_graph.py`. Nothing to bake — a trajectory
  artifact would have to be *built*, not transcoded. Out of scope.
- **`severity_scale` / `threshold` → NARS-truth mapping.**
  `perturb/phenotypes.py:74-96` (`_determine_severity`) turns a value plus a
  `severity_scale` into a severity label; mapping that onto a
  frequency/confidence pair is a real design question with no answer here.
- **The 4-of-22 runnable-model gap.** 22 disorder files declare
  `computational_models:`; the checkout carries **4** runnable models
  (`models/{BIOMD0000000341, BIOMD0000000613, hpt_feedback_axis,
  urate_homeostasis}.xml`, each with a `.config.yaml`). Upstream curation
  work, not a bake.
- **`ogar-obo` / MONDO cross-walk deepening.** Node `curie`s
  (`graph.rs:104`) are HP:/CHEBI:/… — mirroring them into `ogar-obo`
  addresses the way `0x0333` mirrors MONDO is a separate design.
- **Edge- and node-level `evidence[]` bodies** (`model.rs:232-246`) — v1
  bakes counts only; the prose is unbaked by choice, not by oversight.
- **The `kb/` siblings, entirely unconsumed** (LATEST_STATE.md:93-96) —
  measured this pass: `comorbidities` 20, `groupings` 64, `hypotheses` 88,
  `modules` 123, `surrogate_endpoints` 1.
- **Snapshot pinning.** 1,968 vs 1,990 vs 1,996 disorder files across three
  points; D-DCG-5 must name the snapshot it verified. A floating "the
  corpus" is not a baseline.

---

## DOWNSTREAM CONSUMERS

- **`MedCare-rs` (private)** — consumes this public medical-knowledge layer
  alongside `ogar-obo`; the mirrored `0x0333` identity (`pack.rs:16-22`) is
  the join, and edge rows extend that join from identity to mechanism.
- **`lance-graph` `CausalEdge64`** — the DisMech edge row is a **sibling
  representation feeding future readings, NOT a CE64 packer.** The inherited
  session measurement is that no *external-corpus* producer of
  `CausalEdge64` exists in `lance-graph`; the in-repo `CausalEdge64::pack`
  call sites are runtime/cognitive-internal (e.g.
  `crates/lance-graph-planner/src/cache/nars_engine.rs:488`,
  `crates/cognitive-shader-driver/src/driver.rs:483`,
  `crates/p64-bridge/src/lib.rs:540`) plus tests — this plan verified that
  call-site inventory but did not re-derive the internal/external
  classification for each. The V3 ruling retired the CE64
  awareness-mantissa extension. **No CE64 emission is promised here.**
