# CQL-Roofline (alpha research proposal)

**Status: alpha / proposal. Nothing here is implemented or measured yet.**

## Question

Can a *typed* CQL front end predict, **before running a query**, an upper bound on its throughput and
name the saturated resource (storage, network, CPU), the way the Roofline model does for HPC kernels?

## Why this might be new (and what we could not verify)

* **Roofline** (Williams, Waterman, Patterson, CACM 2009) bounds attainable performance by
  `min(peak compute, bandwidth × arithmetic intensity)`. It is standard in HPC.
* Roofline-style analysis has been **adapted to databases and storage** recently: a study of GPU query
  performance ([arXiv:2302.00734](https://arxiv.org/pdf/2302.00734)) and an SSD-adapted roofline for
  on-disk vector search ([arXiv:2602.23342](https://arxiv.org/pdf/2602.23342)). Both take bytes-per-operation as input.
* **Gap we want to test:** for wide-column stores the bytes-per-operation are *derivable statically* from
  the schema and query, because `cqlc-core` resolves them into a typed IR (column widths, partition/clustering
  restrictions, ranges, limits). We found no work that feeds a typed CQL IR into a roofline model.
  This is an absence in *our* searches, not a proof of novelty; a proper literature review is milestone 0.
* **Schema design is prior art:** NoSE (Mior, Salem, Aboulnaga, Liu; IEEE TKDE 2017) picks Cassandra schemas
  with a cost model and integer programming. CQL-Roofline does not design schemas; it bounds *given* ones,
  and could later supply NoSE-style optimizers with a hardware-grounded cost term.

## Model (sketch)

For a query `q` on table `T`, from the typed IR:

* `w` = estimated bytes per result row (fixed-size types exact; text/blob/collections by a declared
  size hint, e.g. `-- size: body ~ 512`), `R` = rows touched, `a` = read amplification (SSTables
  consulted; a parameter, measured per table).
* Ceilings in rows/s:
  `storage = BW_nvme / (a · w)`, `iops = IOPS_nvme / (a · pages_per_row)`,
  `network = BW_net / w_out`, `cpu = (cores · f) / c_row` (per-row decode cost, calibrated).
* Bound = `min(...)`; the minimizing term is the predicted bottleneck.
* Roofline adds the standard caveat: bandwidth-style bounds hide latency, so point reads get an
  explicit IOPS/latency term.

## Experiments

| | Workload | Measured | 
|---|---|---|
| E1 | point reads, varying row width | attained / bound, bottleneck (iostat, perf) |
| E2 | partition scans, varying partition size and page size | same |
| E3 | bulk ingest, shard-aware vs naive batching | same, plus rows/s/core |

Instrument: a Rust shard-aware loader/scanner (token hashing in batches, per-shard batching) so that
the client is not the bottleneck. Public vendor reports state that a shard-aware driver is essential at
scale (ScyllaDB petabyte benchmark); we found no independent HPC-style ingest study, which is part of the motivation.

## Falsification criteria (decided in advance)

The idea is **dropped or reframed** if, over E1–E3:
1. median attained/bound is below 0.3 (the bound is too loose to be useful), or
2. the predicted bottleneck matches the measured one in fewer than 70 % of cases, or
3. the model needs per-table fitting so heavy that it no longer predicts anything.

## Threats to validity

Single node vs cluster behaviour (cache, compaction, tablets, repair); synthetic data; vendor
tuning; hardware counters differ across platforms. Scaling claims need a real cluster (Slurm
or cloud nodes). Alpha runs on one Apple M3 + a ScyllaDB container and can only support single-node statements.

## Milestones

0. Literature review (Roofline for storage/DB, wide-column cost models, SSD/ScyllaDB performance studies).
1. `cqlz-roofline` crate: IR → `(w, R, pages)` extraction, ceilings, report. Unit-tested on hand-computed cases.
2. Single-node measurements E1–E3 against the criteria above; publish the data, including failures.
3. Cluster runs; decide whether to write it up.

## Output (alpha)

`cqlz-roofline` (Rust, built on `cqlc-core`): `cqlz-roofline explain query.cql --schema schema/ --hw hw.toml`
printing the bound, the bottleneck and the assumptions used.
