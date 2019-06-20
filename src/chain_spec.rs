use cennznet_runtime_template_runtime::{
    fees, generic_asset, AccountId, CennzxSpotConfig, ConsensusConfig, ContractConfig, Fee,
    FeeRate, FeesConfig, GenericAssetConfig, GenesisConfig, GrandpaConfig, IndicesConfig, Schedule,
    SessionConfig, StakerStatus, StakingConfig, SudoConfig, TimestampConfig,
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

/// Helper function to generate AccountId from seed
pub fn get_account_id_from_seed(seed: &str) -> AccountId {
    sr25519::Pair::from_string(&format!("//{}", seed), None)
        .expect("static values are valid; qed")
        .public()
}

/// Helper function to generate AuthorityId from seed
pub fn get_session_key_from_seed(seed: &str) -> AuthorityId {
    ed25519::Pair::from_string(&format!("//{}", seed), None)
        .expect("static values are valid; qed")
        .public()
}

/// Helper function to generate stash, controller and session key from seed
pub fn get_authority_keys_from_seed(seed: &str) -> (AccountId, AccountId, AuthorityId) {
    (
        get_account_id_from_seed(seed),
        get_account_id_from_seed(seed),
        get_session_key_from_seed(seed),
    )
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
                        vec![get_authority_keys_from_seed("Alice")],
                        vec![
                            get_account_id_from_seed("Alice"),
                            get_account_id_from_seed("Alice//stash"),
                            get_account_id_from_seed("Bob"),
                            get_account_id_from_seed("Charlie"),
                            get_account_id_from_seed("Dave"),
                            get_account_id_from_seed("Eve"),
                            get_account_id_from_seed("Ferdie"),
                        ],
                        get_account_id_from_seed("Alice"),
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
                        vec![
                            get_authority_keys_from_seed("Alice"),
                            get_authority_keys_from_seed("Bob"),
                        ],
                        vec![
                            get_account_id_from_seed("Alice"),
                            get_account_id_from_seed("Alice//stash"),
                            get_account_id_from_seed("Bob"),
                            get_account_id_from_seed("Bob//stash"),
                            get_account_id_from_seed("Charlie"),
                            get_account_id_from_seed("Dave"),
                            get_account_id_from_seed("Eve"),
                            get_account_id_from_seed("Ferdie"),
                        ],
                        get_account_id_from_seed("Alice"),
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
    initial_authorities: Vec<(AccountId, AccountId, AuthorityId)>,
    endowed_accounts: Vec<AccountId>,
    root_key: AccountId,
) -> GenesisConfig {
    GenesisConfig {
		consensus: Some(ConsensusConfig {
			code: include_bytes!("../runtime/wasm/target/wasm32-unknown-unknown/release/cennznet_runtime_template.compact.wasm").to_vec(),
			authorities: initial_authorities.iter().map(|x| x.2.clone()).collect(),
		}),
		system: None,
		timestamp: Some(TimestampConfig {
			minimum_period: 3, // 6 second block time.
		}),
		indices: Some(IndicesConfig {
			ids: endowed_accounts.clone(),
		}),
        session: Some(SessionConfig {
			validators: initial_authorities.iter().map(|x| x.1.clone()).collect(),
			session_length: 10,
			keys: initial_authorities
				.iter()
				.map(|x| (x.1.clone(), x.2.clone()))
				.collect::<Vec<_>>(),
		}),
        staking: Some(StakingConfig {
			current_era: 0,
			minimum_validator_count: 1,
			validator_count: 4,
			sessions_per_era: 5,
			bonding_duration: 12,
			offline_slash: Default::default(),
			session_reward: Default::default(),
			current_session_reward: 0,
			offline_slash_grace: 0,
			stakers: initial_authorities
				.iter()
				.map(|x| (x.0.clone(), x.1.clone(), 1_000_000_000, StakerStatus::Validator))
				.collect(),
			invulnerables: initial_authorities.iter().map(|x| x.1.clone()).collect(),
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
    	grandpa: Some(GrandpaConfig {
			authorities: initial_authorities.iter().map(|x| (x.2.clone(), 1)).collect(),
		}),
        contract: Some(ContractConfig {
			signed_claim_handicap: 2,
			rent_byte_price: 1,
			rent_deposit_offset: 1000,
			storage_size_offset: 8,
			surcharge_reward: 150,
			tombstone_deposit: 16,
			contract_fee: 1,
			call_base_fee: 1,
			create_base_fee: 1,
			creation_fee: 0,
			transaction_base_fee: 1,
			transaction_byte_fee: 0,
			transfer_fee: 1,
			gas_price: 1,
			max_depth: 1024,
			block_gas_limit: 10_000_000_000,
			current_schedule: Schedule {
				enable_println: true,
				..Default::default()
			},
		}),
	}
}
