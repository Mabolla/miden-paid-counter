# Miden Paid Counter

A Miden-native paid counter contract inspired by the beginner rollup tutorial need tracked in [`0xMiden/examples#209`](https://github.com/0xMiden/examples/issues/209).

The project keeps a counter in account state and requires a real note asset payment before a state update is accepted. It is an independent builder project and now includes CI-backed execution against Miden Testnet.

## Why this project

Miden's tutorial issue asks for beginner-friendly examples that run against testnet rather than `MockChain`, including a simple counter contract where changing the number requires payment. During implementation research, the current official [`0xMiden/project-template`](https://github.com/0xMiden/project-template) was found to already contain a plain counter account plus increment note. That changes the useful builder target: this project is **not another basic counter clone**. It extends the current pattern with an asset-backed payment gate.

## Payment rule

For milestone 1, incrementing from `n` to `n + 1` requires a single fungible asset payment whose amount is at least `n + 1` base units. The note asset is moved into the counter account vault before the counter state is updated.

This rule is intentionally simple so both success and rejection paths are deterministic and testable. It is our builder design choice, not a claim about an official Miden standard.

## Verified milestones

The project is backed by both deterministic MockChain behavior coverage and a live Miden Testnet E2E execution.

- [x] counter state is represented by a Miden account component
- [x] a paid increment note path is implemented
- [x] the implementation enforces a concrete payment rule in code
- [x] account and note packages build in CI
- [x] a payment of 1 increments the committed counter from 0 to 1
- [x] a second payment of 1 is rejected once the required payment rises to 2
- [x] the rejected transaction leaves the committed counter at 1
- [x] tests are reproducible through repository CI
- [x] Testnet client, account, sender wallet, and PAY faucet are created by the E2E runner
- [x] PAY is minted and consumed into the sender vault on Miden Testnet
- [x] a paid increment note is published and consumed on Miden Testnet
- [x] persisted Testnet counter state is verified as 1 after the paid increment
- [ ] explicit multiple/no-asset malformed-note coverage

### Testnet proof

Verified paid increment transaction:

`0xbf1746e47d306257608cffbdb8039cf5a4fa0f70737784205d731a178807347c`

[MidenScan transaction](https://testnet.midenscan.com/tx/0xbf1746e47d306257608cffbdb8039cf5a4fa0f70737784205d731a178807347c)

The successful E2E run created a paid-counter account, authenticated sender, and PAY faucet; minted PAY; consumed the funding note; published the custom paid-increment note; executed it against the counter account; and asserted that the persisted counter became `1`.

Full recorded evidence is kept in [`evidence/README.md`](evidence/README.md).

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
│   ├── src/bin/testnet_paid_counter.rs
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

### Milestone 1 — implementation and MockChain verification

The account and note packages implement an asset-backed payment gate. Integration coverage builds the packages, executes a valid paid increment through Miden's MockChain testing environment, commits the resulting block, reads the persisted counter state, and then verifies that an underpayment is rejected without changing committed state.

**Status: MockChain behavior milestone verified in CI.**

### Milestone 2 — Miden Testnet E2E

The Testnet runner uses the current Miden client stack with a SQLite store, Testnet RPC, and filesystem keystore. It creates the required accounts, obtains a fungible PAY asset through a faucet mint/consume flow, publishes the custom paid-increment note, consumes that note through the paid-counter account, syncs state, and verifies the persisted counter value.

**Status: Miden Testnet paid increment verified end-to-end. Counter state reached `1`.**

## Relationship to upstream

This is a builder project under the Mabolla account. If it eventually becomes useful to upstream Miden, contribution will be evaluated separately under the rule:

> prove the contribution is needed → check duplicate work → implement → test → PR → follow CI/review

A working builder project does not automatically justify an upstream PR.
