# Miden Paid Counter

An experimental Miden-native counter contract inspired by the beginner rollup tutorial need tracked in [`0xMiden/examples#209`](https://github.com/0xMiden/examples/issues/209).

The target is deliberately small and verifiable: keep a counter in account state and require a payment condition before a state update is accepted. The project is being built as an independent builder exercise first; it is **not** presented as an upstream Miden tutorial or completed testnet deployment yet.

## Why this project

Miden's tutorial issue asks for beginner-friendly examples that run against testnet rather than `MockChain`, including a simple counter contract where changing the number requires payment. This repository turns that concrete ecosystem need into a reproducible implementation.

## Definition of done

The first milestone is complete only when all of the following are true:

- [ ] counter state is represented by a Miden account component
- [ ] a transaction can request a counter update
- [ ] the update enforces the intended payment rule
- [ ] success and rejection paths are covered by tests
- [ ] the project can be reproduced from documented commands
- [ ] testnet execution evidence is recorded

Until those boxes are backed by evidence, they stay unchecked.

## Planned layout

```text
miden-paid-counter/
├── README.md
├── Cargo.toml
├── src/
│   └── main.rs
├── masm/
│   └── paid_counter.masm
├── tests/
│   └── paid_counter.rs
└── evidence/
    └── README.md
```

The exact layout may change as the current Miden SDK APIs are validated. We will not freeze an implementation around guessed or outdated APIs.

## Build log

### Milestone 0 — need validated

- Source need: [`0xMiden/examples#209`](https://github.com/0xMiden/examples/issues/209)
- Duplicate/ownership check: the issue currently lists the counter as an open tutorial idea; the visible discussion proposes a separate "Guess my number" example.
- Decision: build independently and validate locally/testnet before considering any upstream contribution.

### Milestone 1 — implementation

In progress. No execution or testnet claim has been made yet.

## Relationship to upstream

This is a builder project under the Mabolla account. If it eventually becomes useful to upstream Miden, contribution will be evaluated separately under the rule:

> prove the contribution is needed → check duplicate work → implement → test → PR → follow CI/review

A working builder project does not automatically justify an upstream PR.
