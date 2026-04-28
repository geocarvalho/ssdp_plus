//! Diversity filtering for SSDP+: final pattern sets are not chosen by score alone.

use crate::dataset::Dataset;
use crate::pattern::Pattern;

/// Controls how aggressively the greedy filter rejects patterns similar to ones already chosen.
#[derive(Debug, Clone, PartialEq)]
pub struct DiversityConfig {
    /// Maximum number of already-selected patterns a candidate may be “similar” to
    /// (Jaccard ≥ `min_similarity`) while still being admitted when not fully diverse.
    pub max_similar: usize,
    /// Jaccard similarity threshold on covered instance sets.
    pub min_similarity: f64,
}

/// Greedy diverse selection: scan candidates by descending score and keep up to `k` patterns.
///
/// A candidate is kept if either:
/// - its Jaccard similarity to **every** already-selected pattern is **strictly below** `min_similarity`, or
/// - the count of selected patterns it is similar to (Jaccard ≥ `min_similarity`) is **strictly less than** `max_similar`.
pub fn diverse_top_k(
    mut candidates: Vec<(Pattern, f64)>,
    dataset: &Dataset,
    k: usize,
    config: &DiversityConfig,
) -> Vec<(Pattern, f64)> {
    if k == 0 {
        return Vec::new();
    }

    candidates.sort_by(|a, b| b.1.total_cmp(&a.1));

    let mut selected: Vec<(Pattern, f64)> = Vec::new();

    for (pattern, score) in candidates {
        if selected.len() >= k {
            break;
        }

        let mut similar_count = 0usize;
        let mut all_below = true;
        for (sel_pat, _) in &selected {
            let j = pattern.jaccard_similarity(sel_pat, dataset);
            if j >= config.min_similarity {
                similar_count += 1;
                all_below = false;
            }
        }

        if all_below || similar_count < config.max_similar {
            selected.push((pattern, score));
        }
    }

    selected
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::pattern::Item;

    fn overlap_dataset() -> Dataset {
        let csv = "a,b,class\n\
                   0,0,p\n\
                   0,0,n\n\
                   1,1,p\n\
                   1,1,n\n";
        Dataset::from_reader(csv.as_bytes(), ',', "class", "p").expect("dataset")
    }

    /// Two different items that cover exactly the same rows {0,1}; Jaccard 1.0 between them.
    fn pattern_a_eq_0() -> Pattern {
        let mut p = Pattern::new();
        p.add_item(Item {
            attribute: 0,
            value: "0".into(),
        });
        p
    }

    fn pattern_b_eq_0() -> Pattern {
        let mut p = Pattern::new();
        p.add_item(Item {
            attribute: 1,
            value: "0".into(),
        });
        p
    }

    #[test]
    fn diverse_top_k_rejects_redundant_same_coverage_with_strict_cache() {
        let ds = overlap_dataset();
        // Sorted by score: tie-break order preserved by stable sort — higher score first
        let candidates = vec![(pattern_a_eq_0(), 0.90), (pattern_b_eq_0(), 0.85)];

        let cfg = DiversityConfig {
            max_similar: 1,
            min_similarity: 0.1,
        };

        let out = diverse_top_k(candidates, &ds, 2, &cfg);
        assert_eq!(
            out.len(),
            1,
            "second pattern is fully overlapping (Jaccard 1.0 ≥ 0.1) and similar_count would be 1; \
             need similar_count < max_similar (1), so reject"
        );

        let only = &out[0].0;
        assert!(
            only.items
                .iter()
                .any(|i| i.attribute == 0 && i.value == "0")
                || only
                    .items
                    .iter()
                    .any(|i| i.attribute == 1 && i.value == "0")
        );
    }

    #[test]
    fn diverse_top_k_allows_overlap_when_max_similar_permits() {
        let ds = overlap_dataset();
        let candidates = vec![(pattern_a_eq_0(), 0.90), (pattern_b_eq_0(), 0.85)];

        let cfg = DiversityConfig {
            max_similar: 2,
            min_similarity: 0.1,
        };

        let out = diverse_top_k(candidates, &ds, 2, &cfg);
        assert_eq!(out.len(), 2, "similar_count (1) < max_similar (2)");
    }

    #[test]
    fn diversity_across_three_patterns_respects_score_order() {
        let ds = overlap_dataset();
        let mut p_diff = Pattern::new();
        p_diff.add_item(Item {
            attribute: 0,
            value: "1".into(),
        });

        let candidates = vec![
            (pattern_a_eq_0(), 0.95),
            (pattern_b_eq_0(), 0.90),
            (p_diff, 0.50),
        ];

        let cfg = DiversityConfig {
            max_similar: 1,
            min_similarity: 0.1,
        };

        let out = diverse_top_k(candidates, &ds, 2, &cfg);
        assert_eq!(out.len(), 2);
        // First must be highest-scoring pattern covering {0,1}
        assert_eq!(out[0].1, 0.95);
        // Second cannot be the overlapping duplicate (0.90); must be the diverse lower-score one
        assert_eq!(out[1].1, 0.50);
        let j = out[0].0.jaccard_similarity(&out[1].0, &ds);
        assert!(
            j < 1.0 - 1e-9,
            "selected pair should not be duplicate coverage, jaccard={}",
            j
        );
    }
}
