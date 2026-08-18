//! `dismech-bake` — a 100%-truthful transcode of the
//! `monarch-initiative/dismech` disorder corpus into the shared 512-byte
//! SoA `NodeRow` byte layout (classid `0x0333`, canon-high).
//!
//! Consumed by `lance-graph`'s zero-copy loader, sunk into the same Lance
//! table the `AdaWorldAPI/ogar-obo` OBO-core bake lands in, and by anyone
//! else with S3 read access to the `dismech-rs/bakes/<tag>/` pin. Public
//! reference only — never patient data (`monarch-initiative/dismech` is
//! itself CC-BY reference content, no PHI).
//!
//! ## Source, precisely (2026-08-17 — read this before re-cloning)
//!
//! **Source is `https://github.com/monarch-initiative/dismech` — the real
//! upstream.** `AdaWorldAPI/dismech` is a fork whose default branch is
//! STALE and carries no `kb/` corpus directory at all (confirmed by a
//! fresh shallow clone: `app/`, `cache/`, `data/`, `docs/`, `attic/`,
//! `dashboard/` only). Do not clone the fork expecting the corpus; use the
//! monarch-initiative original.
//!
//! ## Scope
//!
//! Two layers, both real now (corrected 2026-08-18 — see
//! `.claude/board/EPIPHANIES.md`; the upstream repo DOES ship a real
//! Python resolver, `src/dismech/graph.py::build_causal_graph`, falsified
//! against 1,903 committed `pathographs/MONDO_*.json` oracle files — an
//! earlier claim that no such resolver exists was wrong):
//!
//! 1. **Disorder identity** (`bake::bake_disorders`, `pack::pack_row`) —
//!    `name` / `description` / `category` / the MONDO xref, packed into
//!    the 512-byte SoA `NodeRow`.
//! 2. **The causal-mechanism graph** (`graph::build_causal_graph`) — a
//!    direct port of the real upstream Python resolver: node admission
//!    across `pathophysiology` / `phenotypes` / `environmental` /
//!    `genetic` / `treatments` / `biochemical` / `experimental_models` /
//!    `computational_models` / `animal_models`, and every edge type it
//!    produces (`downstream`, `sequelae`, `influences_mechanisms`,
//!    `target_mechanisms`, `target_phenotypes`, `readouts`, `reports_on`,
//!    `modeled_mechanisms`, genetic gene-key inference, variant edges).
//!    See `graph.rs`'s own module doc comment for the full port contract.
//!    **Not yet wired into `pack.rs`'s SoA row output** — the 512-byte
//!    row's 16-byte edge block cannot obviously hold an unbounded number
//!    of causal edges per disorder across 7 edge-list fields; that byte-
//!    layout decision needs an explicit call, not a guess, and is
//!    deliberately left open (see `.claude/board/TECH_DEBT.md`).

pub mod bake;
pub mod graph;
pub mod model;
pub mod pack;
