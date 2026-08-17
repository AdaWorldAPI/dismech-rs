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

**Status:** Resolved — `dismech-bake` was built and verified against the
correct upstream. **Confidence:** HIGH (measured against a real checkout,
not inferred).

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

**Status:** OPEN — genuine unresolved contradiction between two dated
entries in this file. **Confidence:** the contradiction itself is HIGH
(both source claims are independently well-evidenced); which claim is
correct is UNKNOWN.
