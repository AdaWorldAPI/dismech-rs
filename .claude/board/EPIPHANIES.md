## 2026-08-19 — E-THE-ORACLE-IS-NOT-THE-INTEGRATION-SURFACE-1

**Status:** RULING (operator, 2026-08-19), applied to
`.claude/plans/causal-graph-soa-integration-v1.md` as an ownership
correction banner + a per-deliverable layer split.

**The ruling:** `dismech-rs` is the **semantic oracle**, not the
integration surface. It exists so that upstream Monarch DisMech semantics
and Rust-transcoded DisMech semantics can be compared, and **its value is
precisely that it remains boring.** A researcher who has never heard of
the wider architecture must be able to open this repo and see *"the Rust
transcode of upstream DisMech"* — not *"a proprietary cognitive
architecture wearing a DisMech costume."* That separation is scientific
value, and it is a hard design constraint, not a style preference.

The boundary: `upstream DisMech → dismech-rs (oracle) → ogar-dismech /
ogar-from-dismech (interpretation bridge) → lance-graph (HHTL / masks /
reasoning)`. Nothing about HHTL, masks, `CausalEdge64`, ontology
hydration, or known-unknown inference belongs in this crate.

**What crossed the line in the plan:** the four 512-byte row kinds, the
edge-row value-slab byte map, the side-lane design, the new classid
mints, the predicate-row ontology, and the 16-byte edge-block summary.
All retained verbatim as the design record the bridge layer inherits —
retracted as *this repo's* deliverables.

**The sharpest instance, measured:**
`OGAR/crates/ogar-dismech/src/lib.rs` **already mints the same 19 causal
predicates** as `FnIndex` consts `0x90..0xA2` (`:141-161`) behind a real
`DisMechVocabulary` (`:190-222`). D-DCG-2 proposed freezing a *second*
numbering `1..19` here — two frozen numberings for one vocabulary, in two
repos, neither aware of the other. Only one may be canonical, and the one
that already shipped wins.

**What survives, and is the valuable half:** the untouched resolver; the
corpus census; four-value `causal_link_type` preservation; the
INDIRECT_KNOWN vs INDIRECT_UNKNOWN anti-collapse gate; intermediate-list
order preservation; snapshot pinning; determinism; the round-trip
falsifier **re-scoped to the oracle's own artifact**; the mandatory
anti-vacuity disable-run; and the upstream substring bug recorded as an
anti-pattern rather than copied. The zero-dep posture is now *load-bearing*
— it is what keeps this crate independently valuable.

**Corpus facts measured this pass** (for the downstream POC, recorded here
because they are oracle observations): 693/1,968 files carry an
`INDIRECT_UNKNOWN_INTERMEDIATES` edge; 300/1,968 carry both kinds;
**3,454 `intermediate_mechanisms` entries, 0 carrying an ontology id** —
100% free prose. Any downstream recovery test must therefore rank over
text, never join on ids.

Cross-ref: lance-graph
`docs/architecture/ARC-B-OWNERSHIP-AND-ADDRESSING-REASSESSMENT.md` §2.

## 2026-08-18 — The causal-mechanism graph resolver, ported directly against the real Python (F4)

**What happened:** F4 of the `MedCare-rs`-authored transcode plan asked
this repo to extend `dismech-bake` from disorder identity to the
causal-mechanism graph, porting from a private sibling's
`medcare-dismech::graph::build_causal_graph` (described in the brief as
a field-for-field port of the upstream Python). Rather than port from a
description of a port, this session went one hop further: the real
upstream `src/dismech/graph.py` was ALREADY present in full at
`/tmp/dismech-up/src/dismech/graph.py` (part of the pre-staged corpus
checkout), so `crates/dismech-bake/src/graph.rs` is a direct,
line-cited port against that file, not a paraphrase of a paraphrase.

**What the brief's high-level shape got right vs. missed:** the section
list (8 sections + `animal_models`), the general edge-list field names
(`downstream`/`sequelae`/`influences_mechanisms`/`target_mechanisms`/
`readouts`/`reports_on`/`modeled_mechanisms`), and the `conforms_to`
scalar-or-list warning (verified real, though observed in this corpus
snapshot only for `subtypes`, not `conforms_to` itself) were all
accurate. What the brief's summary-level `Edge`/`NodeInfo` shape did NOT
capture, found only by reading the real Python in full:

- Treatments produce edges from **two independent sources**:
  `target_mechanisms[]` (bare `targets` edges, no other field populated)
  AND `target_phenotypes[]` (resolved by descriptor match against a
  phenotype name/term lookup, `treats` edges) — the brief's shape
  implied only one treatment edge kind.
- Biochemical `readouts[]` and phenotype `reports_on[]` edges run
  **source/target SWAPPED** relative to every other pass: FROM the
  underlying mechanism TO the biomarker/phenotype (a biomarker is a
  downstream observation, not an upstream cause).
- Genetic items infer `contributes_to` edges via a whole gene-key-
  matching subsystem (`_gene_lookup_keys`/`_descriptor_lookup_keys`/
  `_name_lookup_key`), gated by `_genetic_item_infers_mechanism_edges`
  (an `association`/`relationship_type` word-list check that suppresses
  edges for `MODIFIER`/`BIOMARKER`/`DISPUTED`/`PROTECTIVE`/`UNKNOWN`-
  flavored associations) — none of this is in the brief's shape at all.
- Variants (`variants[]` at disease level, or nested under
  `genetic[].variants[]`) get their own edge-inference pass:
  `variant_of` to a resolved genetic parent, OR (only if no genetic
  parent resolves) `contributes_to` to a matched pathophysiology
  mechanism — a THIRD independent node/edge source the brief never
  mentioned.
- `animal_models[]` nodes admit UNCONDITIONALLY on a resolvable label
  (name, or a `genotype`+`species` fallback via `animal_model_label`) —
  the Python source's OWN doc comment immediately above that code block
  claims a model "only becomes a node if some edge uses it," but the
  code that follows contradicts its own comment and admits every
  labeled model regardless. This port follows the code, not the stale
  comment, and says so in `graph.rs`'s doc comment for the next reader
  who trusts the comment over the code.

**Falsifier result:** `dismech_census` against the full real corpus
(1,996 files) reports **diseases 1995** (exact match to `MedCare-rs`'s
own measurement) and **total edges 33,458** (vs. `MedCare-rs`'s 33,328
— 0.4% apart). Given the resolver was built from the SAME upstream
source both times, independently, and lands within 0.4% on total edge
count while landing EXACTLY on disease count, the most likely
explanation is corpus-snapshot drift between the two pulls (the corpus
is actively curated; `pathographs/` alone grew from 1,870 to 1,903 to
1,905 across three points in this repo's own history per earlier
EPIPHANIES entries), not a resolver behavioral discrepancy — but this is
not independently confirmed against the exact snapshot `MedCare-rs`
measured against, and is recorded as a plausible explanation, not a
proven one.

**Consequence:** `pack.rs` (SoA byte packing) was deliberately NOT
touched — the 480-byte value slab / 16-byte edge block has no obvious
way to hold an unbounded per-disorder edge count across 7 edge-list
fields, and guessing a byte layout for that is exactly what this repo's
"stop and report" discipline exists to prevent. See `TECH_DEBT.md` for
the specific decisions this blocks on.

**Status:** Resolver: RESOLVED, high confidence (falsified against full
corpus, line-cited against real Python). SoA packing: OPEN, deliberately
deferred. **Confidence:** HIGH on the resolver port; MEDIUM on the exact
33,458-vs-33,328 explanation (plausible, not independently confirmed).

---

## 2026-08-18 — RESOLVED: the resolver contradiction. `medcare-dismech` was right; "17 files, tooling only" was wrong.

**Finding:** The 2026-08-17 contradiction between (a) this repo's own
"the upstream repo's Python content (17 files total) is entirely
tooling ... no resolver application to diff against" and (b) the private
sibling `MedCare-rs`'s `crates/medcare-dismech` citing a working
`src/dismech/graph.py::build_causal_graph` falsified against 1,870
pathographs — is resolved. **(b) is correct. (a) is wrong.**

Re-verified this session against a fresh, independent `--depth 1` clone
of `https://github.com/monarch-initiative/dismech`, from scratch, with no
input from either prior claim:

| check | result |
|---|---|
| `src/dismech/graph.py:301` `def build_causal_graph(...)` | **present** |
| Python files repo-wide (`find . -name '*.py' \| wc -l`) | **323** |
| Python files under `src/dismech/` alone | **84** |
| `pathographs/MONDO_*.json` | **1,903** files |
| `"RO:` literals anywhere in `src/` | **1** (data, not a predicate) |
| `src/dismech/export/sepio_export.py` predicate CURIEs | `dismech:has_pathophysiology`, `dismech:causally_upstream_of` — DisMech's own namespace, not RO's |

**Root cause of the wrong claim:** whatever produced "17 files total" for
"the upstream repo's Python content" looked at an incomplete slice of the
tree — nowhere near the real 323 (or even the 84 under `src/dismech/`
alone). The fork-vs-upstream source correction in the entry above this one
is unaffected and still correct; only the *second*, nested claim in that
same entry ("ships no separate resolver application") was wrong.

**Consequence:**
- `crates/dismech-bake/src/model.rs`'s doc comment corrected (this repo,
  same session) — no longer asserts "no separate resolver application."
- `CLAUDE.md`'s "⊘ CONTRADICTED, unresolved" section rewritten to
  "✓ RESOLVED" with this entry's measurements.
- The 99.4% (1,848/1,860) parity figure in `MedCare-rs` remains a
  *historical* result — measured against an earlier, smaller pathograph
  snapshot (1,860) than the current 1,903. Re-running it against the
  current snapshot is a separate task, tracked in `MedCare-rs`'s
  `.claude/plans/dismech-mechanism-bake-v1.md` (F0).
- `dismech-bake`'s narrow identity-only scope (§ model.rs) is unaffected
  by this correction — it was never justified by "no oracle exists," so
  nothing about this crate's current behavior needs to change. Whether to
  extend it toward mechanism-graph parity against the now-confirmed
  resolver is a separate, not-yet-made decision.

**Status:** RESOLVED. **Confidence:** HIGH — measured against a fresh,
independent clone; every number above is directly reproducible with the
commands shown.

---

# EPIPHANIES.md — dismech-rs

> **APPEND-ONLY.** Dated findings, corrections, and "aha" moments.
> Prepend new entries at the top. Never edit a past entry except its
> `**Status:**` / `**Confidence:**` lines; corrections append as new
> dated entries, reversals get their own entry. Mirrors the convention
> in `lance-graph/.claude/board/EPIPHANIES.md`.

---

## 2026-08-17 — Source correction: the AdaWorldAPI fork was stale, the real upstream is `monarch-initiative/dismech`

**Finding:** An initial pass toward building this repo assumed the
source corpus lived in the `AdaWorldAPI/dismech` fork. A fresh shallow
clone of that fork's default branch showed it carries **no `kb/`
corpus directory at all** — only `app/`, `cache/`, `data/`, `docs/`,
`attic/`, `dashboard/`. Attempting to bake against it would have
produced a crate with zero real disorder data and no way to notice,
since an empty or missing directory fails silently rather than loudly
in a naive directory walk.

**Correction:** The real source is the upstream
`https://github.com/monarch-initiative/dismech` repository — Apache-2.0/
CC-BY reference content, no PHI, with the actual corpus at
`kb/disorders/*.yaml` (1,990 files as of 2026-08-17), plus sibling
`kb/{comorbidities,groupings,hypotheses,modules,surrogate_endpoints}/`
directories this repo does not yet consume. `dismech-bake` was built and
verified against a real checkout of the correct upstream: 1,990/1,990
files parsed, 0 parse errors, 98.3% MONDO-resolution.

**A second, related correction surfaced in the same pass:** an earlier
assumption (used to plan a "behavioral parity" transcode strategy, the
kind other AdaWorldAPI transcodes use against a Python reference
implementation) was that the upstream repository contained a separate
Python resolver application — `graph.py`, `build_causal_graph`,
`pathograph_export.py`. It does not. The upstream repo's Python content
(17 files total) is entirely tooling: GH Action helpers, LinkML QC, and
a disease-trajectory extraction skill — no resolver application to diff
against. Consequence: "100% truthful transcode" for this repo means
reading exactly what the YAML corpus declares against its own field
names, not building toward parity with a Python reference that does not
exist. This reframes what "correct" means for `dismech-bake` — there is
no oracle to match, only the corpus's own declared shape to read
faithfully.

**Why this belongs in EPIPHANIES and not just in `CLAUDE.md`:** the
"assumed source A, corrected to source B, and a second wrong assumption
about parity strategy was corrected in the same breath" pattern is
exactly the kind of overconfident-but-ungrounded claim the Reading-
Depth-Ladder / Lie-Detector discipline (see sibling repos' `CLAUDE.md`
files) exists to catch. Recording it here, dated, is the honest trail —
not just the corrected end-state in `CLAUDE.md`, but the fact that a
correction happened and why the first assumption was wrong.

**Status:** Resolved (fork-vs-upstream half) — `dismech-bake` was built and
verified against the correct upstream. **PARTIALLY SUPERSEDED 2026-08-18:**
the second correction in this entry ("no separate resolver application ...
17 files total ... no resolver application to diff against") was itself
wrong — see the 2026-08-18 entry below. **Confidence:** HIGH on the
fork-vs-upstream source correction (measured, unaffected); the "no
resolver" claim is now LOW/false, not HIGH.

---

## 2026-08-17 — Addendum: the "no Python resolver app upstream" half of this
## entry is now contradicted by concrete prior evidence, unresolved

**What surfaced:** a review of this session's own raw transcript (the
usual recourse when repo docs don't have an answer — see sibling repos'
CLAUDE.md files on grepping session transcripts) found that an EARLIER
pass in the SAME session, working in the sibling private repo
`MedCare-rs`, read a real Python resolver application at
`/workspace/dismech/src/dismech/`:

- `graph.py:301-703` — `build_causal_graph`, read in full and transcoded
  line-by-line into `MedCare-rs`'s `crates/medcare-dismech` (merged,
  `AdaWorldAPI/MedCare-rs` PR #530/#531/#532).
- `datamodel/dismech.py` (9,105 lines) and `dismech_pydantic.py`
  (15,948 lines) — LinkML-generated data models.
- `templates/disorder.html.j2:2633-2690` — a D3.js rendering template,
  mirrored server-side in a `cx2_export` module.
- `yaml_io.py:41-53` — the YAML loader.

That transcode was falsified against **1,870 committed
`pathographs/MONDO_*.json` files** (the Python resolver's own output) and
reached **99.4% full-corpus parity (1,848/1,860)**, climbing from 0.5% as
the falsifier caught and fixed 7 real resolver bugs — a large, measured,
real result, not a speculative citation.

**This directly contradicts** the finding two sections above in this same
file: "the upstream repo's Python content (17 files total) is entirely
tooling... no resolver application to diff against."

**Not resolved here, deliberately.** Two explanations are both plausible
and neither is verified:
1. The earlier `/workspace/dismech` checkout (used for the
   `medcare-dismech` work) and the later fresh clone of
   `https://github.com/monarch-initiative/dismech` (used for THIS repo's
   `dismech-bake`, and for the "17 files, tooling only" finding above)
   were different repos, different branches, or different points in
   time — e.g. the earlier work may have checked out the stale
   `AdaWorldAPI/dismech` fork (which had an `app/` directory, consistent
   with a Flask/resolver app) rather than the real upstream, or the real
   upstream's resolver app was later split into a separate repository.
2. The "17 files, tooling only" finding in this repo was itself
   incomplete — a shallow clone or a directory-walk that missed the
   `src/dismech/` subtree.

**What would resolve it:** a fresh `git log`/`git remote -v` check
against whatever checkout produced `/workspace/dismech/src/dismech/
graph.py` (not currently present in this container to inspect), compared
against a fresh `monarch-initiative/dismech` clone's actual file tree —
neither performed in this pass. Flagged rather than guessed, per this
file's own stated discipline.

**Status:** RESOLVED 2026-08-18 — see the new entry below. The
`medcare-dismech` claim (resolver exists, `graph.py::build_causal_graph`,
1,903 pathograph oracle) was correct. The "17 files, tooling only" claim
was the incomplete one — explanation 2 above is what happened.
**Confidence:** HIGH (re-verified against a fresh, independent clone of
the real upstream, full file-tree count and grep, not inferred).
