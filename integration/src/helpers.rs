use std::{path::Path, sync::Arc};

use anyhow::{bail, Context, Result};
use cargo_miden::run;
use miden_client::{
    account::{
        component::{BasicWallet, InitStorageData, NoAuth},
        Account, AccountBuilder, AccountComponent, AccountType, StorageSlotName,
    },
    auth::{AuthSchemeId, AuthSecretKey, AuthSingleSig},
    builder::ClientBuilder,
    keystore::{FilesystemKeyStore, Keystore},
    rpc::{Endpoint, GrpcClient},
    Client, Deserializable, Felt, Word,
};
use miden_client_sqlite_store::ClientBuilderSqliteExt;
use miden_mast_package::Package;
use rand::RngCore;

pub const COUNTER_STORAGE_KEY: Word =
    Word::new([Felt::ZERO, Felt::ZERO, Felt::ZERO, Felt::ONE]);

pub fn counter_storage_slot() -> Result<StorageSlotName> {
    StorageSlotName::new("paid_counter_account::paid_counter::count_map")
        .context("invalid paid counter storage slot name")
}

pub struct ClientSetup {
    pub client: Client<FilesystemKeyStore>,
    pub keystore: Arc<FilesystemKeyStore>,
}

pub async fn setup_client() -> Result<ClientSetup> {
    let endpoint = Endpoint::testnet();
    let rpc_client = Arc::new(GrpcClient::new(&endpoint, 10_000));
    let keystore = Arc::new(
        FilesystemKeyStore::new(std::path::PathBuf::from("../keystore"))
            .context("failed to initialize keystore")?,
    );

    let client = ClientBuilder::new()
        .rpc(rpc_client)
        .sqlite_store(std::path::PathBuf::from("../store.sqlite3"))
        .authenticator(keystore.clone())
        .in_debug_mode(true.into())
        .build()
        .await
        .context("failed to build Miden testnet client")?;

    Ok(ClientSetup { client, keystore })
}

pub fn build_project_in_dir(dir: &Path, release: bool) -> Result<Package> {
    let profile = if release { "--release" } else { "--debug" };
    let manifest_path = dir.join("Cargo.toml");
    let manifest_arg = manifest_path.to_string_lossy();

    let args = vec![
        "cargo",
        "miden",
        "build",
        profile,
        "--manifest-path",
        &manifest_arg,
    ];

    let output = run(args.into_iter().map(String::from))
        .context("failed to compile Miden project")?
        .context("cargo miden build returned no output")?;

    let artifact_path = match output {
        cargo_miden::CommandOutput::BuildCommandOutput { output } => output
            .into_iter()
            .next()
            .context("cargo miden build produced no artifact")?,
        other => bail!("expected build output, got {:?}", other),
    };

    let package_bytes = std::fs::read(&artifact_path)
        .with_context(|| format!("failed to read {}", artifact_path.display()))?;

    Package::read_from_bytes(&package_bytes).context("failed to deserialize Miden package")
}

pub struct AccountCreationConfig {
    pub account_type: AccountType,
    pub init_storage_data: InitStorageData,
}

impl Default for AccountCreationConfig {
    fn default() -> Self {
        Self {
            account_type: AccountType::Public,
            init_storage_data: InitStorageData::default(),
        }
    }
}

pub async fn create_account_from_package(
    client: &mut Client<FilesystemKeyStore>,
    package: Arc<Package>,
    config: AccountCreationConfig,
) -> Result<Account> {
    let component = AccountComponent::from_package(package.as_ref(), &config.init_storage_data)
        .context("failed to create paid counter component")?;

    let mut init_seed = [0_u8; 32];
    client.rng().fill_bytes(&mut init_seed);

    let account = AccountBuilder::new(init_seed)
        .account_type(config.account_type)
        .with_component(component)
        .with_auth_component(NoAuth)
        .build()
        .context("failed to build paid counter account")?;

    client
        .add_account(&account, false)
        .await
        .context("failed to add paid counter account to client")?;

    Ok(account)
}

pub async fn create_basic_wallet_account(
    client: &mut Client<FilesystemKeyStore>,
    keystore: Arc<FilesystemKeyStore>,
    config: AccountCreationConfig,
) -> Result<Account> {
    let mut init_seed = [0_u8; 32];
    client.rng().fill_bytes(&mut init_seed);

    let key_pair = AuthSecretKey::new_falcon512_poseidon2_with_rng(client.rng());
    let account = AccountBuilder::new(init_seed)
        .account_type(config.account_type)
        .with_auth_component(AuthSingleSig::new(
            key_pair.public_key().to_commitment(),
            AuthSchemeId::Falcon512Poseidon2,
        ))
        .with_component(BasicWallet)
        .build()
        .context("failed to build sender wallet")?;

    client
        .add_account(&account, false)
        .await
        .context("failed to add sender wallet to client")?;

    keystore
        .add_key(&key_pair, account.id())
        .await
        .context("failed to add sender key to keystore")?;

    Ok(account)
}
