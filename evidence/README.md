# Evidence

This directory records execution evidence only after it has been produced and verified.

## Miden Testnet E2E — verified

GitHub Actions executed the paid-counter flow against Miden Testnet successfully.

Observed execution:

- paid counter account: `0xb6bf63ee3119603160fd270dff0b83`
- sender account: `0xeca5cd71994618d10570c0e8c84e73`
- PAY faucet: `0x274f401f6769fb5128676afae3c109`
- PAY mint transaction: successful
- funding-note consume transaction: successful
- paid-increment note publication: successful
- paid increment transaction: successful
- persisted counter after execution: `1`

### Paid increment transaction

`0xbf1746e47d306257608cffbdb8039cf5a4fa0f70737784205d731a178807347c`

MidenScan:
https://testnet.midenscan.com/tx/0xbf1746e47d306257608cffbdb8039cf5a4fa0f70737784205d731a178807347c

The E2E runner asserts that the persisted counter equals `1`; the workflow exits successfully only when that assertion and the preceding Testnet transaction flow complete successfully.

## MockChain milestone

The repository also has CI-backed integration coverage for the deterministic payment rule:

- payment of 1 increments the committed counter from 0 to 1;
- a subsequent payment of 1 is rejected when the required amount becomes 2;
- the rejected transaction leaves committed state unchanged at 1.

These MockChain checks remain useful deterministic behavioral coverage alongside the live Testnet evidence above.
