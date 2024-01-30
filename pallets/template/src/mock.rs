//! Mock helpers for Validator Set pallet.

#![cfg(test)]

use super::*;
use crate as pallet_template;
// use crate::{constants::*};
// use constants::{currency::*, time::*};

use frame_support::{
	parameter_types,
	traits::{
		ConstU128, ConstU32, ConstU64, ConstU8, KeyOwnerProofSystem, Randomness, StorageInfo, AsEnsureOriginWithArg
	},


};

use sp_consensus_babe::{
	AllowedSlots, BabeEpochConfiguration, Slot, VrfSignature, RANDOMNESS_LENGTH,
};

use frame_system::{EnsureRoot,  EnsureSigned};
use pallet_session::*;
use sp_core::{crypto::key_types::DUMMY, H256};
use sp_runtime::{
	impl_opaque_keys,
	testing::UintAuthorityId,
	traits::{BlakeTwo256, IdentityLookup, OpaqueKeys},
	BuildStorage, KeyTypeId, RuntimeAppPublic,
};
use sp_runtime::{
	create_runtime_str, generic, 
	traits::{
		Convert,
		AccountIdLookup,  Block as BlockT, IdentifyAccount, NumberFor, One, Verify, Identity, 
	},
	curve::PiecewiseLinear,
	transaction_validity::{TransactionSource, TransactionValidity, TransactionPriority},
	ApplyExtrinsicResult, MultiSignature,
};

// mod voter_bags;

use frame_election_provider_support::{
	onchain, BalancingConfig, ElectionDataProvider, SequentialPhragmen, VoteWeight,
};
use pallet_transaction_payment::{ConstFeeMultiplier, CurrencyAdapter, Multiplier};
use sp_state_machine::BasicExternalities;
use std::cell::RefCell;
use node_primitives::Moment;
pub use sp_runtime::{Perbill, Permill};


impl_opaque_keys! {
	pub struct MockSessionKeys {
		pub dummy:  UintAuthorityId,
	}
}

impl From<UintAuthorityId> for MockSessionKeys {
	fn from(dummy: UintAuthorityId) -> Self {
		Self { dummy }
	}
}

pub const KEY_ID_A: KeyTypeId = KeyTypeId([4; 4]);
pub const KEY_ID_B: KeyTypeId = KeyTypeId([9; 4]);

#[derive(Debug, Clone, codec::Encode, codec::Decode, PartialEq, Eq)]
pub struct PreUpgradeMockSessionKeys {
	pub a: [u8; 32],
	pub b: [u8; 64],
}

impl OpaqueKeys for PreUpgradeMockSessionKeys {
	type KeyTypeIdProviders = ();

	fn key_ids() -> &'static [KeyTypeId] {
		&[KEY_ID_A, KEY_ID_B]
	}

	fn get_raw(&self, i: KeyTypeId) -> &[u8] {
		match i {
			i if i == KEY_ID_A => &self.a[..],
			i if i == KEY_ID_B => &self.b[..],
			_ => &[],
		}
	}
}


type UncheckedExtrinsic = frame_system::mocking::MockUncheckedExtrinsic<Test>;
type Block = frame_system::mocking::MockBlock<Test>;

frame_support::construct_runtime!(
	pub struct Test
		// where
		// Block = Block,
		// NodeBlock = Block,
		// UncheckedExtrinsic = UncheckedExtrinsic
	{   
		System: frame_system, //::{Pallet, Call, Config<T>, Storage, Event<T>},
		TemplateModule: pallet_template,
		Session: pallet_session,
		Timestamp: pallet_timestamp,
		Babe: pallet_babe,
		//Grandpa: pallet_grandpa,
		Balances: pallet_balances,
		//TransactionPayment: pallet_transaction_payment,
		// Sudo: pallet_sudo,
		// // staking related pallets
		// ElectionProviderMultiPhase: pallet_election_provider_multi_phase,
		// Staking: pallet_staking,
		Assets: pallet_assets,
		//VoterList: pallet_bags_list::<Instance1>,
		// Historical: pallet_session::historical,
	
	}
);

/// Alias to 512-bit hash when used in the context of a transaction signature on the chain.
pub type Signature = MultiSignature;

/// Some way of identifying an account on the chain. We intentionally make it equivalent
/// to the public key of our transaction signing scheme.
//pub type AccountId = <<Signature as Verify>::Signer as IdentifyAccount>::AccountId;

thread_local! {
	pub static VALIDATORS: RefCell<Vec<u64>> = RefCell::new(vec![1, 2, 3]);
	pub static NEXT_VALIDATORS: RefCell<Vec<u64>> = RefCell::new(vec![1, 2, 3]);
	// pub static AUTHORITIES: RefCell<Vec<UintAuthorityId>> =
	// 	RefCell::new(vec![UintAuthorityId(1), UintAuthorityId(2), UintAuthorityId(3)]);
	pub static FORCE_SESSION_END: RefCell<bool> = RefCell::new(false);
	pub static SESSION_LENGTH: RefCell<u64> = RefCell::new(2);
	pub static SESSION_CHANGED: RefCell<bool> = RefCell::new(false);
	pub static DISABLED: RefCell<bool> = RefCell::new(false);
	pub static BEFORE_SESSION_END_CALLED: RefCell<bool> = RefCell::new(false);
}

// pub struct TestSessionHandler;
// impl SessionHandler<u64> for TestSessionHandler {
// 	const KEY_TYPE_IDS: &'static [sp_runtime::KeyTypeId] = &[UintAuthorityId::ID];
// 	fn on_genesis_session<T: OpaqueKeys>(_validators: &[(u64, T)]) {}
// 	fn on_new_session<T: OpaqueKeys>(
// 		changed: bool,
// 		validators: &[(u64, T)],
// 		_queued_validators: &[(u64, T)],
// 	) {
// 		SESSION_CHANGED.with(|l| *l.borrow_mut() = changed);
// 		AUTHORITIES.with(|l| {
// 			*l.borrow_mut() = validators
// 				.iter()
// 				.map(|(_, id)| id.get::<UintAuthorityId>(DUMMY).unwrap_or_default())
// 				.collect()
// 		});
// 	}
// 	fn on_disabled(_validator_index: u32) {
// 		DISABLED.with(|l| *l.borrow_mut() = true)
// 	}
// 	fn on_before_session_ending() {
// 		BEFORE_SESSION_END_CALLED.with(|b| *b.borrow_mut() = true);
// 	}
// }

// pub struct TestShouldEndSession;
// impl ShouldEndSession<u64> for TestShouldEndSession {
// 	fn should_end_session(now: u64) -> bool {
// 		let l = SESSION_LENGTH.with(|l| *l.borrow());
// 		now % l == 0 ||
// 			FORCE_SESSION_END.with(|l| {
// 				let r = *l.borrow();
// 				*l.borrow_mut() = false;
// 				r
// 			})
// 	}
// }


sp_runtime::impl_opaque_keys! {
	pub struct SessionKeys {
		pub foo: sp_runtime::testing::UintAuthorityId,
	}
}
type AccountId = u64;
pub struct TestSessionHandler;
impl pallet_session::SessionHandler<AccountId> for TestSessionHandler {
	const KEY_TYPE_IDS: &'static [sp_runtime::KeyTypeId] = &[];

	fn on_genesis_session<Ks: sp_runtime::traits::OpaqueKeys>(_validators: &[(AccountId, Ks)]) {}

	fn on_new_session<Ks: sp_runtime::traits::OpaqueKeys>(
		_: bool,
		_: &[(AccountId, Ks)],
		_: &[(AccountId, Ks)],
	) {
	}

	fn on_disabled(_: u32) {}
}

// pub fn authorities() -> Vec<UintAuthorityId> {
// 	AUTHORITIES.with(|l| l.borrow().to_vec())
// }

// pub fn new_test_ext() -> sp_io::TestExternalities {
	// let mut t = frame_system::GenesisConfig::<Test>::default().build_storage().unwrap();
	// let mut committee: Vec<u64> = vec![];
	// let keys: Vec<_> = NEXT_VALIDATORS
	// 	.with(|l| l.borrow().iter().cloned().map(|i| (i, i, UintAuthorityId(i).into())).collect());
	// BasicExternalities::execute_with_storage(&mut t, || {
	// 	for (ref k, ..) in &keys {
	// 		frame_system::Pallet::<Test>::inc_providers(k);
	// 	}
	// 	frame_system::Pallet::<Test>::inc_providers(&4);
	// 	frame_system::Pallet::<Test>::inc_providers(&69);
	// });
	// pallet_template::GenesisConfig::<Test> {
	// 	initial_committee: committee,
	// 	// initial_validators: keys.iter().map(|x| x.0).collect::<Vec<_>>(),
	// }
	// .assimilate_storage(&mut t)
	// .unwrap();
	// pallet_session::GenesisConfig::<Test> { keys: keys.clone() }
	// 	.assimilate_storage(&mut t)
	// 	.unwrap();
	// sp_io::TestExternalities::new(t)
// 	frame_system::GenesisConfig::<Test>::default().build_storage().unwrap().into()
// }

parameter_types! {
	pub const BlockHashCount: u64 = 250;
	pub BlockWeights: frame_system::limits::BlockWeights =
		frame_system::limits::BlockWeights::simple_max(frame_support::weights::Weight::from_parts(1024, 0));

		// pub const BlockHashCount: BlockNumber = 2400;
		// pub const Version: RuntimeVersion = VERSION;
		// /// We allow for 2 seconds of compute with a 6 second average block time.
		// pub RuntimeBlockWeights: frame_system::limits::BlockWeights =
		// 	frame_system::limits::BlockWeights::with_sensible_defaults(
		// 		Weight::from_parts(2u64 * WEIGHT_REF_TIME_PER_SECOND, u64::MAX),
		// 		NORMAL_DISPATCH_RATIO,
		// 	);
		pub RuntimeBlockLength: frame_system::limits::BlockLength = frame_system::limits::BlockLength
			::max_with_normal_ratio(5 * 1024 * 1024, NORMAL_DISPATCH_RATIO);
		// pub const SS58Prefix: u8 = 42;
}

impl frame_system::Config for Test {
	type BaseCallFilter = frame_support::traits::Everything;
	type BlockWeights = ();
	type BlockLength = ();
	type DbWeight = ();
	type RuntimeOrigin = RuntimeOrigin;
	type Nonce = u64;
	type RuntimeCall = RuntimeCall;
	type Hash = H256;
	type Hashing = BlakeTwo256;
	type AccountId = u64;
	type Lookup = IdentityLookup<Self::AccountId>;
	type Block = Block;
	type RuntimeEvent = RuntimeEvent;
	type BlockHashCount = BlockHashCount;
	type Version = ();
	type PalletInfo = PalletInfo;
	type AccountData = pallet_balances::AccountData<u128>;
	type OnNewAccount = ();
	type OnKilledAccount = ();
	type SystemWeightInfo = ();
	type SS58Prefix = ();
	type OnSetCode = ();
	type MaxConsumers = frame_support::traits::ConstU32<16>;
}

impl pallet_session::Config for Test {
	type ValidatorId = <Self as frame_system::Config>::AccountId;
	type ValidatorIdOf = pallet_template::ValidatorOf<Self>;
	type ShouldEndSession = Babe;
	type NextSessionRotation = Babe;
	type SessionManager = TemplateModule; //pallet_session::historical::NoteHistoricalRoot<Self, Staking>;
	type SessionHandler =  TestSessionHandler;//<MockSessionKeys as OpaqueKeys>::KeyTypeIdProviders;
	type Keys = MockSessionKeys;
	type WeightInfo = ();
	type RuntimeEvent = RuntimeEvent;
}

parameter_types! {
	pub const MinAuthorities: u32 = 2;
}

/// Balance of an account.
pub type Balance = u128;
type AssetBalance = Balance;
impl pallet_template::Config for Test {
	type XIndex = Assets;
	type XIndexBalance = AssetBalance;
	type AddRemoveOrigin = EnsureRoot<Self::AccountId>;
	type RuntimeEvent = RuntimeEvent;
	type MinAuthorities = MinAuthorities;
	type WeightInfo = ();
}


impl pallet_assets::Config for Test {
	type RuntimeEvent = RuntimeEvent;
    type Balance = AssetBalance;
    type AssetId = u32;
    type Currency = Balances;
    type ForceOrigin = EnsureRoot<Self::AccountId>;
    type AssetDeposit = ConstU128<1>;
    type AssetAccountDeposit = ConstU128<10>;
    type MetadataDepositBase = ConstU128<1>;
    type MetadataDepositPerByte = ConstU128<1>;
    type ApprovalDeposit = ConstU128<1>;
    type StringLimit = ConstU32<50>;
    type Freezer = ();
    type Extra = ();
    type WeightInfo = ();
	type CallbackHandle = ();
	type RemoveItemsLimit = ();
	type CreateOrigin = AsEnsureOriginWithArg<EnsureSigned<u64>>;
	type AssetIdParameter = codec::Compact<u32>;
}



pub const EXISTENTIAL_DEPOSIT: u128 = 500;

impl pallet_balances::Config for Test {
	type MaxLocks = ConstU32<50>;
	type MaxReserves = ();
	type ReserveIdentifier = [u8; 8];
	/// The type for recording an account's balance.
	type Balance = Balance;
	/// The ubiquitous event type.
	type RuntimeEvent = RuntimeEvent;
	type DustRemoval = ();
	type ExistentialDeposit = ConstU128<EXISTENTIAL_DEPOSIT>;
	type AccountStore = System;
	type WeightInfo = pallet_balances::weights::SubstrateWeight<Test>;
	type FreezeIdentifier = ();
	type MaxFreezes = ();
	type RuntimeHoldReason = ();
	type MaxHolds = ();
}
const NORMAL_DISPATCH_RATIO: Perbill = Perbill::from_percent(75);

pub const MILLICENTS: Balance = 1_000_000_000;
pub const CENTS: Balance = 1_000 * MILLICENTS; // assume this is worth about a cent.
pub const DOLLARS: Balance = 100 * CENTS;

pub const fn deposit(items: u32, bytes: u32) -> Balance {
	items as Balance * 15 * CENTS + (bytes as Balance) * 6 * CENTS
}

use node_primitives::{BlockNumber};

pub const MILLISECS_PER_BLOCK: Moment = 3000;
	pub const SECS_PER_BLOCK: Moment = MILLISECS_PER_BLOCK / 1000;

	// NOTE: Currently it is not possible to change the slot duration after the chain has started.
	//       Attempting to do so will brick block production.
	pub const SLOT_DURATION: Moment = MILLISECS_PER_BLOCK;

	// 1 in 4 blocks (on average, not counting collisions) will be primary BABE blocks.
	pub const PRIMARY_PROBABILITY: (u64, u64) = (1, 4);

	// NOTE: Currently it is not possible to change the epoch duration after the chain has started.
	//       Attempting to do so will brick block production.
	pub const EPOCH_DURATION_IN_BLOCKS: BlockNumber = 10 * MINUTES;
	pub const EPOCH_DURATION_IN_SLOTS: u64 = {
		const SLOT_FILL_RATE: f64 = MILLISECS_PER_BLOCK as f64 / SLOT_DURATION as f64;

		(EPOCH_DURATION_IN_BLOCKS as f64 * SLOT_FILL_RATE) as u64
	};

	// These time units are defined in number of blocks.
	pub const MINUTES: BlockNumber = 60 / (SECS_PER_BLOCK as BlockNumber);
	pub const HOURS: BlockNumber = MINUTES * 60;
	pub const DAYS: BlockNumber = HOURS * 24;

parameter_types! {
	
	// NOTE: Currently it is not possible to change the epoch duration after the chain has started.
	//       Attempting to do so will brick block production.
	pub const EpochDuration: u64 = EPOCH_DURATION_IN_SLOTS;
	pub const ExpectedBlockTime: Moment = MILLISECS_PER_BLOCK;
	pub const ReportLongevity: u64 = 24 * 28 * 6 * EpochDuration::get();
		// BondingDuration::get() as u64 * SessionsPerEra::get() as u64 * EpochDuration::get();
	pub const MaxAuthorities: u32 = 100;
}



impl pallet_babe::Config for Test {
	type EpochDuration = EpochDuration;
	type ExpectedBlockTime = ExpectedBlockTime;
	type EpochChangeTrigger = pallet_babe::SameAuthoritiesForever;
	type DisabledValidators = (); // TODO Session;
	type WeightInfo = ();
	type MaxAuthorities = MaxAuthorities;
	type KeyOwnerProof = <() as KeyOwnerProofSystem<(KeyTypeId, pallet_babe::AuthorityId)>>::Proof;
		// TODO <Historical as KeyOwnerProofSystem<(KeyTypeId, pallet_babe::AuthorityId)>>::Proof;
	type EquivocationReportSystem = ();
	type Committee = TemplateModule;
		// TODO pallet_babe::EquivocationReportSystem<Self, Offences, Historical, ReportLongevity>;
}

impl pallet_timestamp::Config for Test {
	/// A timestamp: milliseconds since the unix epoch.
	type Moment = u64;
	type OnTimestampSet = Babe;
	type MinimumPeriod = ConstU64<{ SLOT_DURATION / 2 }>;
	type WeightInfo = ();
}

// impl pallet_staking::Config for Runtime {
// 	type Currency = Balances;
// 	type CurrencyBalance = Balance;
// 	type UnixTime = Timestamp;
// 	type MaxNominations = ConstU32<16>;
// 	type CurrencyToVote = sp_staking::currency_to_vote::U128CurrencyToVote;
// 	type RewardRemainder = (); // TODO Treasury
// 	type RuntimeEvent = RuntimeEvent;
// 	type Slash = (); // TODO Treasury send the slashed funds to the treasury.
// 	type Reward = (); // rewards are minted from the void
// 	type SessionsPerEra = SessionsPerEra;
// 	type BondingDuration = BondingDuration;
// 	type SlashDeferDuration = SlashDeferDuration;
// 	/// TODO A super-majority of the council can cancel the slash.
// 	// type AdminOrigin = EitherOfDiverse<
// 	// 	EnsureRoot<AccountId>,
// 	// 	pallet_collective::EnsureProportionAtLeast<AccountId, CouncilCollective, 3, 4>,
// 	// >;
// 	type AdminOrigin = EnsureRoot<AccountId>;
// 	type SessionInterface = Self;
// 	type EraPayout = pallet_staking::ConvertCurve<RewardCurve>;
// 	type NextNewSession = Session; //call pallet_session
// 	type MaxNominatorRewardedPerValidator = ConstU32<64>;
// 	type OffendingValidatorsThreshold = OffendingValidatorsThreshold;
// 	type ElectionProvider = ElectionProviderMultiPhase;
// 	type GenesisElectionProvider = onchain::OnChainExecution<OnChainSeqPhragmen>;
// 	type VoterList = VoterList;
// 	// This a placeholder, to be introduced in the next PR as an instance of bags-list
// 	type TargetList = pallet_staking::UseValidatorsMap<Self>;
// 	type MaxUnlockingChunks = ConstU32<32>;
// 	type HistoryDepth = ConstU32<84>;
// 	// type OnStakerSlash = (); // NominationPools
// 	type WeightInfo = pallet_staking::weights::SubstrateWeight<Runtime>;
// 	type BenchmarkingConfig = pallet_staking::TestBenchmarkingConfig;
// 	type EventListeners = ();
// }

// impl pallet_grandpa::Config for Test {
// 	type RuntimeEvent = RuntimeEvent;

// 	type WeightInfo = ();
// 	type MaxAuthorities = ConstU32<32>;
// 	type MaxSetIdSessionEntries = ConstU64<0>;

// 	type KeyOwnerProof = sp_core::Void;
// 	type EquivocationReportSystem = ();
// }
pub use sp_consensus_babe::AuthorityId;
use codec::{Decode, Encode};

// Build genesis storage according to the mock runtime.
pub fn new_test_ext(users: Vec<(u64, u64,u64)>) -> sp_io::TestExternalities {

	let mut t = frame_system::GenesisConfig::<Test>::default().build_storage().unwrap();
	let init_test_authority = vec![(1, 7), (2, 9), (3, 100)];
	let new_candidates = init_test_authority.clone();

	let mut new_next_authority: Vec<(AuthorityId, u64)> = vec![];

	// for (acc, w) in init_test_authority.into_iter(){
	// 	let acc_vec = acc.encode();
	// 	let mut bytes : [u8; 32] = [0u8; 32];
	// 	bytes.copy_from_slice(&acc_vec);
	// 	let who: AuthorityId = sp_core::sr25519::Public::from_raw(bytes).into();
	// 	new_next_authority.push((who, w));
	// }

	let next_config = BabeEpochConfiguration {
		c: (1, 4),
		allowed_slots: sp_consensus_babe::AllowedSlots::PrimarySlots,
	};

	// let authority = acc.iter().map(|x| sp_core::sr25519::Public::from_raw(x).into()).collect();
	// RuntimeGenesisConfig{
	// 	balances: BalancesConfig{
	// 		balances: users.iter().map(|(user,_, _)| (*user, 10)).collect()
	// 	},
	// 	babe: BabeConfig{
	// 		authorities: new_next_authority,
	// 		epoch_config: Some(BabeEpochConfiguration {
	// 			c: (1, 4),
	// 			allowed_slots: sp_consensus_babe::AllowedSlots::PrimarySlots,
	// 		})
	// 	}

	// 	..Default::default()
	// }
	// .assimilate_storage(&mut t).unwrap();


	// let mut ext = sp_io::TestExternalities::new(t);
	// ext.execute_with(|| System::set_block_number(1));
	// ext

	let acc_balance = new_candidates.clone();
	let balances: Vec<_> = (0..acc_balance.len()).map(|i| (i as u64, 10_000_000)).collect();
	let members = vec![1, 2, 3];
	pallet_balances::GenesisConfig::<Test> { balances }
		.assimilate_storage(&mut t)
		.unwrap();

		let keys: Vec<_> = NEXT_VALIDATORS
		.with(|l| l.borrow().iter().cloned().map(|i| (i, i, UintAuthorityId(i).into())).collect());
	
	BasicExternalities::execute_with_storage(&mut t, || {
		for (ref k, ..) in &keys {
			frame_system::Pallet::<Test>::inc_providers(k);
		}
		frame_system::Pallet::<Test>::inc_providers(&4);
		frame_system::Pallet::<Test>::inc_providers(&69);
	});

	// let validators = keys.iter().map(|x: &(u64, u64, _)| x.0).collect::<Vec<_>>();
	

	// stashes are the index.
	let session_keys: Vec<_> = new_candidates
		.iter()
		// .enumerate()
		.map(|(i, k)| 
		 		(*i, *i,  MockSessionKeys { dummy: UintAuthorityId(*k).into() }))

		.collect();

	// NOTE: this will initialize the babe authorities
	// through OneSessionHandler::on_genesis_session
	pallet_session::GenesisConfig::<Test> { keys: session_keys }
		.assimilate_storage(&mut t)
		.unwrap();


	
	

	pallet_template::GenesisConfig::<Test> { 
		// initial_notes: vec![],
		initial_members: members,
		initial_validators: keys.iter().map(|x| x.0).collect::<Vec<_>>(),
		
		}
		.assimilate_storage(&mut t)
		.unwrap();
	
	t.into()
}
