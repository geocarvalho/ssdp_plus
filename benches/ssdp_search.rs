//! Criterion benchmarks for subgroup search.
//!
//! The original Java SSDP+ timings should be compared on the **same machine** and JVM settings;
//! this bench measures Rust wall-clock for one `search` call.
//!
//! Datasets are read from **`data/`** next to `Cargo.toml`. Full `alon` search can take minutes;
//! some benches cap [`ssdp_plus::search::SearchConfig::max_time_secs`] for bounded runtime.

use std::fs::File;
use std::path::PathBuf;
use std::time::Duration;

use criterion::{black_box, criterion_group, criterion_main, Criterion};
use ssdp_plus::dataset::Dataset;
use ssdp_plus::diversity::DiversityConfig;
use ssdp_plus::metrics::Metric;
use ssdp_plus::search::{search, SearchConfig};

fn data_dir() -> PathBuf {
    PathBuf::from(env!("CARGO_MANIFEST_DIR")).join("data")
}

fn bench_matrix_quick(c: &mut Criterion) {
    let path = data_dir().join("matrixBinaria-Global-100-p.csv");
    if !path.exists() {
        return;
    }
    let mut f = File::open(&path).expect("matrix csv");
    let ds = Dataset::from_reader(&mut f, ',', "class", "p").expect("parse");

    let cfg = SearchConfig {
        k: 5,
        metric: Metric::WRAcc,
        seed: 0,
        max_time_secs: Some(30.0),
        diversity: Some(DiversityConfig {
            max_similar: 2,
            min_similarity: 0.10,
        }),
        quiet: true,
    };

    let mut g = c.benchmark_group("matrix");
    g.measurement_time(Duration::from_secs(3));
    g.bench_function("matrixBinaria_search", |b| {
        b.iter(|| search(black_box(&ds), black_box(&cfg)))
    });
    g.finish();
}

fn bench_alon_capped(c: &mut Criterion) {
    let path = data_dir().join("alon-clean50-pn-width-2.CSV");
    if !path.exists() {
        return;
    }
    let mut f = File::open(&path).expect("alon csv");
    let ds = Dataset::from_reader(&mut f, ',', "y", "p").expect("parse");

    let cfg = SearchConfig {
        k: 5,
        metric: Metric::WRAcc,
        seed: 0,
        max_time_secs: Some(5.0),
        diversity: Some(DiversityConfig {
            max_similar: 2,
            min_similarity: 0.10,
        }),
        quiet: true,
    };

    let mut g = c.benchmark_group("alon");
    g.sample_size(10);
    g.measurement_time(Duration::from_secs(5));
    g.bench_function("alon_search_5s_cap", |b| {
        b.iter(|| search(black_box(&ds), black_box(&cfg)))
    });
    g.finish();
}

criterion_group!(benches, bench_matrix_quick, bench_alon_capped);
criterion_main!(benches);
