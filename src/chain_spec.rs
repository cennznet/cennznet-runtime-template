use cennznet_runtime_template_runtime::{
    fees, generic_asset, AccountId, CennzxSpotConfig, ConsensusConfig, Fee, FeeRate, FeesConfig,
    GenericAssetConfig, GenesisConfig, IndicesConfig, SudoConfig, TimestampConfig,
};
use primitives::{ed25519, sr25519, Pair};
use substrate_service;

use ed25519::Public as AuthorityId;

// Note this is the URL for the telemetry server
//const STAGING_TELEMETRY_URL: &str = "wss://telemetry.polkadot.io/submit/";

/// Specialized `ChainSpec`. This is a specialization of the general Substrate ChainSpec type.
pub type ChainSpec = substrate_service::ChainSpec<GenesisConfig>;

/// The chain specification option. This is expected to come in from the CLI and
/// is little more than one of a number of alternatives which can easily be converted
/// from a string (`--chain=...`) into a `ChainSpec`.
#[derive(Clone, Debug)]
pub enum Alternative {
    /// Whatever the current runtime is, with just Alice as an auth.
    Development,
    /// Whatever the current runtime is, with simple Alice/Bob auths.
    LocalTestnet,
}

fn authority_key(s: &str) -> AuthorityId {
    ed25519::Pair::from_string(&format!("//{}", s), None)
        .expect("static values are valid; qed")
        .public()
}

fn account_key(s: &str) -> AccountId {
    sr25519::Pair::from_string(&format!("//{}", s), None)
        .expect("static values are valid; qed")
        .public()
}

impl Alternative {
    /// Get an actual chain config from one of the alternatives.
    pub(crate) fn load(self) -> Result<ChainSpec, String> {
        Ok(match self {
            Alternative::Development => ChainSpec::from_genesis(
                "Development",
                "dev",
                || {
                    testnet_genesis(
                        vec![authority_key("Alice")],
                        vec![
                            account_key("Alice"),
                            account_key("Bob"),
                            account_key("Charlie"),
                            account_key("Dave"),
                            account_key("Eve"),
                            account_key("Ferdie"),
                        ],
                        account_key("Alice"),
                    )
                },
                vec![],
                None,
                None,
                None,
                None,
            ),
            Alternative::LocalTestnet => ChainSpec::from_genesis(
                "Local Testnet",
                "local_testnet",
                || {
                    testnet_genesis(
                        vec![authority_key("Alice"), authority_key("Bob")],
                        vec![
                            account_key("Alice"),
                            account_key("Bob"),
                            account_key("Charlie"),
                            account_key("Dave"),
                            account_key("Eve"),
                            account_key("Ferdie"),
                        ],
                        account_key("Alice"),
                    )
                },
                vec![],
                None,
                None,
                None,
                None,
            ),
        })
    }

    pub(crate) fn from(s: &str) -> Option<Self> {
        match s {
            "dev" => Some(Alternative::Development),
            "" | "local" => Some(Alternative::LocalTestnet),
            _ => None,
        }
    }
}

fn testnet_genesis(
    initial_authorities: Vec<AuthorityId>,
    endowed_accounts: Vec<AccountId>,
    root_key: AccountId,
) -> GenesisConfig {
    GenesisConfig {
		consensus: Some(ConsensusConfig {
			code: include_bytes!("../runtime/wasm/target/wasm32-unknown-unknown/release/cennznet_runtime_template_runtime_wasm.compact.wasm").to_vec(),
			authorities: initial_authorities.clone(),
		}),
		system: None,
		timestamp: Some(TimestampConfig {
			minimum_period: 3, // 6 second block time.
		}),
		indices: Some(IndicesConfig {
			ids: endowed_accounts.clone(),
		}),
		generic_asset: Some(GenericAssetConfig {
			assets: vec![16000, 16001],
			initial_balance: 10u128.pow(10),
			endowed_accounts: endowed_accounts.clone().into_iter().map(Into::into).collect(),
			next_asset_id: 17000,
			create_asset_stake: 0,
			staking_asset_id: 16000,
			spending_asset_id: 16001,
		}),
		fees: Some(FeesConfig {
			_genesis_phantom_data: Default::default(),
			fee_registry: vec![
				(Fee::fees(fees::Fee::Base), 1),
				(Fee::fees(fees::Fee::Bytes), 0),
				(Fee::generic_asset(generic_asset::Fee::Transfer), 1),
			],
		}),
		cennzx_spot: Some(CennzxSpotConfig {
			fee_rate: FeeRate::from_milli(3),
			core_asset_id: 16001,
		}),
		sudo: Some(SudoConfig {
			key: root_key,
		}),
	}
}
