#![cfg_attr(not(feature = "std"), no_std)]
// `construct_runtime!` does a lot of recursion and requires us to increase the limit to 256.
#![recursion_limit = "256"]
// let clippy disregard expressions like 1 * UNITS
#![allow(clippy::identity_op)]

// Make the WASM binary available.
#[cfg(feature = "std")]
include!(concat!(env!("OUT_DIR"), "/wasm_binary.rs"));

use frame_support::{
  traits::{AsEnsureOriginWithArg, Imbalance, OnUnbalanced},
  PalletId,
};
use frame_system::EnsureSigned;
use pallet_grandpa::{fg_primitives, AuthorityId as GrandpaId, AuthorityList as GrandpaAuthorityList};
use pallet_transaction_payment::Multiplier;
use smallvec::smallvec;
use sp_api::impl_runtime_apis;
use sp_consensus_aura::sr25519::AuthorityId as AuraId;
use sp_core::{crypto::KeyTypeId, OpaqueMetadata};
use sp_runtime::{
  create_runtime_str, generic, impl_opaque_keys,
  traits::{AccountIdLookup, BlakeTwo256, Block as BlockT, ConvertInto, IdentifyAccount, NumberFor, Verify},
  transaction_validity::{TransactionSource, TransactionValidity},
  ApplyExtrinsicResult, FixedPointNumber, MultiSignature, Perquintill,
};
use sp_std::prelude::*;
#[cfg(feature = "std")]
use sp_version::NativeVersion;
use sp_version::RuntimeVersion;

// A few exports that help ease life for downstream crates.
use frame_support::weights::{
  ConstantMultiplier, WeightToFeeCoefficient, WeightToFeeCoefficients, WeightToFeePolynomial,
};
pub use frame_support::{
  construct_runtime, parameter_types,
  traits::{ConstU128, ConstU32, ConstU64, ConstU8, EqualPrivilegeOnly, KeyOwnerProofSystem, Randomness, StorageInfo},
  weights::{
    constants::{BlockExecutionWeight, ExtrinsicBaseWeight, RocksDbWeight, WEIGHT_PER_SECOND},
    IdentityFee, Weight,
  },
  StorageValue,
};
pub use frame_system::Call as SystemCall;
use frame_system::EnsureRoot;
pub use pallet_balances::Call as BalancesCall;
use pallet_balances::NegativeImbalance;
pub use pallet_timestamp::Call as TimestampCall;
use pallet_transaction_payment::{CurrencyAdapter, TargetedFeeAdjustment};
#[cfg(any(feature = "std", test))]
pub use sp_runtime::BuildStorage;
pub use sp_runtime::{Perbill, Permill};

/// Constant values used within the runtime.
pub mod constants;
pub use constants::*;
use constants::{currency::*, time::*};

/// Weights used in the runtime.
pub mod weights;

#[cfg(test)]
mod tests;

/// Importing anagolay pallet
pub use anagolay_support;

/// Importing operations pallet
pub use operations;

/// Importing statements pallet
pub use statements;

/// Importing workflows pallet
pub use workflows;

use crate::constants::currency;
/// Importing poe pallet
pub use poe;

/// An index to a block.
pub type BlockNumber = u32;

/// Alias to 512-bit hash when used in the context of a transaction signature on the chain.
pub type Signature = MultiSignature;

/// Some way of identifying an account on the chain. We intentionally make it equivalent
/// to the public key of our transaction signing scheme.
pub type AccountId = <<Signature as Verify>::Signer as IdentifyAccount>::AccountId;

/// Balance of an account.
pub type Balance = u128;

/// Index of a transaction in the chain.
pub type Index = u32;

/// A hash of some data used by the chain.
pub type Hash = sp_core::H256;

/// Type used for expressing timestamp.
pub type Moment = u64;

/// Opaque types. These are used by the CLI to instantiate machinery that don't need to know
/// the specifics of the runtime. They can then be made to be agnostic over specific formats
/// of data like extrinsics, allowing for them to continue syncing the network through upgrades
/// to even the core data structures.
pub mod opaque {
  use super::*;

  pub use sp_runtime::OpaqueExtrinsic as UncheckedExtrinsic;

  /// Opaque block header type.
  pub type Header = generic::Header<BlockNumber, BlakeTwo256>;
  /// Opaque block type.
  pub type Block = generic::Block<Header, UncheckedExtrinsic>;
  /// Opaque block identifier type.
  pub type BlockId = generic::BlockId<Block>;

  impl_opaque_keys! {
      pub struct SessionKeys {
          pub aura: Aura,
          pub grandpa: Grandpa,
      }
  }
}

// To learn more about runtime versioning and what each of the following value means:
//   https://docs.substrate.io/v3/runtime/upgrades#runtime-versioning
#[sp_version::runtime_version]
pub const VERSION: RuntimeVersion = RuntimeVersion {
  spec_name: create_runtime_str!("idiyanale"),
  impl_name: create_runtime_str!("idiyanale"),
  authoring_version: 1,
  // The version of the runtime specification. A full node will not attempt to use its native
  //   runtime in substitute for the on-chain Wasm runtime unless all of `spec_name`,
  //   `spec_version`, and `authoring_version` are the same between Wasm and native.
  // This value is set to 100 to notify Polkadot-JS App (https://polkadot.js.org/apps) to use
  //   the compatible custom types.
  spec_version: 115,
  impl_version: 1,
  apis: RUNTIME_API_VERSIONS,
  transaction_version: 1,
  state_version: 1,
};

/// The version information used to identify this runtime when compiled natively.
#[cfg(feature = "std")]
pub fn native_version() -> NativeVersion {
  NativeVersion {
    runtime_version: VERSION,
    can_author_with: Default::default(),
  }
}

const NORMAL_DISPATCH_RATIO: Perbill = Perbill::from_percent(75);

parameter_types! {
  pub const BlockHashCount: BlockNumber = 2400;
  pub const Version: RuntimeVersion = VERSION;
  /// We allow for 2 seconds of compute with a 6 second average block time.
  pub BlockWeights: frame_system::limits::BlockWeights = frame_system::limits::BlockWeights
    ::with_sensible_defaults(2 * WEIGHT_PER_SECOND, NORMAL_DISPATCH_RATIO);
  pub BlockLength: frame_system::limits::BlockLength = frame_system::limits::BlockLength
    ::max_with_normal_ratio(5 * 1024 * 1024, NORMAL_DISPATCH_RATIO);
  pub const SS58Prefix: u8 = 42;
  pub MaximumSchedulerWeight: Weight = 10_000_000;
  pub const MaxScheduledPerBlock: u32 = 50;
}

// Configure FRAME pallets to include in runtime.

impl frame_system::Config for Runtime {
  /// The basic call filter to use in dispatchable.
  type BaseCallFilter = frame_support::traits::Everything;
  /// Block & extrinsics weights: base values and limits.
  type BlockWeights = BlockWeights;
  /// The maximum length of a block (in bytes).
  type BlockLength = BlockLength;
  /// The identifier used to distinguish between accounts.
  type AccountId = AccountId;
  /// The aggregated dispatch type that is available for extrinsics.
  type Call = Call;
  /// The lookup mechanism to get account ID from whatever is passed in dispatchers.
  type Lookup = AccountIdLookup<AccountId, ()>;
  /// The index type for storing how many extrinsics an account has signed.
  type Index = Index;
  /// The index type for blocks.
  type BlockNumber = BlockNumber;
  /// The type for hashing blocks and tries.
  type Hash = Hash;
  /// The hashing algorithm used.
  type Hashing = BlakeTwo256;
  /// The header type.
  type Header = generic::Header<BlockNumber, BlakeTwo256>;
  /// The ubiquitous event type.
  type Event = Event;
  /// The ubiquitous origin type.
  type Origin = Origin;
  /// Maximum number of block number to block hash mappings to keep (oldest pruned first).
  type BlockHashCount = BlockHashCount;
  /// The weight of database operations that the runtime can invoke.
  type DbWeight = RocksDbWeight;
  /// Version of the runtime.
  type Version = Version;
  /// Converts a module to the index of the module in `construct_runtime!`.
  ///
  /// This type is being generated by `construct_runtime!`.
  type PalletInfo = PalletInfo;
  /// What to do if a new account is created.
  type OnNewAccount = ();
  /// What to do if an account is fully reaped from the system.
  type OnKilledAccount = ();
  /// The data to be stored in an account.
  type AccountData = pallet_balances::AccountData<Balance>;
  /// Weight information for the extrinsics of this pallet.
  type SystemWeightInfo = ();
  /// This is used as an identifier of the chain. 42 is the generic substrate prefix.
  type SS58Prefix = SS58Prefix;
  /// The set code logic, just the default since we're not a parachain.
  type OnSetCode = ();
  type MaxConsumers = frame_support::traits::ConstU32<16>;
}

impl pallet_randomness_collective_flip::Config for Runtime {}

impl pallet_aura::Config for Runtime {
  type AuthorityId = AuraId;
  type DisabledValidators = ();
  type MaxAuthorities = ConstU32<32>;
}

impl pallet_grandpa::Config for Runtime {
  type Event = Event;
  type Call = Call;

  type KeyOwnerProofSystem = ();

  type KeyOwnerProof = <Self::KeyOwnerProofSystem as KeyOwnerProofSystem<(KeyTypeId, GrandpaId)>>::Proof;

  type KeyOwnerIdentification =
    <Self::KeyOwnerProofSystem as KeyOwnerProofSystem<(KeyTypeId, GrandpaId)>>::IdentificationTuple;

  type HandleEquivocation = ();

  type WeightInfo = ();
  type MaxAuthorities = ConstU32<32>;
}

impl pallet_timestamp::Config for Runtime {
  /// A timestamp: milliseconds since the unix epoch.
  type Moment = u64;
  type OnTimestampSet = Aura;
  type MinimumPeriod = ConstU64<{ SLOT_DURATION / 2 }>;
  type WeightInfo = ();
}

impl pallet_balances::Config for Runtime {
  type MaxLocks = ConstU32<50>;
  type MaxReserves = ();
  type ReserveIdentifier = [u8; 8];
  /// The type for recording an account's balance.
  type Balance = Balance;
  /// The ubiquitous event type.
  type Event = Event;
  type DustRemoval = ();
  type ExistentialDeposit = ConstU128<{ 10 * MILLI }>;
  type AccountStore = System;
  type WeightInfo = pallet_balances::weights::SubstrateWeight<Runtime>;
}

parameter_types! {
  pub const ProposalBond: Permill = Permill::from_percent(5);
  pub const TreasuryId: PalletId = PalletId(*b"py/trsry");
}

impl pallet_treasury::Config for Runtime {
  type PalletId = TreasuryId;
  type Currency = Balances;
  // root is required to approve a proposal
  type ApproveOrigin = EnsureRoot<AccountId>;
  // root is required to reject a proposal
  type RejectOrigin = EnsureRoot<AccountId>;
  type Event = Event;
  // If spending proposal rejected, transfer proposer bond to treasury
  type OnSlash = Treasury;
  type ProposalBond = ProposalBond;
  type ProposalBondMinimum = ConstU128<{ 1 * UNITS }>;
  type SpendPeriod = ConstU32<{ 6 * DAYS }>;
  type Burn = ();
  type BurnDestination = ();
  type MaxApprovals = ConstU32<100>;
  type WeightInfo = pallet_treasury::weights::SubstrateWeight<Runtime>;
  type SpendFunds = ();
  type ProposalBondMaximum = ();
  type SpendOrigin = frame_support::traits::NeverEnsureOrigin<Balance>; // Same as Polkadot
}

pub struct DealWithFees<R>(sp_std::marker::PhantomData<R>);
impl<R> OnUnbalanced<NegativeImbalance<R>> for DealWithFees<R>
where
  R: pallet_balances::Config + pallet_treasury::Config,
  pallet_treasury::Pallet<R>: OnUnbalanced<NegativeImbalance<R>>,
{
  // this is called for substrate-based transactions
  fn on_unbalanceds<B>(mut fees_then_tips: impl Iterator<Item = NegativeImbalance<R>>) {
    if let Some(fees) = fees_then_tips.next() {
      // for fees, 80% are burned, 20% to the treasury
      let (_, to_treasury) = fees.ration(80, 20);
      // Balances pallet automatically burns dropped Negative Imbalances by decreasing
      // total_supply accordingly
      <pallet_treasury::Pallet<R> as OnUnbalanced<_>>::on_unbalanced(to_treasury);
    }
  }
}

pub struct LengthToFee;
impl WeightToFeePolynomial for LengthToFee {
  type Balance = Balance;

  fn polynomial() -> WeightToFeeCoefficients<Self::Balance> {
    smallvec![
      WeightToFeeCoefficient {
        degree: 1,
        coeff_frac: Perbill::zero(),
        coeff_integer: currency::TRANSACTION_BYTE_FEE,
        negative: false,
      },
      WeightToFeeCoefficient {
        degree: 3,
        coeff_frac: Perbill::zero(),
        coeff_integer: constants::currency::SUPPLY_FACTOR,
        negative: false,
      },
    ]
  }
}

parameter_types! {
  /// The portion of the `NORMAL_DISPATCH_RATIO` that we adjust the fees with. Blocks filled less
  /// than this will decrease the weight and more will increase.
  pub const TargetBlockFullness: Perquintill = Perquintill::from_percent(25);
  /// The adjustment variable of the runtime. Higher values will cause `TargetBlockFullness` to
  /// change the fees more rapidly. This low value causes changes to occur slowly over time.
  pub AdjustmentVariable: Multiplier = Multiplier::saturating_from_rational(3, 100_000);
  /// Minimum amount of the multiplier. This value cannot be too low. A test case should ensure
  /// that combined with `AdjustmentVariable`, we can recover from the minimum.
  /// See `multiplier_can_grow_from_zero` in integration_tests.rs.
  /// This value is currently only used by pallet-transaction-payment as an assertion that the
  /// next multiplier is always > min value.
  pub MinimumMultiplier: Multiplier = Multiplier::saturating_from_rational(1, 1_000_000u128);
}

/// Parameterized slow adjusting fee updated based on
/// <https://w3f-research.readthedocs.io/en/latest/polkadot/overview/2-token-economics.html#-2.-slow-adjusting-mechanism>
///
/// The adjustment algorithm boils down to:
///
/// diff = (previous_block_weight - target) / maximum_block_weight
/// next_multiplier = prev_multiplier * (1 + (v * diff) + ((v * diff)^2 / 2))
/// assert(next_multiplier > min)
///     where: v is AdjustmentVariable
///            target is TargetBlockFullness
///            min is MinimumMultiplier
pub type SlowAdjustingFeeUpdate<R> =
  TargetedFeeAdjustment<R, TargetBlockFullness, AdjustmentVariable, MinimumMultiplier>;

impl pallet_transaction_payment::Config for Runtime {
  type Event = Event;
  type OnChargeTransaction = CurrencyAdapter<Balances, DealWithFees<Runtime>>;
  type OperationalFeeMultiplier = ConstU8<5>;
  type WeightToFee = ConstantMultiplier<Balance, ConstU128<{ currency::WEIGHT_FEE }>>;
  type LengthToFee = LengthToFee;
  type FeeMultiplierUpdate = SlowAdjustingFeeUpdate<Runtime>;
}

impl pallet_sudo::Config for Runtime {
  type Event = Event;
  type Call = Call;
}

parameter_types! {
  pub const MinVestedTransfer: Balance = 1 * UNITS;
}

impl pallet_utility::Config for Runtime {
  type Event = Event;
  type Call = Call;
  type PalletsOrigin = OriginCaller;
  type WeightInfo = weights::pallet_utility::WeightInfo<Runtime>;
}

impl pallet_vesting::Config for Runtime {
  type Event = Event;
  type Currency = Balances;
  type BlockNumberToBalance = ConvertInto;
  type MinVestedTransfer = MinVestedTransfer;
  type WeightInfo = weights::pallet_vesting::WeightInfo<Runtime>;

  const MAX_VESTING_SCHEDULES: u32 = 28;
}

impl pallet_scheduler::Config for Runtime {
  type Event = Event;
  type Origin = Origin;
  type PalletsOrigin = OriginCaller;
  type Call = Call;
  type MaximumWeight = MaximumSchedulerWeight;
  type ScheduleOrigin = frame_system::EnsureRoot<AccountId>;
  type MaxScheduledPerBlock = MaxScheduledPerBlock;
  type WeightInfo = ();
  type OriginPrivilegeCmp = EqualPrivilegeOnly;
  type PreimageProvider = ();
  type NoPreimagePostponement = ();
}

parameter_types! {
  pub const CollectionDeposit: Balance = 10 * UNITS; // 10 UNIT deposit to create uniques class
  pub const ItemDeposit: Balance = UNITS / 100; // 1 / 100 UNIT deposit to create uniques instance
  pub const KeyLimit: u32 = 32;	// Max 32 bytes per key
  pub const ValueLimit: u32 = 64;	// Max 64 bytes per value
  pub const UniquesMetadataDepositBase: Balance = deposit(1, 129);
  pub const AttributeDepositBase: Balance = deposit(1, 0);
  pub const DepositPerByte: Balance = deposit(0, 1);
  pub const UniquesStringLimit: u32 = 128;
}

impl pallet_uniques::Config for Runtime {
  type Event = Event;
  type CollectionId = u32;
  type ItemId = u32;
  type Currency = Balances;
  type ForceOrigin = EnsureRoot<AccountId>;
  type CollectionDeposit = CollectionDeposit;
  type ItemDeposit = ItemDeposit;
  type MetadataDepositBase = UniquesMetadataDepositBase;
  type AttributeDepositBase = AttributeDepositBase;
  type DepositPerByte = DepositPerByte;
  type StringLimit = UniquesStringLimit;
  type KeyLimit = KeyLimit;
  type ValueLimit = ValueLimit;
  type WeightInfo = weights::pallet_uniques::WeightInfo<Runtime>;
  #[cfg(feature = "runtime-benchmarks")]
  type Helper = ();
  type CreateOrigin = AsEnsureOriginWithArg<EnsureSigned<AccountId>>;
  type Locker = ();
}

// Anagolay pallets:
// ------------------------------------------------------------------------------------------------
impl anagolay_support::Config for Runtime {
  const MAX_ARTIFACTS: u32 = 1_000_000;
}

impl operations::Config for Runtime {
  type Event = Event;
  type WeightInfo = operations::weights::AnagolayWeight<Runtime>;
  type TimeProvider = pallet_timestamp::Pallet<Runtime>;

  const MAX_VERSIONS_PER_OPERATION: u32 = 100;
}

impl statements::Config for Runtime {
  type Event = Event;
  type WeightInfo = statements::weights::AnagolayWeight<Runtime>;

  const MAX_STATEMENTS_PER_PROOF: u32 = 16;
}

impl workflows::Config for Runtime {
  type Event = Event;
  type WeightInfo = workflows::weights::AnagolayWeight<Runtime>;
  type TimeProvider = pallet_timestamp::Pallet<Runtime>;

  const MAX_VERSIONS_PER_WORKFLOW: u32 = 100;
}

impl poe::Config for Runtime {
  type Event = Event;
  type WeightInfo = poe::weights::AnagolayWeight<Runtime>;

  const MAX_PROOFS_PER_WORKFLOW: u32 = 1;
}

impl verification::Config for Runtime {
  type AuthorityId = verification::crypto::VerificationAuthId;
  type Event = Event;
  type VerificationKeyGenerator = poe::types::PoeVerificationKeyGenerator<Runtime>;
  type VerificationInvalidator = statements::types::StatementsVerificationInvalidator<Runtime>;
  type WeightInfo = verification::weights::AnagolayWeight<Runtime>;
  type Currency = Balances;

  const REGISTRATION_FEE: u128 = 1 * UNITS;
  const MAX_REQUESTS_PER_CONTEXT: u32 = 1000;
}

impl frame_system::offchain::SigningTypes for Runtime {
  type Public = <Signature as Verify>::Signer;
  type Signature = Signature;
}

impl<C> frame_system::offchain::SendTransactionTypes<C> for Runtime
where
  Call: From<C>,
{
  type OverarchingCall = Call;
  type Extrinsic = UncheckedExtrinsic;
}

impl tipping::Config for Runtime {
  type Event = Event;
  type Currency = Balances;
  type TimeProvider = pallet_timestamp::Pallet<Runtime>;
  type WeightInfo = tipping::weights::AnagolayWeight<Runtime>;

  const MAX_TIPS_PER_VERIFICATION_CONTEXT: u32 = 10000;
}

// Create the runtime by composing the FRAME pallets that were previously configured.
construct_runtime!(
  pub enum Runtime where
    Block = Block,
    NodeBlock = opaque::Block,
    UncheckedExtrinsic = UncheckedExtrinsic
  {
    System: frame_system = 0,
    RandomnessCollectiveFlip: pallet_randomness_collective_flip = 1,
    Timestamp: pallet_timestamp = 2,
    Aura: pallet_aura = 3,
    Grandpa: pallet_grandpa = 4,
    Balances: pallet_balances = 5,
    TransactionPayment: pallet_transaction_payment = 6,
    Sudo: pallet_sudo = 7,
    Treasury: pallet_treasury = 8,

    // Customizations
    Utility: pallet_utility::{Pallet, Call, Event} = 9,
    // Vesting. Usable initially, but removed once all vesting is finished.
    Vesting: pallet_vesting::{Pallet, Call, Storage, Event<T>, Config<T>} = 10,
    Scheduler: pallet_scheduler = 11,
    Uniques: pallet_uniques::{Pallet, Call, Storage, Event<T>} = 12,

    // Used for anagolay blockchain
    Anagolay: anagolay_support::{Pallet} = 13,
    Operations: operations::{Pallet, Call, Storage, Event<T>, Config<T>} = 14,
    Poe: poe::{Pallet, Call, Storage, Event<T>} = 15,
    Statements: statements::{Pallet, Call, Storage, Event<T>} = 16,
    Workflows: workflows::{Pallet, Call, Storage, Event<T>, Config<T>} = 17,

    // Verification and Tipping
    Verification: verification::{Pallet, Call, Storage, Event<T>, ValidateUnsigned} = 18,
    Tipping: tipping::{Pallet, Call, Storage, Event<T>} = 19,
  }
);

/// The address format for describing accounts.
pub type Address = sp_runtime::MultiAddress<AccountId, ()>;
/// Block header type as expected by this runtime.
pub type Header = generic::Header<BlockNumber, BlakeTwo256>;
/// Block type as expected by this runtime.
pub type Block = generic::Block<Header, UncheckedExtrinsic>;
/// The SignedExtension to the basic transaction logic.
pub type SignedExtra = (
  frame_system::CheckNonZeroSender<Runtime>,
  frame_system::CheckSpecVersion<Runtime>,
  frame_system::CheckTxVersion<Runtime>,
  frame_system::CheckGenesis<Runtime>,
  frame_system::CheckEra<Runtime>,
  frame_system::CheckNonce<Runtime>,
  frame_system::CheckWeight<Runtime>,
  pallet_transaction_payment::ChargeTransactionPayment<Runtime>,
);
/// Unchecked extrinsic type as expected by this runtime.
pub type UncheckedExtrinsic = generic::UncheckedExtrinsic<Address, Call, Signature, SignedExtra>;
/// The payload being signed in transactions.
pub type SignedPayload = generic::SignedPayload<Call, SignedExtra>;
/// Executive: handles dispatch to the various modules.
pub type Executive =
  frame_executive::Executive<Runtime, Block, frame_system::ChainContext<Runtime>, Runtime, AllPalletsWithSystem>;

#[cfg(feature = "runtime-benchmarks")]
#[macro_use]
extern crate frame_benchmarking;

#[cfg(feature = "runtime-benchmarks")]
mod benches {
  define_benchmarks!(
    [frame_benchmarking, BaselineBench::<Runtime>]
    [frame_system, SystemBench::<Runtime>]
    [pallet_balances, Balances]
    [pallet_timestamp, Timestamp]
    [pallet_utility, Utility]
    [pallet_vesting, Vesting]
    [pallet_uniques, Uniques]
    [operations, Operations]
    [poe, Poe]
    [statements, Statements]
    [workflows, Workflows]
    [verification, Verification]
    [tipping, Tipping]
  );
}

impl_runtime_apis! {
    impl sp_api::Core<Block> for Runtime {
        fn version() -> RuntimeVersion {
            VERSION
        }

        fn execute_block(block: Block) {
            Executive::execute_block(block)
        }

        fn initialize_block(header: &<Block as BlockT>::Header) {
            Executive::initialize_block(header)
        }
    }

  impl sp_api::Metadata<Block> for Runtime {
    fn metadata() -> OpaqueMetadata {
      OpaqueMetadata::new(Runtime::metadata().into())
    }
  }

    impl sp_block_builder::BlockBuilder<Block> for Runtime {
        fn apply_extrinsic(extrinsic: <Block as BlockT>::Extrinsic) -> ApplyExtrinsicResult {
            Executive::apply_extrinsic(extrinsic)
        }

        fn finalize_block() -> <Block as BlockT>::Header {
            Executive::finalize_block()
        }

        fn inherent_extrinsics(data: sp_inherents::InherentData) -> Vec<<Block as BlockT>::Extrinsic> {
            data.create_extrinsics()
        }

        fn check_inherents(
            block: Block,
            data: sp_inherents::InherentData,
        ) -> sp_inherents::CheckInherentsResult {
            data.check_extrinsics(&block)
        }
    }

  impl sp_transaction_pool::runtime_api::TaggedTransactionQueue<Block> for Runtime {
    fn validate_transaction(
      source: TransactionSource,
      tx: <Block as BlockT>::Extrinsic,
      block_hash: <Block as BlockT>::Hash,
    ) -> TransactionValidity {
      Executive::validate_transaction(source, tx, block_hash)
    }
  }

  impl sp_offchain::OffchainWorkerApi<Block> for Runtime {
    fn offchain_worker(header: &<Block as BlockT>::Header) {
      Executive::offchain_worker(header)
    }
  }

  impl sp_consensus_aura::AuraApi<Block, AuraId> for Runtime {
    fn slot_duration() -> sp_consensus_aura::SlotDuration {
      sp_consensus_aura::SlotDuration::from_millis(Aura::slot_duration())
    }

    fn authorities() -> Vec<AuraId> {
      Aura::authorities().into_inner()
    }
  }

  impl sp_session::SessionKeys<Block> for Runtime {
    fn generate_session_keys(seed: Option<Vec<u8>>) -> Vec<u8> {
      opaque::SessionKeys::generate(seed)
    }

    fn decode_session_keys(
      encoded: Vec<u8>,
    ) -> Option<Vec<(Vec<u8>, KeyTypeId)>> {
      opaque::SessionKeys::decode_into_raw_public_keys(&encoded)
    }
  }

  impl fg_primitives::GrandpaApi<Block> for Runtime {
    fn grandpa_authorities() -> GrandpaAuthorityList {
      Grandpa::grandpa_authorities()
    }

    fn current_set_id() -> fg_primitives::SetId {
      Grandpa::current_set_id()
    }

    fn submit_report_equivocation_unsigned_extrinsic(
      _equivocation_proof: fg_primitives::EquivocationProof<
        <Block as BlockT>::Hash,
        NumberFor<Block>,
      >,
      _key_owner_proof: fg_primitives::OpaqueKeyOwnershipProof,
    ) -> Option<()> {
      None
    }

    fn generate_key_ownership_proof(
      _set_id: fg_primitives::SetId,
      _authority_id: GrandpaId,
    ) -> Option<fg_primitives::OpaqueKeyOwnershipProof> {
      // NOTE: this is the only implementation possible since we've
      // defined our key owner proof type as a bottom type (i.e. a type
      // with no values).
      None
    }
  }

  impl frame_system_rpc_runtime_api::AccountNonceApi<Block, AccountId, Index> for Runtime {
    fn account_nonce(account: AccountId) -> Index {
      System::account_nonce(account)
    }
  }

  impl pallet_transaction_payment_rpc_runtime_api::TransactionPaymentApi<Block, Balance> for Runtime {
    fn query_info(
      uxt: <Block as BlockT>::Extrinsic,
      len: u32,
    ) -> pallet_transaction_payment_rpc_runtime_api::RuntimeDispatchInfo<Balance> {
      TransactionPayment::query_info(uxt, len)
    }
    fn query_fee_details(
      uxt: <Block as BlockT>::Extrinsic,
      len: u32,
    ) -> pallet_transaction_payment::FeeDetails<Balance> {
      TransactionPayment::query_fee_details(uxt, len)
    }
  }

  impl operations_rpc_runtime_api::OperationsApi<Block> for Runtime {
    fn get_operations_by_ids(
      operation_ids: Vec<operations::types::OperationId>,
      offset: u64,
      limit: u16,
    ) -> Vec<operations::types::Operation> {
      Operations::get_operations_by_ids(operation_ids, offset, limit)
    }
    fn get_operation_versions_by_ids(
      operation_versions_ids: Vec<operations::types::OperationVersionId>,
      offset: u64,
      limit: u16,
    ) -> Vec<operations::types::OperationVersion> {
      Operations::get_operation_versions_by_ids(operation_versions_ids, offset, limit)
    }
  }

  impl workflows_rpc_runtime_api::WorkflowsApi<Block> for Runtime {
    fn get_workflows_by_ids(
      workflow_ids: Vec<workflows::types::WorkflowId>,
      offset: u64,
      limit: u16,
    ) -> Vec<workflows::types::Workflow> {
      Workflows::get_workflows_by_ids(workflow_ids, offset, limit)
    }
    fn get_workflow_versions_by_ids(
      workflow_version_ids: Vec<workflows::types::WorkflowVersionId>,
      offset: u64,
      limit: u16,
    ) -> Vec<workflows::types::WorkflowVersion> {
      Workflows::get_workflow_versions_by_ids(workflow_version_ids, offset, limit)
    }
  }

  impl verification_rpc_runtime_api::VerificationApi<Block, AccountId> for Runtime {
    fn get_requests(
      contexts: Vec<verification::types::VerificationContext>,
      status: Option<verification::types::VerificationStatus>,
      offset: u64,
      limit: u16,
    ) -> Vec<verification::types::VerificationRequest<AccountId>> {
      Verification::get_requests(contexts, status, None, offset, limit)
    }
    fn get_requests_for_account(
      account: AccountId,
      status: Option<verification::types::VerificationStatus>,
      offset: u64,
      limit: u16,
    ) -> Vec<verification::types::VerificationRequest<AccountId>> {
      Verification::get_requests(vec![], status, Some(account), offset, limit)
    }
  }

  impl tipping_rpc_runtime_api::TippingApi<Block, Balance, AccountId, BlockNumber> for Runtime {
    fn total_received(account_id: AccountId, verification_context: verification::types::VerificationContext) -> Balance {
      Tipping::total_received(account_id, verification_context)
    }
    fn total(account_id: AccountId, verification_context: verification::types::VerificationContext) -> u64 {
      Tipping::total(account_id, verification_context)
    }
    fn get_tips (
      account_id: AccountId,
      verification_context: verification::types::VerificationContext,
      offset: u64,
      limit: u16,
    ) -> Vec<tipping::types::Tip<Balance, AccountId, BlockNumber>> {
      Tipping::get_tips(account_id, verification_context, offset, limit)
    }
  }

  #[cfg(feature = "runtime-benchmarks")]
  impl frame_benchmarking::Benchmark<Block> for Runtime {
    fn benchmark_metadata(extra: bool) -> (
      Vec<frame_benchmarking::BenchmarkList>,
      Vec<frame_support::traits::StorageInfo>,
    ) {
      use frame_benchmarking::{baseline, Benchmarking, BenchmarkList};
      use frame_support::traits::StorageInfoTrait;
      use frame_system_benchmarking::Pallet as SystemBench;
      use baseline::Pallet as BaselineBench;

      let mut list = Vec::<BenchmarkList>::new();
      list_benchmarks!(list, extra);

      let storage_info = AllPalletsWithSystem::storage_info();

      (list, storage_info)
    }

    fn dispatch_benchmark(
      config: frame_benchmarking::BenchmarkConfig
    ) -> Result<Vec<frame_benchmarking::BenchmarkBatch>, sp_runtime::RuntimeString> {
      use frame_benchmarking::{baseline, Benchmarking, BenchmarkBatch, TrackedStorageKey};

      use frame_system_benchmarking::Pallet as SystemBench;
      use baseline::Pallet as BaselineBench;

      impl frame_system_benchmarking::Config for Runtime {}
      impl baseline::Config for Runtime {}

      let whitelist: Vec<TrackedStorageKey> = vec![
        // Block Number
        hex_literal::hex!("26aa394eea5630e07c48ae0c9558cef702a5c1b19ab7a04f536c519aca4983ac").to_vec().into(),
        // Total Issuance
        hex_literal::hex!("c2261276cc9d1f8598ea4b6a74b15c2f57c875e4cff74148e4628f264b974c80").to_vec().into(),
        // Execution Phase
        hex_literal::hex!("26aa394eea5630e07c48ae0c9558cef7ff553b5a9862a516939d82b3d3d8661a").to_vec().into(),
        // Event Count
        hex_literal::hex!("26aa394eea5630e07c48ae0c9558cef70a98fdbe9ce6c55837576c60c7af3850").to_vec().into(),
        // System Events
        hex_literal::hex!("26aa394eea5630e07c48ae0c9558cef780d41e5e16056765bc8461851072c9d7").to_vec().into(),
      ];

      let mut batches = Vec::<BenchmarkBatch>::new();
      let params = (&config, &whitelist);
      add_benchmarks!(params, batches);

      Ok(batches)
    }
  }

  #[cfg(feature = "try-runtime")]
  impl frame_try_runtime::TryRuntime<Block> for Runtime {
    fn on_runtime_upgrade() -> (Weight, Weight) {
      // NOTE: intentional unwrap: we don't want to propagate the error backwards, and want to
      // have a backtrace here. If any of the pre/post migration checks fail, we shall stop
      // right here and right now.
      let weight = Executive::try_runtime_upgrade().unwrap();
      (weight, BlockWeights::get().max_block)
    }

    fn execute_block_no_check(block: Block) -> Weight {
      Executive::execute_block_no_check(block)
    }
  }
}
