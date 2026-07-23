//! Evolutionary / stochastic beam search over conjunctive patterns (SSDP+ style).

use std::cmp::Ordering;
use std::collections::hash_map::Entry;
use std::collections::HashMap;
use std::time::{Duration, Instant};

use rand::rngs::StdRng;
use rand::seq::SliceRandom;
use rand::SeedableRng;

use crate::dataset::Dataset;
use crate::diversity::{diverse_top_k, DiversityConfig};
use crate::metrics::{evaluate, Metric};
use crate::pattern::{ItemSpace, Pattern};

/// Maximum generations to avoid infinite loops if scores oscillate under floating quirks.
const MAX_ITERATIONS: usize = 10_000;

const SCORE_EPS: f64 = 1e-12;

/// Hyperparameters for [`search`].
#[derive(Debug, Clone)]
pub struct SearchConfig {
    /// Number of subgroup descriptions to return.
    pub k: usize,
    /// Objective used when ranking candidate patterns during beam expansion.
    pub metric: Metric,
    /// RNG seed (`StdRng`) controlling item-space shuffle reproducibility.
    pub seed: u64,
    /// Wall-clock limit for the outer search loop (`None` = bounded only by [`MAX_ITERATIONS`]).
    pub max_time_secs: Option<f64>,
    /// When `Some`, beam outputs are filtered with [`diverse_top_k`] before returning top‑`k`.
    pub diversity: Option<DiversityConfig>,
    /// Suppress iteration / stop messages on stderr (e.g. benchmarks).
    pub quiet: bool,
}

/// Total-order comparator: descending score, then ascending canonical key as tiebreaker.
///
/// Using score alone leaves ties unresolved in a stable sort, so which tied patterns survive
/// `truncate` depends on HashMap iteration order (OS-seeded per process).  The canonical key
/// is a deterministic total order over patterns, making the combined comparator unique for
/// every pair and therefore reproducible across runs.
fn score_desc_then_key(a: &(Pattern, f64), b: &(Pattern, f64)) -> Ordering {
    b.1.total_cmp(&a.1)
        .then_with(|| a.0.canonical_key().cmp(&b.0.canonical_key()))
}

fn dedupe_keep_best(candidates: Vec<(Pattern, f64)>) -> Vec<(Pattern, f64)> {
    let mut best: HashMap<Vec<(usize, String)>, (Pattern, f64)> = HashMap::new();
    for (p, s) in candidates {
        let k = p.canonical_key();
        match best.entry(k) {
            Entry::Occupied(mut e) => {
                if s > e.get().1 {
                    e.insert((p, s));
                }
            }
            Entry::Vacant(v) => {
                v.insert((p, s));
            }
        }
    }
    best.into_values().collect()
}

fn beam_capacity(k: usize, population_size: usize) -> usize {
    let scaled = k.saturating_mul(40).max(population_size);
    scaled.min(5000).max(k)
}

/// Runs beam-style evolutionary search: extend patterns with single items, prune, optionally diversify.
///
/// Returns up to `k` `(pattern, score)` pairs sorted by **descending** score.
pub fn search(dataset: &Dataset, config: &SearchConfig) -> Vec<(Pattern, f64)> {
    if config.k == 0 {
        return Vec::new();
    }

    let mut rng = StdRng::seed_from_u64(config.seed);
    let mut item_space = ItemSpace::generate(dataset);
    if item_space.is_empty() {
        return Vec::new();
    }
    item_space.shuffle(&mut rng);

    let mut beam: Vec<(Pattern, f64)> = item_space
        .iter()
        .cloned()
        .map(|item| {
            let mut p = Pattern::new();
            p.add_item(item);
            let s = evaluate(config.metric, &p, dataset);
            (p, s)
        })
        .collect();

    beam.sort_by(score_desc_then_key);

    let cap = beam_capacity(config.k, beam.len());
    beam.truncate(cap);

    let mut best_global = beam.first().map(|(_, s)| *s).unwrap_or(0.0);
    let start = Instant::now();
    let mut iteration: usize = 0;

    loop {
        if let Some(limit) = config.max_time_secs {
            if start.elapsed() > Duration::from_secs_f64(limit) {
                if !config.quiet {
                    eprintln!("search: stopping — time limit {:.3}s exceeded", limit);
                }
                break;
            }
        }

        if iteration >= MAX_ITERATIONS {
            if !config.quiet {
                eprintln!("search: stopping — max iterations {}", MAX_ITERATIONS);
            }
            break;
        }

        iteration += 1;

        let mut candidates: Vec<(Pattern, f64)> = Vec::with_capacity(beam.len() * item_space.len());
        candidates.extend_from_slice(&beam);

        let parents = beam.clone();
        for (parent, _) in parents {
            for item in &item_space {
                if parent
                    .items
                    .iter()
                    .any(|existing| existing.attribute == item.attribute)
                {
                    continue;
                }
                let mut child = parent.clone();
                child.add_item(item.clone());
                let score = evaluate(config.metric, &child, dataset);
                candidates.push((child, score));
            }
        }

        candidates = dedupe_keep_best(candidates);
        candidates.sort_by(score_desc_then_key);
        candidates.truncate(cap);

        let new_best = candidates.first().map(|(_, s)| *s).unwrap_or(0.0);

        if !config.quiet {
            eprintln!("iteration {}, best score {:.12}", iteration, new_best);
        }

        beam = candidates;

        if new_best <= best_global + SCORE_EPS {
            break;
        }

        best_global = new_best;
    }

    beam.sort_by(score_desc_then_key);

    match &config.diversity {
        Some(div_cfg) => diverse_top_k(beam, dataset, config.k, div_cfg),
        None => beam.into_iter().take(config.k).collect(),
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::dataset::Dataset;

    #[test]
    fn search_runs_on_tiny_inline_data() {
        let csv = "a,b,class\n\
                   1,x,p\n\
                   2,y,n\n\
                   3,x,p\n\
                   4,z,n\n";
        let ds = Dataset::from_reader(csv.as_bytes(), ',', "class", "p").expect("dataset");
        let config = SearchConfig {
            k: 2,
            metric: Metric::WRAcc,
            seed: 42,
            max_time_secs: Some(5.0),
            diversity: None,
            quiet: false,
        };
        let out = search(&ds, &config);
        assert!(out.len() <= 2);
        for w in out.windows(2) {
            assert!(w[0].1 + SCORE_EPS >= w[1].1);
        }
    }

    /// Two in-process calls with identical config must return byte-identical results.
    ///
    /// Before the fix, `dedupe_keep_best` iterated a `HashMap` whose order is OS-seeded per
    /// process, so tied patterns survived `truncate` non-deterministically.  After the fix,
    /// `score_desc_then_key` is a total order, so the same output is guaranteed regardless of
    /// the HashMap iteration order.
    ///
    /// The dataset is chosen to produce WRAcc ties (multiple items yield equal coverage counts).
    #[test]
    fn search_is_reproducible_in_process() {
        // 8 rows, 3 binary features — many patterns share the same WRAcc score.
        let csv = "f1,f2,f3,class\n\
                   A,X,0,p\n\
                   A,Y,1,p\n\
                   A,X,0,n\n\
                   B,Y,1,p\n\
                   B,X,0,n\n\
                   B,Y,1,n\n\
                   A,X,1,p\n\
                   B,Y,0,n\n";
        let ds = Dataset::from_reader(csv.as_bytes(), ',', "class", "p").expect("dataset");
        let config = SearchConfig {
            k: 5,
            metric: Metric::WRAcc,
            seed: 0,
            max_time_secs: None,
            diversity: None,
            quiet: true,
        };

        let run1 = search(&ds, &config);
        let run2 = search(&ds, &config);

        assert_eq!(
            run1.len(),
            run2.len(),
            "both runs must return the same number of patterns"
        );
        for (i, ((p1, s1), (p2, s2))) in run1.iter().zip(run2.iter()).enumerate() {
            assert_eq!(
                p1.canonical_key(),
                p2.canonical_key(),
                "rank {}: pattern mismatch between run 1 and run 2",
                i + 1
            );
            assert_eq!(
                s1.to_bits(),
                s2.to_bits(),
                "rank {}: score bits differ between run 1 and run 2",
                i + 1
            );
        }
    }

    /// Two separate process invocations with the same arguments must produce byte-identical stdout.
    ///
    /// Marked `#[ignore]` because it requires the release binary to be present.
    /// Build it first: `cargo build --release`
    #[test]
    #[ignore = "requires `cargo build --release` to be run first"]
    fn search_is_reproducible_cross_process() {
        use std::io::Write;
        use std::process::Command;

        let csv = b"f1,f2,f3,class\n\
                    A,X,0,p\n\
                    A,Y,1,p\n\
                    A,X,0,n\n\
                    B,Y,1,p\n\
                    B,X,0,n\n\
                    B,Y,1,n\n\
                    A,X,1,p\n\
                    B,Y,0,n\n";

        let tmp = std::env::temp_dir().join("ssdp_plus_repro_test.csv");
        {
            let mut f = std::fs::File::create(&tmp).expect("create temp csv");
            f.write_all(csv).expect("write temp csv");
        }

        let bin = std::path::Path::new(env!("CARGO_MANIFEST_DIR"))
            .join("target")
            .join("release")
            .join("ssdp_plus");
        assert!(
            bin.exists(),
            "release binary not found at {}; run `cargo build --release` first",
            bin.display()
        );

        let args = [
            "--dataset",
            tmp.to_str().unwrap(),
            "--target-attr",
            "class",
            "--target-value",
            "p",
            "-k",
            "5",
            "--seed",
            "0",
            "--cache-size",
            "2",
            "--min-similarity",
            "0.10",
        ];

        let out1 = Command::new(&bin)
            .args(args)
            .output()
            .expect("first run failed");
        let out2 = Command::new(&bin)
            .args(args)
            .output()
            .expect("second run failed");

        // Strip the "Total runtime" line because wall-clock differs between runs.
        let filter = |raw: &[u8]| -> Vec<u8> {
            raw.split(|&b| b == b'\n')
                .filter(|line| !line.starts_with(b"Total runtime"))
                .flat_map(|line| line.iter().chain(std::iter::once(&b'\n')))
                .copied()
                .collect()
        };

        assert_eq!(
            filter(&out1.stdout),
            filter(&out2.stdout),
            "stdout differs between two process runs with the same seed"
        );
    }
}
