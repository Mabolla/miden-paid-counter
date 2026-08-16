# Miden Paid Counter

An experimental Miden-native counter contract inspired by the beginner rollup tutorial need tracked in [`0xMiden/examples#209`](https://github.com/0xMiden/examples/issues/209).

The target is deliberately small and verifiable: keep a counter in account state and require a real note asset payment before a state update is accepted. The project is being built as an independent builder exercise first; it is **not** presented as an upstream Miden tutorial or completed testnet deployment yet.

## Why this project

Miden's tutorial issue asks for beginner-friendly examples that run against testnet rather than `MockChain`, including a simple counter contract where changing the number requires payment. During implementation research, the current official [`0xMiden/project-template`](https://github.com/0xMiden/project-template) was found to already contain a plain counter account plus increment note. That changes the useful builder target: this project is **not another basic counter clone**. It extends the current pattern with an asset-backed payment gate.

## Payment rule

For milestone 1, incrementing from `n` to `n + 1` requires a single fungible asset payment whose amount is at least `n + 1` base units. The note asset is moved into the counter account vault before the counter state is updated.

This rule is intentionally simple so both success and rejection paths are deterministic and testable. It is our builder design choice, not a claim about an official Miden standard.

## Definition of done

The first milestone is complete only when all of the following are true:

- [x] counter state is represented by a Miden account component
- [x] a paid increment note path is implemented
- [x] the implementation enforces a concrete payment rule in code
- [ ] account and note packages build with the current Miden toolchain
- [ ] successful payment increments the counter and retains the asset in the account vault
- [ ] underpayment is rejected without incrementing state
- [ ] multiple/no-asset malformed payment notes are rejected
- [ ] tests are reproducible from documented commands
- [ ] testnet execution evidence is recorded

Unchecked boxes remain unclaimed until backed by execution evidence.

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
│   │   ├── Cargo.toml
│   │   ├── miden-project.toml
│   │   └── src/lib.rs
│   └── paid-increment-note/
│       ├── Cargo.toml
│       ├── miden-project.toml
│       └── src/lib.rs
├── integration/              # next: build + behavior tests
├── evidence/
│   └── README.md
└── README.md
```

## Build log

### Milestone 0 — need validated

- Source need: [`0xMiden/examples#209`](https://github.com/0xMiden/examples/issues/209)
- Duplicate/ownership check: the visible issue discussion does not claim the paid-counter idea.
- Important duplicate discovery: the current official `project-template` already ships a basic `counter-account` and `increment-note`.
- Decision: do not clone that example; build the missing payment-gated extension instead.

### Milestone 1 — implementation

Initial account and note packages are now in the repository. The implementation uses current documented Miden patterns including `active_note::get_assets()` for note assets and `native_account::add_asset()` for retaining payment in a custom account.

**Status: implementation written; build/test validation still pending.** No local, CI, or testnet success claim is made yet.

## Relationship to upstream

This is a builder project under the Mabolla account. If it eventually becomes useful to upstream Miden, contribution will be evaluated separately under the rule:

> prove the contribution is needed → check duplicate work → implement → test → PR → follow CI/review

A working builder project does not automatically justify an upstream PR.
