//! Conjunctive patterns (`attribute = value` items) and item-space generation.

use std::collections::{BTreeSet, HashSet};

use crate::dataset::Dataset;

/// A single literal in a conjunctive subgroup description: column index and value string.
#[derive(Debug, Clone, PartialEq, Eq, Hash)]
pub struct Item {
    /// Index into [`Dataset::attributes`] for the attribute name.
    pub attribute: usize,
    /// Discrete value for that attribute (cell text from the CSV).
    pub value: String,
}

/// Conjunction (AND) of [`Item`] literals; describes a subgroup / rule body.
#[derive(Debug, Clone, Default)]
pub struct Pattern {
    /// Ordered list of literals (typically at most one value per attribute).
    pub items: Vec<Item>,
}

impl Pattern {
    /// Creates an empty pattern (vacuously matches all rows).
    pub fn new() -> Self {
        Pattern { items: Vec::new() }
    }

    /// Appends a literal to this pattern.
    pub fn add_item(&mut self, item: Item) {
        self.items.push(item);
    }

    /// Returns `true` if `record` satisfies **every** item (AND semantics).
    pub fn covers(&self, record: &[String]) -> bool {
        self.items.iter().all(|it| {
            record
                .get(it.attribute)
                .map(|cell| cell == &it.value)
                .unwrap_or(false)
        })
    }

    /// Row indices in `dataset` whose records satisfy [`Pattern::covers`].
    pub fn covered_indices(&self, dataset: &Dataset) -> Vec<usize> {
        dataset
            .records
            .iter()
            .enumerate()
            .filter(|(_, row)| self.covers(row))
            .map(|(i, _)| i)
            .collect()
    }

    /// Jaccard similarity between covered instance sets: \|A ∩ B\| / \|A ∪ B\|.
    ///
    /// Returns `1.0` if both covered sets are empty (convention used here to avoid `0/0`).
    pub fn jaccard_similarity(&self, other: &Pattern, dataset: &Dataset) -> f64 {
        let a: HashSet<usize> = self.covered_indices(dataset).into_iter().collect();
        let b: HashSet<usize> = other.covered_indices(dataset).into_iter().collect();
        if a.is_empty() && b.is_empty() {
            return 1.0;
        }
        let inter = a.intersection(&b).count();
        let uni = a.union(&b).count();
        if uni == 0 {
            1.0
        } else {
            inter as f64 / uni as f64
        }
    }

    /// Canonical, sort-stable key for a pattern: items sorted by attribute index.
    ///
    /// Two patterns that describe the same subgroup (same attributes and values, possibly in
    /// different insertion order) produce identical keys.  Used as a tiebreaker when patterns
    /// have equal quality scores, so that any sort on `(score desc, canonical_key asc)` is a
    /// total order and produces byte-identical output across runs.
    pub fn canonical_key(&self) -> Vec<(usize, String)> {
        let mut key: Vec<(usize, String)> = self
            .items
            .iter()
            .map(|it| (it.attribute, it.value.clone()))
            .collect();
        key.sort_by_key(|x| x.0);
        key
    }

    /// Human-readable rule using dataset column names: `name=value AND ...`.
    ///
    /// Note: This is **not** [`std::string::ToString`]; it requires the dataset for attribute names.
    #[allow(clippy::should_implement_trait)]
    pub fn to_string(&self, dataset: &Dataset) -> String {
        self.items
            .iter()
            .map(|it| {
                let name = dataset
                    .attributes
                    .get(it.attribute)
                    .map(String::as_str)
                    .unwrap_or("?");
                format!("{}={}", name, it.value)
            })
            .collect::<Vec<_>>()
            .join(" AND ")
    }
}

/// Namespace for constructing the universe of single literals from a [`Dataset`].
pub struct ItemSpace;

impl ItemSpace {
    /// Enumerates every `(attribute_index, observed_value)` pair, **excluding** the target column.
    pub fn generate(dataset: &Dataset) -> Vec<Item> {
        let target_idx = dataset
            .attributes
            .iter()
            .position(|a| a == dataset.target_attr.as_str());

        let mut out = Vec::new();
        for (attr_idx, _) in dataset.attributes.iter().enumerate() {
            if Some(attr_idx) == target_idx {
                continue;
            }
            let mut values = BTreeSet::new();
            for row in &dataset.records {
                if let Some(v) = row.get(attr_idx) {
                    values.insert(v.clone());
                }
            }
            for value in values {
                out.push(Item {
                    attribute: attr_idx,
                    value,
                });
            }
        }
        out
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::dataset::Dataset;

    fn tiny_dataset() -> Dataset {
        let csv = "a,b,class\n\
                   1,x,p\n\
                   2,y,n\n\
                   3,x,p\n\
                   4,z,n\n";
        Dataset::from_reader(csv.as_bytes(), ',', "class", "p").expect("dataset")
    }

    #[test]
    fn covers_matching_record() {
        let mut p = Pattern::new();
        p.add_item(Item {
            attribute: 0,
            value: "1".into(),
        });
        assert!(p.covers(&["1".into(), "x".into(), "p".into()]));
        assert!(!p.covers(&["2".into(), "x".into(), "p".into()]));
    }

    #[test]
    fn covers_conjunction_requires_all_items() {
        let mut p = Pattern::new();
        p.add_item(Item {
            attribute: 1,
            value: "x".into(),
        });
        p.add_item(Item {
            attribute: 0,
            value: "3".into(),
        });
        assert!(p.covers(&["3".into(), "x".into(), "p".into()]));
        assert!(!p.covers(&["1".into(), "x".into(), "p".into()]));
    }

    #[test]
    fn jaccard_similarity_known_patterns() {
        let ds = tiny_dataset();
        let mut pa = Pattern::new();
        pa.add_item(Item {
            attribute: 1,
            value: "x".into(),
        });
        let mut pb = Pattern::new();
        pb.add_item(Item {
            attribute: 0,
            value: "2".into(),
        });
        assert_eq!(pa.jaccard_similarity(&pb, &ds), 0.0);

        let mut pb2 = Pattern::new();
        pb2.add_item(Item {
            attribute: 1,
            value: "x".into(),
        });
        assert_eq!(pa.jaccard_similarity(&pb2, &ds), 1.0);

        let mut pc = Pattern::new();
        pc.add_item(Item {
            attribute: 0,
            value: "1".into(),
        });
        assert_eq!(pa.jaccard_similarity(&pc, &ds), 0.5);
    }

    #[test]
    fn to_string_human_readable() {
        let ds = tiny_dataset();
        let mut p = Pattern::new();
        p.add_item(Item {
            attribute: 0,
            value: "1".into(),
        });
        p.add_item(Item {
            attribute: 1,
            value: "x".into(),
        });
        assert_eq!(p.to_string(&ds), "a=1 AND b=x");
    }

    #[test]
    fn item_space_excludes_target() {
        let ds = tiny_dataset();
        let items = ItemSpace::generate(&ds);
        assert!(!items.iter().any(|i| i.attribute == 2));
        assert!(items.iter().any(|i| i.attribute == 0 && i.value == "1"));
        assert!(items.iter().any(|i| i.attribute == 1 && i.value == "x"));
    }
}
