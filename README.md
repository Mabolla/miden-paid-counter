# Miden Paid Counter

A Miden-native paid counter contract inspired by the beginner rollup tutorial need tracked in [`0xMiden/examples#209`](https://github.com/0xMiden/examples/issues/209).

The project keeps a counter in account state and requires a real note asset payment before a state update is accepted. It is an independent builder project; it is **not** presented as an upstream Miden tutorial or completed testnet deployment yet.

## Why this project

Miden's tutorial issue asks for beginner-friendly examples that run against testnet rather than `MockChain`, including a simple counter contract where changing the number requires payment. During implementation research, the current official [`0xMiden/project-template`](https://github.com/0xMiden/project-template) was found to already contain a plain counter account plus increment note. That changes the useful builder target: this project is **not another basic counter clone**. It extends the current pattern with an asset-backed payment gate.

## Payment rule

For milestone 1, incrementing from `n` to `n + 1` requires a single fungible asset payment whose amount is at least `n + 1` base units. The note asset is moved into the counter account vault before the counter state is updated.

This rule is intentionally simple so both success and rejection paths are deterministic and testable. It is our builder design choice, not a claim about an official Miden standard.

## Verified milestone

Milestone 1 is now backed by CI execution on GitHub Actions using the current repository dependency set.

- [x] counter state is represented by a Miden account component
- [x] a paid increment note path is implemented
- [x] the implementation enforces a concrete payment rule in code
- [x] account and note packages build in CI
- [x] a payment of 1 increments the committed counter from 0 to 1
- [x] a second payment of 1 is rejected once the required payment rises to 2
- [x] the rejected transaction leaves the committed counter at 1
- [x] tests are reproducible through the repository CI workflow
- [ ] explicit multiple/no-asset malformed-note coverage
- [ ] testnet execution evidence

The behavior test was introduced through PR #1 and passed both on the pull-request branch and again on `main` after merge. Unchecked boxes remain unclaimed until backed by execution evidence.

## Current architecture

```text
payment note (exactly one asset)
        |
        v
paid-increment-note
        |
        | calls pay_and_increment(asset)
        v
paid-counter-account
        |-- verify amount >= current_count + 1
        |-- native_account::add_asset(payment)
        `-- persist count + 1
```

The contract shape follows the current Miden Rust project model: account components hold persistent state while note scripts execute one-time logic when a note is consumed.

## Repository layout

```text
miden-paid-counter/
├── Cargo.toml
├── contracts/
│   ├── paid-counter-account/
│   └── paid-increment-note/
├── integration/
│   └── tests/                # package-build and payment-behavior coverage
├── evidence/
│   └── README.md
└── README.md
```

## Build log

### Milestone 0 — need validated

- Source need: [`0xMiden/examples#209`](https://github.com/0xMiden/examples/issues/209)
- Duplicate/ownership check: the visible issue discussion did not claim the paid-counter idea when this builder exercise was selected.
- Important duplicate discovery: the official `project-template` already ships a basic `counter-account` and `increment-note`.
- Decision: do not clone that example; build the missing payment-gated extension instead.

### Milestone 1 — implementation and verification

The account and note packages implement an asset-backed payment gate. Integration coverage builds the packages, executes a valid paid increment through Miden's MockChain testing environment, commits the resulting block, reads the persisted counter state, and then verifies that an underpayment is rejected without changing committed state.

**Status: MockChain behavior milestone verified in CI. Testnet deployment/execution remains a separate future milestone and is not claimed here.**

## Relationship to upstream

This is a builder project under the Mabolla account. If it eventually becomes useful to upstream Miden, contribution will be evaluated separately under the rule:

> prove the contribution is needed → check duplicate work → implement → test → PR → follow CI/review

A working builder project does not automatically justify an upstream PR.
