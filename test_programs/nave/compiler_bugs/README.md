# Reproducing Noir Security Bugs

## Overview

This directory contains reproductions of security bugs documented in the [Noir Security Bug List](https://github.com/noir-lang/noir/blob/master/security/entomotaxy/Security%20Bug%20List.md).

## Motivation

We demonstrate that the Noir verifier can identify significant bugs, even though it's not yet fully optimized for performance.

## Approach

1. **Identified** the security bugs in the official Noir repository
2. **Isolated** the root cause by reverting specific changes
3. **Mapped** each reversion to its corresponding security bug
4. **Tagged** all relevant changes in the feature branch `nave_test_bug_repro`

## Setup

To reproduce these bugs, update the git tags in the Cargo configuration files for `acir_checker` and `nave_cli`:

Change all occurrences of:
```toml
{ git = "https://github.com/blockhouse-sec/noir.git", tag = "nave-v1.0.0-beta.19" }
```

To:
```toml
{ git = "https://github.com/blockhouse-sec/noir.git", tag = "nave_test_bug_repro" }
```

## Running the Bug Reproductions

Navigate to any bug directory and run the formal verification tool. For example:

```bash
cd noir/test_programs/nave/bug_repro/bug_1/field_trunc
cargo run -- formal-verify
```

Use the `--verbose` option to increase output verbosity.

## Security Bugs

| Bug | Status | Notes |
|-----|--------|-------|
| bug_1 | Working | Improper truncation of field elements |
| bug_2 | Working | Optimization error in wrapped multiplication |
| bug_3 | Working | Incorrect handling of predicate: ACIR vs Brillig |
| bug_4 | Timeout | Inconsistency in bitwise shift operation |

See the corresponding `main.nr` file in each bug directory for additional details.

