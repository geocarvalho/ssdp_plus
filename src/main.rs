//! Command-line interface for **SSDP+** subgroup discovery (`ssdp_plus` binary).
//!
//! Parses CSV paths and hyperparameters, runs [`ssdp_plus::search::search`], prints ranked patterns.

use std::fs::File;
use std::path::PathBuf;
use std::time::Instant;

use anyhow::{anyhow, bail, Context, Result};
use clap::Parser;

use ssdp_plus::dataset::Dataset;
use ssdp_plus::diversity::DiversityConfig;
use ssdp_plus::metrics::{Coverage, Metric};
use ssdp_plus::search::{search, SearchConfig};

#[derive(Parser, Debug)]
#[command(name = "ssdp_plus", version, about = "SSDP+ subgroup discovery (Rust)")]
struct Cli {
    /// Path to CSV file (header row = attribute names)
    #[arg(short = 'd', long = "dataset")]
    dataset: PathBuf,

    /// CSV field separator (single character, or `tab` / `\t`)
    #[arg(short = 's', long = "separator", default_value = ",")]
    separator: String,

    /// Name of the target / class column in the header
    #[arg(long = "target-attr")]
    target_attr: String,

    /// Positive class value in the target column
    #[arg(long = "target-value")]
    target_value: String,

    /// Number of subgroups to report
    #[arg(short = 'k', long = "k", default_value_t = 5)]
    k: usize,

    /// Quality metric: wracc | qg
    #[arg(long = "metric", default_value = "wracc")]
    metric: String,

    /// Max similar patterns per subgroup (diversity cache size `ks`)
    #[arg(long = "cache-size", visible_alias = "ks", default_value_t = 2)]
    cache_size: usize,

    /// Jaccard similarity threshold for diversity filtering
    #[arg(long = "min-similarity", default_value_t = 0.10)]
    min_similarity: f64,

    #[arg(long = "seed", default_value_t = 0)]
    seed: u64,

    /// Maximum runtime in seconds (omit for no limit)
    #[arg(long = "max-time")]
    max_time: Option<f64>,
}

fn parse_separator(raw: &str) -> Result<char> {
    let s = raw.trim();
    match s {
        "\\t" | "tab" | "TAB" => Ok('\t'),
        _ => {
            let mut chars = s.chars();
            let Some(c) = chars.next() else {
                bail!("separator must not be empty (use ',' or ';' or tab)");
            };
            if chars.next().is_some() {
                bail!(
                    "separator must be exactly one character, or `tab`/`\t` for tab (got {:?})",
                    raw
                );
            }
            Ok(c)
        }
    }
}

fn parse_metric(raw: &str) -> Result<Metric> {
    match raw.trim().to_ascii_lowercase().as_str() {
        "wracc" => Ok(Metric::WRAcc),
        "qg" => Ok(Metric::QG),
        other => bail!("unknown metric {:?}: expected \"wracc\" or \"qg\"", other),
    }
}

fn main() {
    let started = Instant::now();
    if let Err(e) = run(started) {
        eprintln!("error: {:#}", e);
        std::process::exit(1);
    }
}

fn run(started: Instant) -> Result<()> {
    let cli = Cli::parse();
    let sep = parse_separator(&cli.separator).context("invalid `--separator`")?;
    let metric = parse_metric(&cli.metric).context("invalid `--metric`")?;

    let path_str = cli
        .dataset
        .to_str()
        .ok_or_else(|| anyhow!("dataset path is not valid UTF-8"))?;

    let mut file = File::open(path_str)
        .with_context(|| format!("could not open dataset file {:?}", cli.dataset))?;

    let dataset = Dataset::from_reader(&mut file, sep, &cli.target_attr, &cli.target_value)
        .map_err(|e| anyhow!("{}", e))
        .context("failed to parse CSV")?;

    let cfg = SearchConfig {
        k: cli.k,
        metric,
        seed: cli.seed,
        max_time_secs: cli.max_time,
        diversity: Some(DiversityConfig {
            max_similar: cli.cache_size,
            min_similarity: cli.min_similarity,
        }),
        quiet: false,
    };

    let results = search(&dataset, &cfg);

    println!("Rank,Score,Coverage,Pattern");
    for (rank, (pattern, score)) in results.iter().enumerate() {
        let cov = Coverage::compute(pattern, &dataset);
        let coverage_str = format!("{}/{}", cov.covered, cov.total);
        let desc = pattern.to_string(&dataset);
        println!("{},{:.4},{},{}", rank + 1, score, coverage_str, desc);
    }

    println!();
    let dataset_label = cli
        .dataset
        .file_name()
        .and_then(|s| s.to_str())
        .unwrap_or("(unknown)");
    println!("Dataset: {}", dataset_label);
    println!(
        "  instances: {}, attributes: {}, positives ({}): {}, negatives: {}",
        dataset.records.len(),
        dataset.attributes.len(),
        dataset.target_value,
        dataset.pos.len(),
        dataset.neg.len()
    );
    println!(
        "Parameters: separator={:?}, target_attr={:?}, target_value={:?}, k={}, metric={}, cache_size={}, min_similarity={}, seed={}, max_time={}",
        sep,
        cli.target_attr,
        cli.target_value,
        cli.k,
        cli.metric.trim(),
        cli.cache_size,
        cli.min_similarity,
        cli.seed,
        cli.max_time
            .map(|t| format!("{}s", t))
            .unwrap_or_else(|| "unlimited".into()),
    );
    println!("Total runtime: {:?}", started.elapsed());

    Ok(())
}
