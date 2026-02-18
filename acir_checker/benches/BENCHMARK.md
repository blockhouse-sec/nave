# Benchmarking NAVe

This folder has scripts that can be used to benchmark NAVe.

We rely on [hyperfine](https://github.com/sharkdp/hyperfine) to
carry out our benchmarking; [installation instructions](https://github.com/sharkdp/hyperfine?tab=readme-ov-file#installation).

The Noir programs giving rise to our set of benchmarks is in `acir_checker/test_programs/benches`; each program has its own folder in this directory. The main goal is to analyse changes in the verifier's backends.

* `benchmark.sh`: Uses hyperfine to
benchmark runs of NAVe for Noir programs in our set of benchmarks. To install hyperfine

* `comparison.py`: Produces a markdown table comparing two benchmarking runs.





