#![warn(missing_docs)]

//! SSDP+-style **subgroup discovery** in Rust: evolutionary beam search over conjunctive patterns,
//! with optional **diversity filtering** so top‑`k` results are not near-duplicates.
//!
//! # Modules
//! - [`dataset`] — CSV loading and positive/negative instance indices.
//! - [`pattern`] — Items, patterns, and item space generation.
//! - [`metrics`] — WRAcc and QG scores.
//! - [`diversity`] — Greedy diverse top‑`k` selection using Jaccard similarity.
//! - [`search`] — Main beam search entry point [`search::search`].
//!
//! The CLI binary is built from `src/main.rs`. User-facing documentation lives in the crate `README.md`.

pub mod dataset;
pub mod diversity;
pub mod metrics;
pub mod pattern;
pub mod search;
