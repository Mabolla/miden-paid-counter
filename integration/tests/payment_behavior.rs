use std::{path::Path, sync::Arc};

use anyhow::Context;
use integration::helpers::{build_project_in_dir, counter_storage_slot, COUNTER_STORAGE_KEY};
use miden_client::{
    account::{component::InitStorageData, AccountBuilder, AccountComponent, AccountType},
    auth::AuthSchemeId,
    crypto::RandomCoin,
    note::NoteScript,
    transaction::RawOutputNote,
    Word,
};
use miden_protocol::asset::{Asset, FungibleAsset};
use miden_standards::testing::note::NoteBuilder;
use miden_testing::{AccountState, Auth, MockChain};

#[tokio::test]
async fn payment_increments_counter_and_underpayment_is_rejected() -> anyhow::Result<()> {
    let mut builder = MockChain::builder();

    let sender = builder.add_existing_wallet(Auth::BasicAuth {
        auth_scheme: AuthSchemeId::Falcon512Poseidon2,
    })?;

    let faucet = builder.add_existing_basic_faucet(
        Auth::IncrNonce,
        "PAY",
        100_000,
        Some(100),
    )?;

    let account_package = Arc::new(build_project_in_dir(
        Path::new("../contracts/paid-counter-account"),
        true,
    )?);
    let note_package = Arc::new(build_project_in_dir(
        Path::new("../contracts/paid-increment-note"),
        true,
    )?);

    let storage_slot = counter_storage_slot()?;
    let mut init_storage_data = InitStorageData::default();
    init_storage_data.insert_map_entry(storage_slot.clone(), COUNTER_STORAGE_KEY, 0_u64)?;

    let component = AccountComponent::from_package(&account_package, &init_storage_data)
        .context("failed to build paid counter component")?;
    let counter_account = builder.add_account_from_builder(
        Auth::BasicAuth {
            auth_scheme: AuthSchemeId::Falcon512Poseidon2,
        },
        AccountBuilder::new([7_u8; 32])
            .account_type(AccountType::Public)
            .with_component(component),
        AccountState::Exists,
    )?;

    let note_script = NoteScript::from_package(note_package.as_ref())
        .context("failed to build paid increment note script")?;
    let mut note_rng = RandomCoin::new(Word::from(note_script.root()));

    let exact_payment = Asset::from(FungibleAsset::new(faucet.id(), 1)?);
    let paid_note = NoteBuilder::new(sender.id(), &mut note_rng)
        .package((*note_package).clone())
        .add_assets([exact_payment])
        .build()
        .context("failed to build paid increment note")?;

    // After the first increment the required payment becomes 2, so another payment of 1
    // must fail and must not mutate the counter.
    let underpayment = Asset::from(FungibleAsset::new(faucet.id(), 1)?);
    let underpaid_note = NoteBuilder::new(sender.id(), &mut note_rng)
        .package((*note_package).clone())
        .add_assets([underpayment])
        .build()
        .context("failed to build underpaid increment note")?;

    builder.add_output_note(RawOutputNote::Full(paid_note.clone()));
    builder.add_output_note(RawOutputNote::Full(underpaid_note.clone()));

    let mut mock_chain = builder.build()?;

    let paid_tx = mock_chain
        .build_tx_context(counter_account.clone(), &[paid_note.id()], &[])?
        .build()?
        .execute()
        .await?;

    mock_chain.add_pending_executed_transaction(&paid_tx)?;
    mock_chain.prove_next_block()?;

    let count_after_payment = mock_chain
        .committed_account(counter_account.id())?
        .storage()
        .get_map_item(&storage_slot, COUNTER_STORAGE_KEY)
        .context("counter value missing after paid increment")?;
    assert_eq!(count_after_payment[0].as_canonical_u64(), 1);

    let underpaid_tx = mock_chain
        .build_tx_context(counter_account.id(), &[underpaid_note.id()], &[])?
        .build()?
        .execute()
        .await;
    assert!(underpaid_tx.is_err(), "underpayment unexpectedly executed");

    let count_after_rejection = mock_chain
        .committed_account(counter_account.id())?
        .storage()
        .get_map_item(&storage_slot, COUNTER_STORAGE_KEY)
        .context("counter value missing after rejected underpayment")?;
    assert_eq!(count_after_rejection[0].as_canonical_u64(), 1);

    Ok(())
}
