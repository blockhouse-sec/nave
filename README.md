# NAVe — Noir Formal Verifier

NAVe is a formal verifier for [Noir](https://www.noir-lang.org/), a domain-specific language for SNARK proving systems. Not to be confused with a ZK verifier, NAVe checks that Noir programs give rise to the expected constraints.

NAVe translates an ACIR program into a corresponding set of SMT constraints that are verified by the CVC5 SMT solver. It relies on the Noir compiler to compile a Noir program into ACIR.

**This implementation is in early development. It has not been reviewed or audited. It is not suitable for production use. Expect bugs!**

The paper describing NAVe — and giving a formal semantics to (a subset of) ACIR — is at: [https://arxiv.org/abs/2601.09372](https://arxiv.org/abs/2601.09372)

## How It Works

A developer annotates their Noir program with *verification asserts*. Unlike Noir's built-in `assert`, these are not runtime constraints — they are verification conditions checked by the SMT solver.

```
Noir source → Noir compiler → ACIR circuit → NAVe translator → SMT2 → CVC5 → result
```

The main components are in [tooling/acir_checker](tooling/acir_checker).

## Quick Start

**Requirements**: CVC5 with finite field support must be on `PATH`. Download from [cvc5/cvc5 releases](https://github.com/cvc5/cvc5/releases/) (for macOS ARM: `cvc5-macOS-arm64-static-gpl.zip`).

```bash
# Build
cargo build --release

# Run the verifier on a Noir project
cargo run -- formal-verify
cargo run -- formal-verify --backend=ff-split
cargo run -- formal-verify --backend=int
cargo run -- formal-verify --relaxed
cargo run -- formal-verify --check-execution --verbose=normal
```

## Tutorial

A simple tutorial using a Rock-Paper-Scissors example is in [test_programs/nave/tutorial/rps](test_programs/nave/tutorial/rps).

NAVe test programs are in [test_programs/nave](test_programs/nave).

## Backends

| Backend | Description |
|---------|-------------|
| `ff-gb` | Finite field with Gröbner bases — default, most complete |
| `ff-split` | Finite field split constraints — faster approximation |
| `int` | Integer encoding — for testing/approximation |

## Verification Outcomes

- **Verified** — SMT query is UNSAT (property holds for all inputs)
- **Falsified(Model)** — SAT with counterexample
- **Unknown** — Solver could not determine

## Minimum Rust Version

This workspace's minimum supported rustc version is 1.85.0.

## License

NAVe is free and open source. It is distributed under a dual license (MIT/APACHE).

Unless you explicitly state otherwise, any contribution intentionally submitted for inclusion in this repository by you, as defined in the Apache-2.0 license, shall be dual licensed as above, without any additional terms or conditions.
