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
