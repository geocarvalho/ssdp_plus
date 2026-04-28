# SSDP+ (Rust)

[Versão em inglês](README.md)

## O que é SSDP+?

**SSDP+** é um método evolutivo de **descoberta de subgrupos** para **mineração de padrões discriminativos**: busca conjunções de condições nos atributos (subgrupos) que se correlacionem com uma classe alvo binária. Em relação ao SSDP simples, o SSDP+ inclui um **mecanismo de diversidade**, de modo que os `k` melhores padrões finais não sejam quase duplicados—usando similaridade de Jaccard sobre as instâncias cobertas e uma regra de admissão em estilo cache (`ks`, `min_similarity`), produzindo um conjunto de padrões mais informativo.

Este repositório oferece uma **CLI e biblioteca** em Rust (`ssdp_plus`) alinhadas a essa ideia.

## Instalação

A partir deste diretório (raiz do crate):

```bash
cargo install --path .
```

Compilação de desenvolvimento:

```bash
cargo build --release
```

O nome do binário é **`ssdp_plus`** (sublinhado).

## Exemplos de uso

Os exemplos assumem que os comandos são executados **neste diretório do crate** (no repositório original, `ssdp_plus/ssdp_plus`). Os conjuntos de dados ficam em **`data/`**.

### Matriz binária (`data/matrixBinaria-Global-100-p.csv`)

Separado por vírgula; coluna alvo **`class`**, valor positivo **`p`**:

```bash
ssdp_plus \
  --dataset data/matrixBinaria-Global-100-p.csv \
  --target-attr class \
  --target-value p \
  --k 5
```

### Expressão gênica Alon (`data/alon-clean50-pn-width-2.CSV`)

Separado por vírgula (cabeçalhos entre aspas); coluna de rótulo **`y`**, valor positivo **`p`**:

```bash
ssdp_plus \
  --dataset data/alon-clean50-pn-width-2.CSV \
  --target-attr y \
  --target-value p \
  --k 5 \
  --max-time 600
```

Se `alon-clean50-pn-width-2.CSV` não existir, descompacte `Bioinformatic.zip` da pasta `data sets/` do repositório original e copie o arquivo para `data/` (veja `data/README.md`).

### Atalhos de linha de comando

`-d` equivale a `--dataset`; `-s` a `--separator` para o delimitador (`tab` ou `\t` para TAB).

## Referência de parâmetros

| Opção | Significado | Padrão |
|--------|-------------|--------|
| `-d`, `--dataset` | Caminho do CSV (primeira linha = nomes dos atributos) | *(obrigatório)* |
| `-s`, `--separator` | Separador de campos (um caractere, ou `tab` / `\t`) | `,` |
| `--target-attr` | Nome da coluna de classe / rótulo | *(obrigatório)* |
| `--target-value` | Valor da classe **positiva** nessa coluna | *(obrigatório)* |
| `-k`, `--k` | Número de subgrupos a exibir | `5` |
| `--metric` | `wracc` ou `qg` | `wracc` |
| `--cache-size`, `--ks` | Tamanho do cache de diversidade (`ks`): quantos padrões “similares” já selecionados um candidato pode sobrepor no máximo | `2` |
| `--min-similarity` | Limiar de Jaccard para considerar dois padrões similares | `0.10` |
| `--seed` | Semente do gerador aleatório (reprodutibilidade) | `0` |
| `--max-time` | Tempo máximo de busca em segundos; omitir = sem limite | ilimitado |

A saída é CSV em **stdout**: `Rank,Score,Coverage,Pattern`, seguida de um resumo curto.

## Referências

- **Implementação original em Java (NetBeans):** [SSDPplus](https://github.com/tarcisiodpl/ssdp) — o repositório pai também contém este port em Rust.
- **Artigo IEEE:** [SSDP+: An Evolutionary Algorithm for Subgroup Discovery with Diversity Control](https://ieeexplore.ieee.org/document/8477855)

## Atalhos do Makefile

Veja o `Makefile`: `make test`, `make bench`, `make run-matrix`, `make run-alon`.
