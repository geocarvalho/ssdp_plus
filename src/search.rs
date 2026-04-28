//! Evolutionary / stochastic beam search over conjunctive patterns (SSDP+ style).

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

fn canonical_key(pattern: &Pattern) -> Vec<(usize, String)> {
    let mut key: Vec<(usize, String)> = pattern
        .items
        .iter()
        .map(|it| (it.attribute, it.value.clone()))
        .collect();
    key.sort_by_key(|x| x.0);
    key
}

fn dedupe_keep_best(candidates: Vec<(Pattern, f64)>) -> Vec<(Pattern, f64)> {
    let mut best: HashMap<Vec<(usize, String)>, (Pattern, f64)> = HashMap::new();
    for (p, s) in candidates {
        let k = canonical_key(&p);
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

    beam.sort_by(|a, b| b.1.total_cmp(&a.1));

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
        candidates.sort_by(|a, b| b.1.total_cmp(&a.1));
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

    beam.sort_by(|a, b| b.1.total_cmp(&a.1));

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
}
