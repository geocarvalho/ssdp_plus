//! End-to-end checks using datasets under `data/` (next to `Cargo.toml`).
//!
//! **Java reference:** `Const.SEEDS[0]` in the original SSDP+ codebase is `179424673`, not `0`.
//! Tests here use **`seed = 0`** for reproducible Rust runs; for parity with the Java executable,
//! switch the seed to match `Const.java`.
//!
//! **Diversity:** With `cache_size` (`ks`) > 1, SSDP+ admits patterns that may be pairwise Jaccard-
//! similar to each other (cache rule). We validate the **greedy admission rule** from
//! [`ssdp_plus::diversity::diverse_top_k`], not the stronger property “all pairs Jaccard ≤ τ”.

use std::fs::File;
use std::path::PathBuf;

use ssdp_plus::dataset::Dataset;
use ssdp_plus::diversity::DiversityConfig;
use ssdp_plus::metrics::{evaluate, Metric};
use ssdp_plus::pattern::Pattern;
use ssdp_plus::search::{search, SearchConfig};

/// Data directory relative to `CARGO_MANIFEST_DIR` (the crate root containing `Cargo.toml`).
fn data_dir() -> PathBuf {
    PathBuf::from(env!("CARGO_MANIFEST_DIR")).join("data")
}

fn load_csv(path: &PathBuf, sep: char, target_attr: &str, target_value: &str) -> Dataset {
    let mut f = File::open(path).unwrap_or_else(|e| panic!("open {:?}: {}", path, e));
    Dataset::from_reader(&mut f, sep, target_attr, target_value)
        .unwrap_or_else(|e| panic!("parse {:?}: {}", path, e))
}

fn standard_search_config(metric: Metric) -> SearchConfig {
    SearchConfig {
        k: 5,
        metric,
        seed: 0,
        max_time_secs: Some(120.0),
        diversity: Some(DiversityConfig {
            max_similar: 2,
            min_similarity: 0.10,
        }),
        quiet: false,
    }
}

/// Replay greedy admission in **output order** (same order as returned by `diverse_top_k`).
fn assert_greedy_diversity_order(patterns: &[Pattern], dataset: &Dataset, div: &DiversityConfig) {
    let mut chosen: Vec<&Pattern> = Vec::new();
    for p in patterns {
        let mut similar_count = 0usize;
        let mut all_below = true;
        for prev in &chosen {
            let j = p.jaccard_similarity(prev, dataset);
            if j >= div.min_similarity {
                similar_count += 1;
                all_below = false;
            }
        }
        assert!(
            all_below || similar_count < div.max_similar,
            "greedy diversity violated: similar_count={}, all_below={}",
            similar_count,
            all_below
        );
        chosen.push(p);
    }
}

fn assert_scores_wracc_range(scores: &[f64]) {
    for &s in scores {
        assert!(
            (-1.0..=1.0).contains(&s),
            "WRAcc out of expected range [-1, 1]: {}",
            s
        );
    }
}

/// Rust baseline top-1 WRAcc on `data/matrixBinaria-Global-100-p.csv`.
const MATRIX_MIN_TOP1_WRACC: f64 = 0.09;

#[test]
fn matrix_binaria_global_e2e_wracc() {
    let path = data_dir().join("matrixBinaria-Global-100-p.csv");
    let ds = load_csv(&path, ',', "class", "p");
    let div_cfg = DiversityConfig {
        max_similar: 2,
        min_similarity: 0.10,
    };

    let cfg = standard_search_config(Metric::WRAcc);
    let results = search(&ds, &cfg);

    assert_eq!(
        results.len(),
        5,
        "expected exactly k=5 subgroups; got {}",
        results.len()
    );

    let scores: Vec<f64> = results.iter().map(|(_, s)| *s).collect();
    assert_scores_wracc_range(&scores);

    assert!(
        results[0].1 >= MATRIX_MIN_TOP1_WRACC,
        "top-1 WRAcc {:.6} below baseline {:.6}",
        results[0].1,
        MATRIX_MIN_TOP1_WRACC
    );

    let patterns: Vec<_> = results.iter().map(|(p, _)| p.clone()).collect();
    assert_greedy_diversity_order(&patterns, &ds, &div_cfg);

    for w in results.windows(2) {
        assert!(w[0].1 >= w[1].1 - 1e-9);
    }
}

/// Calibrated minimum top-1 WRAcc on full `alon-clean50-pn-width-2.CSV`.
const ALON_MIN_TOP1_WRACC: f64 = 0.05;

#[test]
#[ignore = "slow (~minutes): full beam search on alon; run `cargo test -- --ignored`"]
fn alon_clean50_e2e_wracc() {
    let path = data_dir().join("alon-clean50-pn-width-2.CSV");
    assert!(
        path.exists(),
        "missing {:?}; copy from Bioinformatic.zip into data/ (see data/README.md)",
        path
    );

    let ds = load_csv(&path, ',', "y", "p");
    let div_cfg = DiversityConfig {
        max_similar: 2,
        min_similarity: 0.10,
    };

    let cfg = standard_search_config(Metric::WRAcc);
    let results = search(&ds, &cfg);

    assert_eq!(results.len(), 5);

    let scores: Vec<f64> = results.iter().map(|(_, s)| *s).collect();
    assert_scores_wracc_range(&scores);

    assert!(
        results[0].1 >= ALON_MIN_TOP1_WRACC,
        "top-1 WRAcc {:.6} below floor {:.6}",
        results[0].1,
        ALON_MIN_TOP1_WRACC
    );

    let patterns: Vec<_> = results.iter().map(|(p, _)| p.clone()).collect();
    assert_greedy_diversity_order(&patterns, &ds, &div_cfg);
}

#[test]
fn alon_dataset_loads_and_metric_eval_runs() {
    let path = data_dir().join("alon-clean50-pn-width-2.CSV");
    if !path.exists() {
        return;
    }
    let ds = load_csv(&path, ',', "y", "p");
    let mut p = Pattern::new();
    use ssdp_plus::pattern::Item;
    p.add_item(Item {
        attribute: 0,
        value: ds.records[0][0].clone(),
    });
    let s = evaluate(Metric::WRAcc, &p, &ds);
    assert!((-1.0..=1.0).contains(&s));
}
