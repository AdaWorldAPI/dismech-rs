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
//! ## Scope of this first slice
//!
//! Disorder IDENTITY only — `name` / `description` / `category` / the
//! MONDO xref. Not the causal-mechanism graph: the corpus ships no
//! separate resolver application to transcode against (no `graph.py`
//! exists anywhere in the upstream repo), so there is no behavioral
//! parity oracle to build toward yet — only the YAML's own declared
//! shape, read truthfully.

pub mod bake;
pub mod model;
pub mod pack;
