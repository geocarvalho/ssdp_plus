# SSDP+ (Rust)

[Portuguese version](README_PT.md)

## What is SSDP+?

**SSDP+** is an evolutionary subgroup discovery method for **discriminative pattern mining**: it searches for conjunctions of attribute conditions (subgroups) that correlate with a binary target class. Compared to plain SSDP, SSDP+ adds a **diversity mechanism** so the final top‑`k` patterns are not almost duplicates—using Jaccard similarity on covered instances and a cache‑style admission rule (`ks`, `min_similarity`), so you get a more informative pattern set.

This repository ships a Rust **CLI and library** (`ssdp_plus`) aligned with those ideas.

## Installation

From this directory (the crate root):

```bash
cargo install --path .
```

Development build:

```bash
cargo build --release
```

The binary name is **`ssdp_plus`** (underscore).

## Usage examples

Examples assume you run commands from **this crate directory** (`ssdp_plus/ssdp_plus` in the upstream repo layout). Datasets live under **`data/`**.

### Binary matrix (`data/matrixBinaria-Global-100-p.csv`)

Comma‑separated; target column **`class`**, positive value **`p`**:

```bash
ssdp_plus \
  --dataset data/matrixBinaria-Global-100-p.csv \
  --target-attr class \
  --target-value p \
  --k 5
```

### Alon gene expression (`data/alon-clean50-pn-width-2.CSV`)

Comma‑separated (quoted headers); label column **`y`**, positive value **`p`**:

```bash
ssdp_plus \
  --dataset data/alon-clean50-pn-width-2.CSV \
  --target-attr y \
  --target-value p \
  --k 5 \
  --max-time 600
```

If `alon-clean50-pn-width-2.CSV` is missing, unzip `Bioinformatic.zip` from the upstream `data sets/` folder and copy the file into `data/` (see `data/README.md`).

### Short flags

Same as `--dataset`, `-d`; `--separator` / `-s` for delimiter (`tab` or `\t` for TAB).

## Parameter reference

| Flag / option | Meaning | Default |
|----------------|---------|---------|
| `-d`, `--dataset` | Path to CSV (header = attribute names) | *(required)* |
| `-s`, `--separator` | Field separator (one character, or `tab` / `\t`) | `,` |
| `--target-attr` | Name of the class / label column | *(required)* |
| `--target-value` | Positive class label in that column | *(required)* |
| `-k`, `--k` | Number of subgroups to output | `5` |
| `--metric` | `wracc` or `qg` | `wracc` |
| `--cache-size`, `--ks` | Diversity cache size (`ks`): max “similar” selected patterns a candidate may overlap | `2` |
| `--min-similarity` | Jaccard threshold for treating two patterns as similar | `0.10` |
| `--seed` | RNG seed (reproducibility) | `0` |
| `--max-time` | Wall‑clock limit for search (seconds); omit for no limit | unlimited |

Output is CSV on **stdout**: `Rank,Score,Coverage,Pattern`, followed by a short summary.

## References

- **Original Java implementation (NetBeans):** [SSDPplus](https://github.com/tarcisiodpl/ssdp) — see also the parent repository that contains this Rust port.
- **IEEE paper:** [SSDP+: An Evolutionary Algorithm for Subgroup Discovery with Diversity Control](https://ieeexplore.ieee.org/document/8477855)

## Makefile shortcuts

See `Makefile`: `make test`, `make bench`, `make run-matrix`, `make run-alon`.
