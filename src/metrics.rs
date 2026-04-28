//! Subgroup quality metrics: Weighted Relative Accuracy (WRAcc) and Qg.

use crate::dataset::Dataset;
use crate::pattern::Pattern;

/// Confusion-style counts for instances covered by a [`Pattern`] versus global class frequencies.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Coverage {
    /// True positives: covered rows that belong to the positive class.
    pub tp: usize,
    /// False positives: covered rows that belong to the negative class.
    pub fp: usize,
    /// Total covered rows (`tp + fp`).
    pub covered: usize,
    /// Dataset-wide count of positive instances.
    pub total_pos: usize,
    /// Dataset-wide count of negative instances.
    pub total_neg: usize,
    /// Total instances (`total_pos + total_neg`).
    pub total: usize,
}

impl Coverage {
    /// Computes coverage and TP/FP using [`Dataset::pos`] membership for positives.
    pub fn compute(pattern: &Pattern, dataset: &Dataset) -> Coverage {
        let covered_indices = pattern.covered_indices(dataset);
        let covered = covered_indices.len();

        let tp = covered_indices
            .iter()
            .filter(|&&idx| dataset.pos.contains(&idx))
            .count();
        let fp = covered.saturating_sub(tp);

        Coverage {
            tp,
            fp,
            covered,
            total_pos: dataset.pos.len(),
            total_neg: dataset.neg.len(),
            total: dataset.records.len(),
        }
    }
}

/// **Weighted Relative Accuracy** (WRAcc): \((|S|/|D|) \cdot (p_S - p_D))\) in empirical form.
///
/// Returns `0.0` if nothing is covered or the dataset is empty.
pub fn wracc(cov: &Coverage) -> f64 {
    if cov.covered == 0 || cov.total == 0 {
        return 0.0;
    }

    let covered_ratio = cov.covered as f64 / cov.total as f64;
    let precision = cov.tp as f64 / cov.covered as f64;
    let prior_pos = cov.total_pos as f64 / cov.total as f64;
    covered_ratio * (precision - prior_pos)
}

/// **Qg** quality: `tp / (fp + 1)` (smoothed ratio of positives among covered rows).
pub fn qg(cov: &Coverage) -> f64 {
    cov.tp as f64 / (cov.fp as f64 + 1.0)
}

/// Selectable objective for [`evaluate`].
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Metric {
    /// Weighted Relative Accuracy.
    WRAcc,
    /// Qg score.
    QG,
}

/// Evaluates `pattern` on `dataset` with the chosen [`Metric`].
pub fn evaluate(metric: Metric, pattern: &Pattern, dataset: &Dataset) -> f64 {
    let cov = Coverage::compute(pattern, dataset);
    match metric {
        Metric::WRAcc => wracc(&cov),
        Metric::QG => qg(&cov),
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::dataset::Dataset;
    use crate::pattern::{Item, Pattern};

    fn tiny_dataset() -> Dataset {
        let csv = "a,b,class\n\
                   1,x,p\n\
                   2,y,n\n\
                   3,x,p\n\
                   4,z,n\n";
        Dataset::from_reader(csv.as_bytes(), ',', "class", "p").expect("dataset")
    }

    #[test]
    fn coverage_compute_counts() {
        let ds = tiny_dataset();
        let mut p = Pattern::new();
        p.add_item(Item {
            attribute: 1,
            value: "x".into(),
        });

        let cov = Coverage::compute(&p, &ds);
        assert_eq!(cov.tp, 2);
        assert_eq!(cov.fp, 0);
        assert_eq!(cov.covered, 2);
        assert_eq!(cov.total_pos, 2);
        assert_eq!(cov.total_neg, 2);
        assert_eq!(cov.total, 4);
    }

    #[test]
    fn wracc_manual_expected() {
        let ds = tiny_dataset();
        let mut p = Pattern::new();
        p.add_item(Item {
            attribute: 1,
            value: "x".into(),
        });

        let cov = Coverage::compute(&p, &ds);
        let score = wracc(&cov);
        assert!((score - 0.25).abs() < 1e-12);
        assert!((evaluate(Metric::WRAcc, &p, &ds) - 0.25).abs() < 1e-12);
    }

    #[test]
    fn wracc_zero_when_uncovered() {
        let ds = tiny_dataset();
        let mut p = Pattern::new();
        p.add_item(Item {
            attribute: 0,
            value: "999".into(),
        });

        let cov = Coverage::compute(&p, &ds);
        assert_eq!(cov.covered, 0);
        assert_eq!(wracc(&cov), 0.0);
    }

    #[test]
    fn qg_manual_expected() {
        let ds = tiny_dataset();
        let mut p_neg = Pattern::new();
        p_neg.add_item(Item {
            attribute: 1,
            value: "y".into(),
        });
        assert!((evaluate(Metric::QG, &p_neg, &ds) - 0.0).abs() < 1e-12);

        let mut p_pos = Pattern::new();
        p_pos.add_item(Item {
            attribute: 1,
            value: "x".into(),
        });
        assert!((evaluate(Metric::QG, &p_pos, &ds) - 2.0).abs() < 1e-12);
    }
}
