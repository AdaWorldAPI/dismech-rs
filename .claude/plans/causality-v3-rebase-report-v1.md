# Causality-V3 Rebase Report (§26 deliverable)

> Repository-grounded. Every number below was measured in this session against
> the working trees, not carried from the brief. Where the brief's §0 "current
> truth" disagrees with the repositories, the repositories win and the
> disagreement is named.

## 0. Executive: three findings that change the plan

**F-A — The supervised benchmark of §11 cannot be scored as written.**
The brief proposes using "the 3,869 known-intermediate cases as supervision"
with Recall@1/3/5/10 and MRR. Measured over the real corpus
(`monarch-initiative/dismech`, `kb/disorders/*.yaml`, 2,100 files):

| quantity | measured |
|---|---|
| `INDIRECT_KNOWN_INTERMEDIATES` edges | **3,978** |
| ...carrying >=1 named intermediate | **2,489** |
| ...with no `intermediate_mechanisms` key at all | **1,489** |
| total intermediate strings | 3,424 (distinct 3,048) |
| strings per edge | mean 1.38, max 4 |
| **strings that are an EXACT node reference** | **45 / 3,048 (1.5%)** |
| string length | median 4 words, mean 5.2, max 39 |

The intermediates are **free-text prose**, not identities:
*"Classical-pathway inhibition yields serum resistance, permitting spirochete
survival during hematogenous dissemination."* There is no identity-typed gold
set, so Recall@k over a mechanism-identity candidate list has nothing to score
against. 1,489 of the 3,978 are `KNOWN_INTERMEDIATES` **in label only** — the
authors asserted intermediates exist and did not write them down.

**F-B — `HoleV3`, `awareness_state`, `unknown_kind` and `EpisodicWitness64`
do not exist as code.** *(NOT a new finding — see "Prior art" below. The
lance-graph board has carried the EW64 half for months.)* Zero symbols across `lance-graph-contract` and the
workspace. `soa_view.rs:272` states it explicitly: *"`EpisodicWitness64` is NOT
YET a code symbol (a queued design)."* The brief's §0 lists it as existing
substrate. §§8-10 and §16 are therefore **new build**, not wiring.

**F-C — CE64 bits 61..63 are `SPARE`, not `ReasoningBand`.** *(Also prior art —
`plans/graphrag-doc-retrieval-soa-integration-v1.md:89` already documents
"spare (61-63)".)* `layout.rs:77`
names them spare/reserved; `ReasoningBand` is an *additive quantized view* over
those bits with **no auto-derivation** ("nothing writes this field except an
explicit `with_reasoning_band()` call"), and it appears only in
`v2_layout_tests.rs` — zero production readers. Treat bits 61..63 as available,
not as an occupied semantic axis.

## 1. Implementation inventory

| capability | home | state |
|---|---|---|
| DisMech transcode | `dismech-rs/crates/dismech-bake` (2,871 LOC: `bake/graph/model/pack/vocab`) | resolver ported, `pack.rs` 512-byte NodeRow, **graph not yet packed** |
| CausalEdge64 | `lance-graph/crates/causal-edge` (`edge.rs` + `layout.rs`) | v1/v2 feature-gated, layout const-asserted |
| DeepNSM-v2 | `lance-graph/crates/deepnsm-v2` (14 modules) | real grammar surface, see §6 |
| AriGraph | **no crate** — referenced from `planner/src/cache/convergence.rs`, `contract/graph_render.rs`, `persona.rs` | see §7 |
| CLAM / CHAODA | `perturbation-sim/src/chaoda.rs`, `lance-graph-osint/src/reader.rs`, `holograph/width_16k` | see §8 |
| spider-rs | `lance-graph-osint/Cargo.toml:28`, feature `spider-crawl`, **optional, off** | see §9 |
| tesseract-rs | separate repo, `tesseract-ogar` executor + `doc.v1` | ingest-ready |
| ontology identity | `lance-graph-ontology/src/namespace_registry.rs`; `contract/dismech_evidence.rs` | see §10 |
| GraphBLAS | **two** implementations: `lance-graph::graph::blasgraph` and `holograph::graphblas` | duplicate type names; only holograph carries a mask operand |

## 2. Brief assumptions: confirmed vs stale

| §0 claim | verdict |
|---|---|
| ~1,996 disease YAMLs | **stale** — 2,100 files in the corpus; 1,995 diseases of record in the bake |
| 33,455 causal edges | **near** — 33,458 of record (`LATEST_STATE.md:88`) |
| link distribution 8,313 / 3,869 / 4,250 / 371 | **stale** — measured **9,073 / 3,978 / 4,539 / 408** (17,998 total) |
| "99.9996% parity" | **not found in dismech-rs.** The in-repo figure is MedCare-rs's 99.4% (1,848/1,860), and `EPIPHANIES.md:141` records that gate's green status as **not independently confirmed** |
| 586 mechanism identities | not re-verified this session |
| MONDO ~99.6% | plausible; dismech-rs records **98.3%** MONDO mirroring in `CLAUDE.md` |
| EpisodicWitness64 available | **false** (F-B) |
| AriGraph provides PPR / BM25 / RRF | **false** — no such functions found (§7) |
| bits 61..63 = ReasoningBand | **misleading** (F-C) |

`4,621 explicit gaps` in §4 becomes **4,947** (4,539 + 408) at measured counts.

## 3. DisMech-rs parity status

- `graph.rs::build_causal_graph` is a direct port of upstream
  `src/dismech/graph.py::build_causal_graph`.
- Falsified by `dismech_census` on the full corpus: **1,995 diseases /
  33,458 edges**, vs MedCare-rs's independent 33,328 — 0.4% apart, explanation
  recorded as plausible-not-confirmed.
- `vocab.rs` (this session) makes the LLM-noise fallbacks visible **without
  changing parity** — `causal_link_type` is a CLOSED 4-value vocabulary,
  0 unmatched over 2,100 files, fail-closed.
- **No committed end-to-end parity gate lives in dismech-rs.** The
  1,848/1,860 gate is in private MedCare-rs against its own pathographs.
  A "99.9996%" figure has no in-repo referent.

## 4. CausalEdge64 / V3 layout (ground truth)

`causal-edge/src/layout.rs` (v2, const-asserted full coverage):

```
0   S:u8      8   P:u8     16  O:u8     24  freq:u8    32  conf:u8
40  causal:3  43  dir:3    46  infer mantissa:i4 (BITS4)
50  plast:3   53  W-slot:6 59  TRUTH:2  61  SPARE:3
```

Two live hazards:
- `edge.rs:175` still defines a **v1 `PLAST_SHIFT = 49`** beside
  `layout.rs:37`'s v2 `50`, feature-switched at every read/write site. This is
  the exact `I-LEGACY-API-FEATURE-GATED` pattern; it is handled, not broken,
  but any new writer must route through `crate::layout::`.
- Bits 59..60 carry **two additive readings** — `TrustTexture` (canonical) and
  `CausalTopology` (ordinal-identical: Crystalline/Solid/Fuzzy/Murky ==
  Direct/KnownInt/UnknownInt/Unknown). DisMech's four link types map 1:1 onto
  `CausalTopology`, already encoded in
  `dismech-bake/data/causal_link_type.tsv` as `bits2`.
- v1 provenance warning: a v1 edge with `temporal >= 512` reads a non-zero
  band from 61..63. Any hydration path needs the version gate.

## 5. Existing Hole-related code

**None.** No `HoleV3`, `struct Hole`, `hole_kind`, `EpistemicHole`,
`AwarenessState`, `KnownUnknown`, `UnknownKnown`, `UnknownUnknown`, or
`unknown_kind` anywhere in the workspace.

Nearest existing carriers to build on rather than beside:
- `contract/dismech_evidence.rs` — `DismechTopology`, `Supports`,
  `EvidenceSource`, `CitationNamespace`, `CitationKey`, `BibliographyRecord`
- `contract/exploration.rs` — `NarsTruth`
- `contract/splat.rs` — `SplatDecision::ScenarioOnly`, `is_evidence_bearing()`
- `contract/counterfactual.rs`, `causal_audit.rs`, `causal_witness.rs`,
  `quorum.rs`, `escalation.rs`

## 6. DeepNSM-v2 entry points for causal grammar

Real, and grammar-shaped rather than search-shaped — the brief's §2 framing is
supported by the code:

- `fsm.rs:118  parse_to_spo(&[Tagged]) -> Vec<Spo>` — the PoS FSM
- `spo.rs:53   Spo::pairs() -> [(u8,u8);3]` — role-pair projection
- `shape.rs:274/293  detect_all_measured / detect_all(&[Spo])` — structural
  shape detection over a triple set
- `reason.rs:114/129  derive_transitive{,_capped}` — closure with an explicit
  cap; `resolve(premise)`
- `wave.rs:185  resolve_at(...)`, `lib.rs:202/214  window_at / window_range`
  — the version-range temporal read
- `basin.rs`, `belief.rs`, `evidence.rs`, `ancestry.rs`, `codebook.rs`

Gap for this arc: `Spo` is `(u8,u8,u8)` over a codebook. Grounding a 5-word
prose intermediate into that alphabet is the missing step, not the FSM.

## 7. AriGraph basin / PPR / community entry points

> ⚠ **THIS SECTION WAS WRONG AND IS RETRACTED (2026-08-21).** It claimed *"the
> assumption is stale — no `ppr`, `personalized_page*`, `bm25`, `rrf`, or
> `community` function exists in the workspace"* and that *"AriGraph appears
> only as narrative in doc comments."* Both statements are false. The
> retraction is kept in place rather than deleted, per append-only discipline.

**AriGraph is a real, substantial subsystem**:
`crates/lance-graph/src/graph/arigraph/` — **15 modules, ~327 KB**, carrying
exactly the organs the brief names:

| module | surface |
|---|---|
| `ppr.rs` | `PersonalizedPageRank` · `personalized_pagerank()` · `score_of` / `ranked` / `top_k` |
| `bm25.rs` | `Bm25Index::{build, score, rank}` |
| `rrf.rs` | `reciprocal_rank_fusion()` · `DEFAULT_RRF_K` |
| `community.rs` | `Communities::{community_of, members}` · `communities()` |
| `markov_soa.rs` | `SpoRanks` · `WaveProjection` · `BundleProvenance` |
| `episodic.rs` | `EpisodeTheses` · `EpisodicBasins` |
| `witness_corpus.rs` | `WitnessCorpus` · `WitnessEntry` · `WitnessIndexCamPq` |
| plus | `retrieval` · `triplet_graph` · `spo_bridge` · `orchestrator` · `sensorium` · `language` · `xai_client` |

So §3 of the brief — *"use them as basin-forming and support-gathering organs;
do not rebuild them"* — is **correct as written**, organs included. My
"the organs it names are not the organs present" was the error.

**The real state is the one the board already records**, and it is a different
claim from mine: `E-ARIGRAPH-IS-AN-ISLAND`. Every module exists and tests
green; **the chain is open at the joints**. Verbatim from the board: *"the most
expensive kind of gap: invisible in green suites (every crate passes; the
system doesn't do the thing) because the integrating seam was never built."*
The unwired task is named `Ee→EW64(hot prefetch)+WitnessCorpus(cold)`;
`HotWitness` is `todo!()` bodies (`witness_tombstone.rs:70`);
`cache/convergence.rs` is the half-built join with p64 drift.

**Consequence for this plan (the important part).** Absent ≠ unwired, and they
have opposite remedies. Absent would mean *build the organs*. Unwired means
**do not build anything new — close the seam.** The rebase report's own
ablation ladder (§12 Level 3 "+ AriGraph basin formation") is therefore not a
build step at all; it is a wiring step against a shipped subsystem, which is
both far cheaper and a completely different risk profile.

## 8. HHTL / CLAM / CHAODA entry points

- HHTL geometry: `contract/hhtl.rs` — `FAN_OUT=16`, `MAX_DEPTH=16`,
  `NiblePath` with `is_ancestor_of`, `common_prefix_depth`, `family_hop_count`,
  `common_ancestor`, `from_guid_prefix_v3`. Complete and const.
- CHAODA: `perturbation-sim/src/chaoda.rs` + two examples
  (`chaoda.rs`, `chaoda_surge_epicenter.rs`).
- CLAM: `lance-graph-osint/src/reader.rs`, `holograph/width_16k/schema.rs`,
  `onebrc-probe` lanes E/G/H/J.

**Blocker for §19 (discontinuity detection as the unknown-unknown path):** the
GraphBLAS layer cannot express masked frontier expansion.
`blasgraph::mxm/mxv` take **no mask operand** — `Descriptor` defines
`Mask/Complement/Replace/Structure` and `complement_mask()`,
`replace_output()`, `structure_mask()` have **zero readers outside
`descriptor.rs`**. `holograph::graphblas::grb_mxv` does take a mask, but
applies it **after** the full multiply (post-filter, not prune), and
double-clones on the non-transposed path. Neither is territory expansion.

## 9. spider-rs / tesseract-rs ingestion readiness

- `spider` is wired as an **optional, default-off** git dep of
  `lance-graph-osint` behind feature `spider-crawl` (+ tokio). Not exercised.
- `lance-graph-osint/examples/ocr_pipeline.rs` already couples OCR to the
  substrate — the closest thing to §5's chain that exists.
- tesseract-rs ships `tesseract_ogar::{decode_image, execute}` → `doc.v1`
  (pages→regions→lines→words, per-word bbox+conf, table cell grids). Its own
  `CLAUDE.md` marks `doc.v1` as *"the OPTIONAL seed a consumer feeds via
  OGAR"* — and records that **no consumer has ever been built**.
- The genuine gap named there: **there is no sentence.** `doc.v1` yields lines,
  which are typographic, while DeepNSM's FSM is per-sentence.
  `tesseract-ogar/src/sentences.rs` exists (assembly + dehyphenation) and
  `reasoning.rs` wires `SentenceReasoner`. So §5's chain is ~80% built and
  never connected end to end.

## 10. Ontology identity / roundtrip substrate

- `lance-graph-ontology/src/namespace_registry.rs` — the CURIE namespace surface.
- `contract/dismech_evidence.rs` — `CitationNamespace::from_prefix`,
  `CitationKey::parse`, `BibliographyRecord::new`.
- `contract/ontology.rs`, `ontology_warrant.rs`, `identity_quad.rs`
  (`IdentityQuad` 4×u24, raw 0 = absent), `ogar_codebook.rs`.
- dismech-rs mirrors MONDO numerically into `identity` (98.3%), fallback band
  `0x0080_0000+` flagged by `pack.rs`'s `mondo_mirrored` byte — so a reader can
  never mistake a fallback for a join key. **This is the §22 advantage and it
  is real.**
- Not verified this session: SNOMED / CUI / GO / CL / ChEBI / NCIT / ELK
  closure availability. SNOMED is MedCare-rs-owned (private) per its CLAUDE.md.

## 11. Smallest possible held-out benchmark

Given F-A, the benchmark must be re-shaped before it can be built. Three
options, with measured headroom:

**Option 1 — exact-reference gold.** Score only the 45 distinct strings that
are exact node references. *Too small to separate ablation levels.*

**Option 2 — independently-grounded gold (recommended).** Ground the 3,048
prose strings to corpus mechanism names by **label matching only** — an
algorithm with no overlap with DeepNSM/AriGraph, so it cannot leak. Measured
headroom over 48,467 distinct corpus names:

| grounding rung | distinct strings | share |
|---|---|---|
| exact (normalized) | 204 | 6.7% |
| a corpus name is a substring | 1,190 | 39.0% |
| token-Jaccard >= 0.5 | 415 | 13.6% |
| **groundable total** | **1,809** | **59.4%** |
| ungrounded prose | 1,239 | 40.6% |

~1,800 distinct gold intermediates over ~2,489 supervised edges is enough to
separate seven ablation levels. **The 40.6% ungrounded residue must be
reported, not dropped** — silently dropping it inflates every Recall@k.

**Option 3 — topology-class benchmark.** Drop identity recovery; predict the
4-way `CausalTopology` of a masked edge. 17,998 labelled examples, zero
grounding needed, but answers a weaker question.

Recommendation: **Option 2 as the primary, Option 3 as the free smoke test**
that can run on day one and calibrate the harness before grounding lands.

Leakage splits (§11 A-E) are all constructible from corpus metadata
(`disorder_identity`, `category`, mechanism `name`) — no blocker there. The
disease-held-out split is the load-bearing one: sibling diseases share
mechanism prose almost verbatim.

## 12. Minimal PR sequence with falsifiers

| PR | content | falsifier (must be red before, green after) |
|---|---|---|
| **R0** | ~~Correct the stale numbers~~ **DONE / partly void.** The lance-graph board was ALREADY correct (`E-DISMECH-CORPUS-CENSUS-1`, 2026-08-20). Only the *brief* was stale. What shipped instead: `E-DISMECH-KNOWN-INTERMEDIATES-ARE-PROSE-NOT-IDENTITIES-1` + the two overstatement fixes. | the oracle-population count in `LATEST_STATE.md` and in `source_knows_intermediates()`'s doc must read 2,489, not 3,978 |
| **R1** | Gold-set builder: prose -> mechanism-name grounding by label match only, emitting per-string rung (exact/substring/jaccard/ungrounded) | ungrounded fraction is *reported*; a run that silently drops it fails a coverage assertion |
| **R2** | Frozen held-out corpus + the five splits, committed as data (never `/tmp`) | disease-held-out split shares zero mechanism names with train; a random split provably does not |
| **R3** | Topology-class smoke benchmark (Option 3), Level 0 structural baseline only | baseline beats the majority-class prior (50.4% DIRECT) by a stated margin, or the harness is wrong |
| **R4** | `HoleV3` carrier: `awareness_state` and `unknown_kind` as **orthogonal** lanes (§9), SoA not AoS | a test constructs `KNOWN_UNKNOWN x CONTEXT` and `UNKNOWN_KNOWN x REPRESENTATION` and proves neither collapses into the other |
| **R5** | DisMech -> CausalEdge64 bits 59..60 hydration, v1-provenance version-gated | a v1 edge with `temporal >= 512` is *refused*, not silently read as a topology |
| **R6** | Masked frontier op in one GraphBLAS (decide which), + a candidate-elimination semiring | expansion into an already-visited cell does **no work**, proven by an op counter, not by output equality |

Everything past R6 (DeepNSM grammar, basins, CHAODA, spider, Wikidata, rig)
stays behind the R3 harness so each level's contribution is measured, per §12.

## Prior art (checked 2026-08-21, before writing anything)

The lance-graph board was searched first. Outcome: **one of the three
"findings" was new; two were already recorded, and one of my own corrections
was itself wrong.**

| item | verdict |
|---|---|
| link distribution 9,073/3,978/4,539/408 | **already on the board** — `E-DISMECH-CORPUS-CENSUS-1` (2026-08-20) carries the identical census. The *brief* was stale; the repository was not. My proposed R0 "correct the stale numbers" was therefore aimed at the wrong target. |
| F-A prose-not-identities | **NEW, and it corrects that same entry** — which claims "the source names the mediators, so they can be hidden and recovery measured". Landed as `E-DISMECH-KNOWN-INTERMEDIATES-ARE-PROSE-NOT-IDENTITIES-1`. |
| F-B EW64 / HoleV3 absent | **prior art, extensively** — `E-ARIGRAPH-IS-AN-ISLAND` ("0 code symbols"), `E-EW64-IS-PREDICTIVE-PREFETCH`, plan `ew64-witness-unification-v1.md` (whose problem statement quotes `soa_view.rs:272` verbatim), `SYNERGY-MAP-S00-S07.md` ("NEW_REQUIRED — the one memory structure to land"). Cite the plan; do not re-file. |
| F-C bits 61..63 spare | **prior art** — `plans/graphrag-doc-retrieval-soa-integration-v1.md:89`. |

**The process lesson:** two of three headline findings dissolved on contact
with a board search that cost one grep. The one that survived did so *because*
it contradicted a specific sentence in a specific existing entry — which is
also what made it worth writing. A finding that merely restates the board is
waste; a finding that corrects it is the whole point.

## Iron falsifiers already at risk

From §24, these are live given the findings above:

- *"unknown intermediates are treated as negative examples"* — the 1,489
  `KNOWN_INTERMEDIATES` edges with **no** named intermediate are neither
  positive nor negative. They must be a third bucket or they poison the gold.
- *"AriGraph is treated as a competitor"* — moot: the named organs do not
  exist. Do not build them under that name to satisfy the brief.
- *"CausalEdge64 becomes a dumping ground for every V3 field"* — bits 61..63
  are the only free space (3 bits). `HoleV3` must NOT try to fit there.
