use std::{path::Path, sync::Arc};

use anyhow::{Context, Result};
use integration::helpers::{
    build_project_in_dir, counter_storage_slot, create_account_from_package,
    create_basic_wallet_account, create_fungible_faucet, setup_client, AccountCreationConfig,
    ClientSetup, COUNTER_STORAGE_KEY,
};
use miden_client::{
    account::component::InitStorageData,
    asset::FungibleAsset,
    note::{NoteScript, NoteType},
    transaction::TransactionRequestBuilder,
};
use miden_standards::testing::note::NoteBuilder;

#[tokio::main]
async fn main() -> Result<()> {
    let ClientSetup {
        mut client,
        keystore,
    } = setup_client().await?;

    let sync = client.sync_state().await.context("initial testnet sync failed")?;
    println!("Latest testnet block: {}", sync.block_num);

    let account_package = Arc::new(
        build_project_in_dir(Path::new("../contracts/paid-counter-account"), true)
            .context("failed to build paid counter package")?,
    );
    let note_package = Arc::new(
        build_project_in_dir(Path::new("../contracts/paid-increment-note"), true)
            .context("failed to build paid increment package")?,
    );

    let storage_slot = counter_storage_slot()?;
    let mut init_storage_data = InitStorageData::default();
    init_storage_data.insert_map_entry(
        storage_slot.clone(),
        COUNTER_STORAGE_KEY,
        0_u64,
    )?;

    let counter_account = create_account_from_package(
        &mut client,
        account_package,
        AccountCreationConfig {
            init_storage_data,
            ..Default::default()
        },
    )
    .await?;
    println!("Paid counter account: {}", counter_account.id());

    let sender = create_basic_wallet_account(
        &mut client,
        keystore.clone(),
        AccountCreationConfig::default(),
    )
    .await?;
    println!("Sender account: {}", sender.id());

    let faucet = create_fungible_faucet(&mut client, keystore).await?;
    println!("PAY faucet: {}", faucet.id());

    // Mint PAY to the sender through the faucet, using the same v0.15 flow as the official
    // Miden rust-client mint/consume tutorial.
    let funded_asset = FungibleAsset::new(faucet.id(), 10)?;
    let mint_request = TransactionRequestBuilder::new().build_mint_fungible_asset(
        funded_asset,
        sender.id(),
        NoteType::Public,
        client.rng(),
    )?;
    let mint_tx = client
        .submit_new_transaction(faucet.id(), mint_request)
        .await
        .context("failed to submit PAY mint transaction")?;
    println!("Mint tx: https://testnet.midenscan.com/tx/{mint_tx:?}");

    // Wait until the minted note is visible, then consume it so the sender vault actually owns PAY.
    let minted_note = loop {
        client.sync_state().await?;
        let consumable = client.get_consumable_notes(Some(sender.id())).await?;
        if let Some((note, _)) = consumable.first() {
            break note.clone().try_into()?;
        }
        tokio::time::sleep(std::time::Duration::from_secs(3)).await;
    };

    let consume_funding = TransactionRequestBuilder::new().build_consume_notes(vec![minted_note])?;
    let funding_tx = client
        .submit_new_transaction(sender.id(), consume_funding)
        .await
        .context("failed to consume PAY funding note")?;
    println!("Funding consume tx: https://testnet.midenscan.com/tx/{funding_tx:?}");

    client.sync_state().await?;

    // Create a custom paid-increment note carrying exactly 1 PAY. The note script calls the
    // paid-counter account component, which accepts this first payment because the counter is 0.
    let note_script = NoteScript::from_package(note_package.as_ref())
        .context("failed to load paid increment note script")?;
    let payment = FungibleAsset::new(faucet.id(), 1)?;
    let paid_note = NoteBuilder::new(sender.id(), client.rng())
        .package((*note_package).clone())
        .add_assets([payment.into()])
        .tag(0)
        .build()
        .context("failed to build paid increment note")?;
    println!("Paid note script root: {:?}", note_script.root());
    println!("Paid note id: {}", paid_note.id());

    let publish_request = TransactionRequestBuilder::new()
        .own_output_notes(vec![paid_note.clone()])
        .build()?;
    let publish_tx = client
        .submit_new_transaction(sender.id(), publish_request)
        .await
        .context("failed to publish paid increment note")?;
    println!("Paid-note publish tx: https://testnet.midenscan.com/tx/{publish_tx:?}");

    // Sync until the publish transaction is visible before consuming the custom note.
    client.sync_state().await?;
    tokio::time::sleep(std::time::Duration::from_secs(3)).await;
    client.sync_state().await?;

    let increment_request = TransactionRequestBuilder::new()
        .input_notes([(paid_note.clone(), None)])
        .build()?;
    let increment_tx = client
        .submit_new_transaction(counter_account.id(), increment_request)
        .await
        .context("failed to execute paid increment on testnet")?;
    println!("Paid increment tx: https://testnet.midenscan.com/tx/{increment_tx:?}");

    client.sync_state().await?;
    let updated = client
        .get_account(counter_account.id())
        .await?
        .context("paid counter account missing after execution")?;
    let count = updated
        .storage()
        .get_map_item(&storage_slot, COUNTER_STORAGE_KEY)
        .context("counter storage missing after testnet execution")?;

    println!("Counter after paid increment: {}", count[0].as_canonical_u64());
    anyhow::ensure!(
        count[0].as_canonical_u64() == 1,
        "expected counter to be 1 after first paid increment"
    );

    Ok(())
}
