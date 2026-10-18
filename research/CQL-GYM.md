# CQL-Gym: verifier-in-the-loop reinforcement learning for Cassandra data modelling (alpha research proposal)

**Status: alpha / proposal. Nothing here is implemented or measured yet.**

## Question

Cassandra data modelling means choosing denormalised tables so that every query of a workload is a cheap,
single-partition read. Can a small neural policy, trained by self-play against `cqlc-core` as an
**exact verifier**, find good table sets faster than exact integer programming, and keep working at
workload sizes where the integer program stops being solvable?

## Why this fits the family and HPC

* **Exact reward for free.** `cqlc-core` decides in microseconds whether a query is valid and warning-free
  against a proposed schema (no `ALLOW FILTERING`, full partition key, no clustering gaps). A design that
  fails is simply masked out, so *every proposed design is valid by construction*. No labelled data, no hallucination.
* **Throughput is the bottleneck, so it is an HPC problem.** Self-play needs millions of environment
  steps. Our own benchmark runs lex + parse + schema resolution + type check of 36 queries in about 103 µs
  (≈ 3 µs per query, single core, Apple M3), which suggests an environment step in the
  10–100 µs range and 10^4–10^5 steps/s/core. That is an extrapolation to be measured, not a result.
  The plan is a vectorised, allocation-free Rust environment in the spirit of GPU-accelerated RL
  simulators, scaled across all cores, with learning on a GPU (wgpu).
* **Stack already exists:** Rust environment on `cqlc-core`; policy in Burn (we already have a small
  Transformer encoder over lexer tokens in `cql-ai-model`); designs can be emitted as `cqlx` entities.

## Related work (cited from memory, to be verified in milestone 0)

* **NoSE** (Mior, Salem, Aboulnaga, Liu; IEEE TKDE 2017): Cassandra schema selection by binary integer
  programming. This is the baseline to beat or match, and the source of the problem formulation.
* **Neural combinatorial optimization**: Bello et al., *Neural Combinatorial Optimization with
  Reinforcement Learning* (2016); Kool, van Hoof, Welling, *Attention, Learn to Solve Routing Problems!* (ICLR 2019).
  They train attention policies by RL on combinatorial problems; our twist is an exact, cheap constraint
  checker that comes from a real compiler front end.
* **Throughput-oriented RL environments**: Freeman et al., *Brax* (2021), as precedent for making the
  environment, not the learner, the scalable part.

We did not find prior work applying learned policies to Cassandra schema design (our searches only, not a
literature review).

## Formulation (sketch)

* **Instance:** entities, attributes, and a workload of queries (equality/range predicates, ordering, projections, weights).
* **State:** the tables chosen so far and which queries are already served. **Observation:** token sequence
  from the `cqlc` lexer plus role tokens, same encoding family as `cql-ai-model`.
* **Action:** add a table `(partition cols, clustering cols + order, included cols)`, or assign a query to an
  existing table. Invalid actions are masked using `cqlc-core`.
* **Reward:** `-(read cost + write amplification + storage)` from an explicit analytical cost model, with a
  penalty per unserved query. The cost model is **synthetic at first**; later it can be calibrated with measured
  numbers from the CQL-Roofline proposal and a real ScyllaDB.
* **Learner:** group-relative policy gradient (no critic) over parallel rollouts; behaviour-clone from the
  greedy "one table per query" solution as a warm start.

## Experiments

| | Setting | Compared against | Measured |
|---|---|---|---|
| E1 | workloads of 5–50 queries (ILP solvable) | NoSE-style ILP (open solver), greedy per-query tables, random valid | cost ratio to ILP optimum, time to solution |
| E2 | workloads of 50–500 queries (ILP times out) | best of ILP at timeout, greedy | cost, time, validity rate |
| E3 | unseen entity graphs and workload shapes | the same | generalisation gap of the policy |
| E4 | environment scaling 1 to N cores | itself | steps/s per core, scaling efficiency |

## Falsification criteria (decided in advance)

The idea is **dropped or reframed** if:
1. at sizes where the ILP is solvable, the policy's median cost is more than 20 % above the optimum, **and**
2. at sizes where the ILP times out, the policy is not better than the greedy baseline at equal wall time, **or**
3. environment throughput is below 10^4 steps/s/core after optimisation (the HPC premise fails).

## Threats to validity

The reward is a model, not a measurement: a policy can exploit errors in the cost model, so final designs
should be checked on a real ScyllaDB (a small set, by hand). Synthetic workloads may not resemble production ones.
The ILP baseline is only as good as our reimplementation. Everything runs single-node on an Apple M3 at first;
multi-core scaling claims need a many-core machine.

## Milestones

0. Literature review (NoSE and successors, neural combinatorial optimisation, schema-design benchmarks).
1. `cqlz-gym` crate: instance generator, environment, action masking via `cqlc-core`, unit tests on hand-solved cases.
2. Baselines: greedy, random, ILP.
3. Policy and training in Burn; E1–E3.
4. Scaling study E4; decide whether it merits a write-up.
