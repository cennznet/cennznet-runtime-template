//! The CENNZnet Runtime Template runtime. This can be compiled with `#[no_std]`, ready for Wasm.

#![cfg_attr(not(feature = "std"), no_std)]

// `construct_runtime!` does a lot of recursion and requires us to increase the limit to 256.
#![recursion_limit="512"]

pub use cennznet_primitives::{
	AccountId, AccountIndex, AuthorityId, AuthoritySignature, Balance, BlockNumber, CennznetExtrinsic, Hash, Index,
	Signature,
};

#[cfg(feature = "std")]
use serde::{Serialize, Deserialize};
use parity_codec::{Encode, Decode};
use rstd::prelude::*;
#[cfg(feature = "std")]
use primitives::bytes;
use primitives::OpaqueMetadata;
use runtime_primitives::{
	ApplyResult, transaction_validity::TransactionValidity, generic, create_runtime_str,
	traits::{self, NumberFor, BlakeTwo256, Block as BlockT, StaticLookup, Checkable, DigestFor, AuthorityIdFor, Convert},
};
use grandpa::fg_primitives::{self, ScheduledChange};
use client::{
	block_builder::api::{CheckInherentsResult, InherentData, self as block_builder_api},
	runtime_api as client_api, impl_runtime_apis,
};
use version::RuntimeVersion;
#[cfg(feature = "std")]
use version::NativeVersion;

use generic_asset::{SpendingAssetCurrency, StakingAssetCurrency};
use support::traits::Currency;
use support::construct_runtime;
pub use contract::Schedule;
pub use staking::StakerStatus;

pub use cennzx_spot::{ExchangeAddressGenerator, FeeRate};

pub use fees;
pub use generic_asset;

mod fee;

/// Used for the module template in `./template.rs`
mod template;

/// Opaque types. These are used by the CLI to instantiate machinery that don't need to know
/// the specifics of the runtime. They can then be made to be agnostic over specific formats
/// of data like extrinsics, allowing for them to continue syncing the network through upgrades
/// to even the core datastructures.
pub mod opaque {
	use super::*;

	/// Opaque, encoded, unchecked extrinsic.
	#[derive(PartialEq, Eq, Clone, Default, Encode, Decode)]
	#[cfg_attr(feature = "std", derive(Serialize, Deserialize))]
	pub struct UncheckedExtrinsic(#[cfg_attr(feature = "std", serde(with="bytes"))] pub Vec<u8>);
	#[cfg(feature = "std")]
	impl std::fmt::Debug for UncheckedExtrinsic {
		fn fmt(&self, fmt: &mut std::fmt::Formatter) -> std::fmt::Result {
			write!(fmt, "{}", primitives::hexdisplay::HexDisplay::from(&self.0))
		}
	}
	impl traits::Extrinsic for UncheckedExtrinsic {
		fn is_signed(&self) -> Option<bool> {
			None
		}
	}
	/// Opaque block header type.
	pub type Header = generic::Header<BlockNumber, BlakeTwo256, generic::DigestItem<Hash, AuthorityId, AuthoritySignature>>;
	/// Opaque block type.
	pub type Block = generic::Block<Header, UncheckedExtrinsic>;
	/// Opaque block identifier type.
	pub type BlockId = generic::BlockId<Block>;
	/// Opaque session key type.
	pub type SessionKey = AuthorityId;
}

/// This runtime version.
pub const VERSION: RuntimeVersion = RuntimeVersion {
	spec_name: create_runtime_str!("cennznet-runtime-template"),
	impl_name: create_runtime_str!("cennznet-runtime-template"),
	authoring_version: 3,
	spec_version: 3,
	impl_version: 0,
	apis: RUNTIME_API_VERSIONS,
};

/// The version infromation used to identify this runtime when compiled natively.
#[cfg(feature = "std")]
pub fn native_version() -> NativeVersion {
	NativeVersion {
		runtime_version: VERSION,
		can_author_with: Default::default(),
	}
}

pub struct CurrencyToVoteHandler;

impl CurrencyToVoteHandler {
	fn factor() -> u128 {
		(<StakingAssetCurrency<Runtime>>::total_issuance() / u64::max_value() as u128).max(1)
	}
}

impl Convert<u128, u64> for CurrencyToVoteHandler {
	fn convert(x: u128) -> u64 {
		(x / Self::factor()) as u64
	}
}

impl Convert<u128, u128> for CurrencyToVoteHandler {
	fn convert(x: u128) -> u128 {
		x * Self::factor()
	}
}

impl system::Trait for Runtime {
	/// The identifier used to distinguish between accounts.
	type AccountId = AccountId;
	/// The lookup mechanism to get account ID from whatever is passed in dispatchers.
	type Lookup = Indices;
	/// The index type for storing how many extrinsics an account has signed.
	type Index = Index;
	/// The index type for blocks.
	type BlockNumber = BlockNumber;
	/// The type for hashing blocks and tries.
	type Hash = Hash;
	/// The hashing algorithm used.
	type Hashing = BlakeTwo256;
	/// The header digest type.
	type Digest = generic::Digest<Log>;
	/// The header type.
	type Header = generic::Header<BlockNumber, BlakeTwo256, Log>;
	/// The ubiquitous event type.
	type Event = Event;
	/// The ubiquitous log type.
	type Log = Log;
	/// The ubiquitous origin type.
	type Origin = Origin;
	type Signature = Signature;
}

impl aura::Trait for Runtime {
	type HandleReport = ();
}

impl consensus::Trait for Runtime {
	/// The identifier we use to refer to authorities.
	type SessionKey = AuthorityId;
	// The aura module handles offline-reports internally
	// rather than using an explicit report system.
	type InherentOfflineReport = ();
	/// The ubiquitous log type.
	type Log = Log;
}

impl indices::Trait for Runtime {
	/// The type for recording indexing into the account enumeration. If this ever overflows, there
	/// will be problems!
	type AccountIndex = AccountIndex;
	/// Use the standard means of resolving an index hint from an id.
	type ResolveHint = indices::SimpleResolveHint<Self::AccountId, Self::AccountIndex>;
	/// Determine whether an account is dead.
	type IsDeadAccount = ();
	/// The uniquitous event type.
	type Event = Event;
}

impl timestamp::Trait for Runtime {
	/// A timestamp: seconds since the unix epoch.
	type Moment = u64;
	type OnTimestampSet = Aura;
}

impl session::Trait for Runtime {
	type ConvertAccountIdToSessionKey = ();
	type OnSessionChange = (Staking, grandpa::SyncedAuthorities<Runtime>);
	type Event = Event;
}

impl staking::Trait for Runtime {
	type Currency = StakingAssetCurrency<Self>;
	type RewardCurrency = SpendingAssetCurrency<Self>;
	type CurrencyToReward = Balance;
	type BalanceToU128 = Balance;
	type U128ToBalance = Balance;
	type CurrencyToVote = CurrencyToVoteHandler;
	type OnRewardMinted = ();
	type Event = Event;
	type Slash = ();
	type Reward = ();
}

impl grandpa::Trait for Runtime {
	type Log = Log;
	type SessionKey = AuthorityId;
	type Event = Event;
}

impl contract::Trait for Runtime {
	type Currency = SpendingAssetCurrency<Self>;
	type Call = Call;
	type Event = Event;
	type Gas = u64;
	type DetermineContractAddress = contract::SimpleAddressDeterminator<Runtime>;
	type ComputeDispatchFee = contract::DefaultDispatchFeeComputor<Runtime>;
	type TrieIdGenerator = contract::TrieIdFromParentCounter<Runtime>;
	type GasPayment = ();
}

impl sudo::Trait for Runtime {
	/// The uniquitous event type.
	type Event = Event;
	type Proposal = Call;
}

impl generic_asset::Trait for Runtime {
	type Balance = u128;
	type AssetId = u32;
	type Event = Event;
}

impl fees::Trait for Runtime {
	type Event = Event;
	type Currency = SpendingAssetCurrency<Self>;
	type BuyFeeAsset = ();
	type OnFeeCharged = ();
	type Fee = Fee;
}

impl cennzx_spot::Trait for Runtime {
	type Call = Call;
	type Event = Event;
	type ExchangeAddressGenerator = ExchangeAddressGenerator<Self>;
	type BalanceToU128 = Balance;
	type U128ToBalance = Balance;
}

/// Used for the module template in `./template.rs`
impl template::Trait for Runtime {
	type Event = Event;
}

construct_runtime!(
	pub enum Runtime with Log(InternalLog: DigestItem<Hash, AuthorityId, AuthoritySignature>) where
		Block = Block,
		NodeBlock = opaque::Block,
		UncheckedExtrinsic = UncheckedExtrinsic
	{
		System: system::{default, Log(ChangesTrieRoot)},
		Timestamp: timestamp::{Module, Call, Storage, Config<T>, Inherent},
		Consensus: consensus::{Module, Call, Storage, Config<T>, Log(AuthoritiesChange), Inherent},
		Aura: aura::{Module},
		Indices: indices,
		GenericAsset: generic_asset::{Module, Call, Storage, Config<T>, Event<T>, Fee},
		Session: session,
		Staking: staking,
		Grandpa: grandpa::{Module, Call, Storage, Config<T>, Log(), Event<T>},
		Contract: contract::{Module, Call, Storage, Config<T>, Event<T>},
		Sudo: sudo,
		Fees: fees::{Module, Call, Fee, Storage, Config<T>, Event<T>},
		CennzxSpot: cennzx_spot::{Module, Call, Storage, Config<T>, Event<T>},
		// Used for the module template in `./template.rs`
		TemplateModule: template::{Module, Call, Storage, Event<T>},
	}
);

/// The type used as a helper for interpreting the sender of transactions.
type Context = system::ChainContext<Runtime>;
/// The address format for describing accounts.
type Address = <Indices as StaticLookup>::Source;
/// Block header type as expected by this runtime.
pub type Header = generic::Header<BlockNumber, BlakeTwo256, Log>;
/// Block type as expected by this runtime.
pub type Block = generic::Block<Header, UncheckedExtrinsic>;
/// BlockId type as expected by this runtime.
pub type BlockId = generic::BlockId<Block>;
/// Unchecked extrinsic type as expected by this runtime.
pub type UncheckedExtrinsic = CennznetExtrinsic<AccountId, Address, Index, Call, Signature, Balance>;
/// Extrinsic type that has already been checked.
pub type CheckedExtrinsic = <<Block as BlockT>::Extrinsic as Checkable<system::ChainContext<Runtime>>>::Checked;
/// A type that handles payment for extrinsic fees
pub type ExtrinsicFeePayment = fee::ExtrinsicFeeCharger;
/// Executive: handles dispatch to the various modules.
pub type Executive = executive::Executive<Runtime, Block, Context, ExtrinsicFeePayment, AllModules>;

// Implement our runtime API endpoints. This is just a bunch of proxying.
impl_runtime_apis! {
	impl client_api::Core<Block> for Runtime {
		fn version() -> RuntimeVersion {
			VERSION
		}

		fn execute_block(block: Block) {
			Executive::execute_block(block)
		}

		fn initialize_block(header: &<Block as BlockT>::Header) {
			Executive::initialize_block(header)
		}

		fn authorities() -> Vec<AuthorityIdFor<Block>> {
			panic!("Deprecated, please use `AuthoritiesApi`.")
		}
	}

	impl client_api::Metadata<Block> for Runtime {
		fn metadata() -> OpaqueMetadata {
			Runtime::metadata().into()
		}
	}

	impl block_builder_api::BlockBuilder<Block> for Runtime {
		fn apply_extrinsic(extrinsic: <Block as BlockT>::Extrinsic) -> ApplyResult {
			Executive::apply_extrinsic(extrinsic)
		}

		fn finalize_block() -> <Block as BlockT>::Header {
			Executive::finalize_block()
		}

		fn inherent_extrinsics(data: InherentData) -> Vec<<Block as BlockT>::Extrinsic> {
			data.create_extrinsics()
		}

		fn check_inherents(block: Block, data: InherentData) -> CheckInherentsResult {
			data.check_extrinsics(&block)
		}

		fn random_seed() -> <Block as BlockT>::Hash {
			System::random_seed()
		}
	}

	impl client_api::TaggedTransactionQueue<Block> for Runtime {
		fn validate_transaction(tx: <Block as BlockT>::Extrinsic) -> TransactionValidity {
			Executive::validate_transaction(tx)
		}
	}

	impl offchain_primitives::OffchainWorkerApi<Block> for Runtime {
		fn offchain_worker(number: NumberFor<Block>) {
			Executive::offchain_worker(number)
		}
	}

	impl fg_primitives::GrandpaApi<Block> for Runtime {
		fn grandpa_pending_change(digest: &DigestFor<Block>)
			-> Option<ScheduledChange<NumberFor<Block>>>
		{
			for log in digest.logs.iter().filter_map(|l| match l {
				Log(InternalLog::grandpa(grandpa_signal)) => Some(grandpa_signal),
				_=> None
			}) {
				if let Some(change) = Grandpa::scrape_digest_change(log) {
					return Some(change);
				}
			}
			None
		}

		fn grandpa_forced_change(digest: &DigestFor<Block>)
			-> Option<(NumberFor<Block>, ScheduledChange<NumberFor<Block>>)>
		{
			for log in digest.logs.iter().filter_map(|l| match l {
				Log(InternalLog::grandpa(grandpa_signal)) => Some(grandpa_signal),
				_ => None
			}) {
				if let Some(change) = Grandpa::scrape_digest_forced_change(log) {
					return Some(change);
				}
			}
			None
		}

		fn grandpa_authorities() -> Vec<(AuthorityId, u64)> {
			Grandpa::grandpa_authorities()
		}
	}

	impl consensus_aura::AuraApi<Block> for Runtime {
		fn slot_duration() -> u64 {
			Aura::slot_duration()
		}
	}

	impl consensus_authorities::AuthoritiesApi<Block> for Runtime {
		fn authorities() -> Vec<AuthorityIdFor<Block>> {
			Consensus::authorities()
		}
	}
}
