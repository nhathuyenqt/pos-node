//! # Validator Set Pallet
//!
//! The Validator Set Pallet allows addition and removal of
//! authorities/validators via extrinsics (transaction calls), in
//! Substrate-based PoA networks. It also integrates with the im-online pallet
//! to automatically remove offline validators.
//!
//! The pallet depends on the Session pallet and implements related traits for session
//! management. Currently it uses periodic session rotation provided by the
//! session pallet to automatically rotate sessions. For this reason, the
//! validator addition and removal becomes effective only after 2 sessions
//! (queuing + applying).

#![cfg_attr(not(feature = "std"), no_std)]

mod benchmarking;
#[cfg(test)]
mod mock;

#[cfg(test)]
mod tests;

pub mod weights;

use core::mem;

use frame_system::pallet_prelude::*;
use frame_support::{
	ensure,
	pallet_prelude::*,
	traits::{
		EstimateNextSessionRotation, Get, ValidatorSet, ValidatorSetWithIdentification,
		tokens::Balance,
		fungibles::{Create, Destroy, Inspect, Mutate}
	},
	DefaultNoBound,
};
use log;
pub use pallet::*;
// use pallet_babe::Committee;
use sp_runtime::traits::{Convert, Zero, Member};
use sp_staking::offence::{Offence, OffenceError, ReportOffence};
use sp_std::prelude::*;
pub use weights::*;

// use sp_consensus_babe::AuthorityId as BabeId;
// use sp_consensus_grandpa::AuthorityId as GrandpaId;

pub const LOG_TARGET: &'static str = "runtime::committee";

#[frame_support::pallet()]
pub mod pallet {
	use core::ops::Bound;

	use sp_runtime::FixedPointOperand;

use super::*;

	/// Configure the pallet by specifying the parameters and types on which it
	/// depends.
	#[pallet::config]
	pub trait Config: frame_system::Config + pallet_session::Config {
		/// The Event type.
		type RuntimeEvent: From<Event<Self>> + IsType<<Self as frame_system::Config>::RuntimeEvent>;

		/// Origin for adding or removing a validator.
		type AddRemoveOrigin: EnsureOrigin<Self::RuntimeOrigin>;

		/// Minimum number of validators to leave in the validator set during
		/// auto removal.
		type MinAuthorities: Get<u32>;

		/// Information on runtime weights.
		type WeightInfo: WeightInfo;

		type XIndexBalance: Balance
			+ FixedPointOperand + MaxEncodedLen + MaybeSerializeDeserialize + TypeInfo;
		type XIndex: Inspect<Self::AccountId, AssetId = u32, Balance = Self::XIndexBalance> 
			+ Mutate<Self::AccountId> + Create<Self::AccountId> + Destroy<Self::AccountId>;

		// type AccountIdToValidatorId: Convert<Self::AccountId, Option<Self::ValidatorId>>;
		// type ValidatorId: Member + Parameter + MaybeSerializeDeserialize + MaxEncodedLen + TryFrom<Self::AccountId>;
		// type ValidatorIdOf: Convert<Self::AccountId, Option<Self::ValidatorId>>;
	}

	#[pallet::pallet]
	#[pallet::without_storage_info]
	pub struct Pallet<T>(_);

	#[derive(Clone, Encode, Decode, PartialEq, RuntimeDebug, TypeInfo, MaxEncodedLen)]
	#[scale_info(skip_type_params(T))]
	pub struct Profile<T: Config>{
		pub id: [u8; 16],
		pub reputation: u32,
		pub addr: T::AccountId,
		pub note: BoundedVec<u8, ConstU32<1000>>
	}

	#[pallet::storage]
	#[pallet::getter(fn committee)]
	pub type Committee<T: Config> = StorageValue<_, Vec<(T::AccountId, u64)>, ValueQuery>;

	#[pallet::storage]
	#[pallet::getter(fn notes)]
	pub type Notes<T: Config> = StorageMap<_, Twox64Concat, T::AccountId, BoundedVec<u8, ConstU32<1000>>>;

	// #[pallet::storage]
	// #[pallet::getter(fn nextcommittee)]
	// pub type NextCommittee<T: Config> = StorageMap<_, T::AccountId, u64>;

	#[pallet::storage]
	#[pallet::getter(fn candidates)]
	pub type Candidates<T: Config> = StorageMap<_, Twox64Concat, [u8; 16], Profile<T>>;


    #[pallet::storage]
    #[pallet::getter(fn validators)]
    pub type Validators<T: Config> = StorageValue<_, Vec<T::ValidatorId>, ValueQuery>;

    #[pallet::storage]
    #[pallet::getter(fn approved_validators)]
    pub type ApprovedValidators<T: Config> = StorageValue<_, Vec<T::AccountId>, ValueQuery>;

    #[pallet::storage]
    #[pallet::getter(fn validators_to_remove)]
    pub type OfflineValidators<T: Config> = StorageValue<_, Vec<T::AccountId>, ValueQuery>;

	#[pallet::event]
	#[pallet::generate_deposit(pub(super) fn deposit_event)]
	pub enum Event<T: Config> {
		/// New validator addition initiated. Effective in ~2 sessions.
		ValidatorAdditionInitiated(T::AccountId),

        /// Validator removal initiated. Effective in ~2 sessions.
        ValidatorRemovalInitiated(T::AccountId),
		NewProfile(T::AccountId, [u8; 16])
	}

	
	// Errors inform users that something went wrong.
	#[pallet::error]
	pub enum Error<T> {
		/// Target (post-removal) validator count is below the minimum.
		TooLowValidatorCount,
		/// Validator is already in the validator set.
		Duplicate,
		TooLong
	}

	#[pallet::hooks]
	impl<T: Config> Hooks<BlockNumberFor<T>> for Pallet<T> {}

	#[pallet::genesis_config]
	// #[derive(DefaultNoBound)]
	pub struct GenesisConfig<T: Config> {
		pub initial_validators: Vec<T::ValidatorId>,
		pub initial_committee: Vec<T::AccountId>,
	}

	impl<T: Config> Default for GenesisConfig<T>{
		fn default() -> Self{
			Self{
				initial_committee: Default::default(),
				initial_validators: Default::default(),

			}
		}
	}

	#[pallet::genesis_build]
	impl<T: Config> BuildGenesisConfig for GenesisConfig<T> {
		fn build(&self) {
			Pallet::<T>::initialize_validators(&self.initial_validators);
		}
	}

	#[pallet::call]
	impl<T: Config> Pallet<T> {

		#[pallet::weight(0)]
        pub fn register_validator(origin: OriginFor<T>) -> DispatchResult {
            let sender = ensure_signed(origin)?;

            // Self::do_add_validator2(sender)?;

            Ok(())
        }
		/// Add a new validator.
		///
		/// New validator's session keys should be set in Session pallet before
		/// calling this.
		///
		/// The origin can be configured using the `AddRemoveOrigin` type in the
		/// host runtime. Can also be set to sudo/root.
		/*#[pallet::call_index(0)]
		#[pallet::weight(0)]
        pub fn add_validator(origin: OriginFor<T>, validator_id: T::ValidatorId) -> DispatchResult {
            T::AddRemoveOrigin::ensure_origin(origin)?;

            Self::do_add_validator(validator_id.clone())?;
            Self::approve_validator(validator_id)?;

            Ok(())
        }*/

		/// Remove a validator.
		///
		/// The origin can be configured using the `AddRemoveOrigin` type in the
		/// host runtime. Can also be set to sudo/root.

		#[pallet::weight(<T as pallet::Config>::WeightInfo::remove_validator())]
		pub fn remove_validator(
			origin: OriginFor<T>,
			validator_id: T::ValidatorId,
		) -> DispatchResult {
			T::AddRemoveOrigin::ensure_origin(origin)?;

			Self::do_remove_validator(validator_id.clone())?;

			Ok(())
		}

		#[pallet::weight(<T as pallet::Config>::WeightInfo::add_validator())]
		pub fn create_profile(origin: OriginFor<T>, note: Vec<u8>) -> DispatchResult {

			let sender = ensure_signed(origin)?;
			let bounded_note : BoundedVec<_, _> = note.try_into().map_err(|_| Error::<T>::TooLong)?;
			Self::do_add_profile(&sender, bounded_note)?;

			Ok(())
		}
	}


}

impl<T: Config> Pallet<T> {
	fn initialize_validators(validators: &[T::ValidatorId]) {
		assert!(
			validators.len() as u32 >= T::MinAuthorities::get(),
			"Initial set of validators must be at least T::MinAuthorities"
		);
		assert!(<Validators<T>>::get().is_empty(), "Validators are already initialized!");

		<Validators<T>>::put(validators);
	}

	fn do_add_validator(validator_id: T::ValidatorId) -> DispatchResult {
		ensure!(!<Validators<T>>::get().contains(&validator_id), Error::<T>::Duplicate);
		<Validators<T>>::mutate(|v| v.push(validator_id.clone()));

		Self::deposit_event(Event::ValidatorAdditionInitiated(validator_id.clone()));
		log::debug!(target: LOG_TARGET, "Validator addition initiated.");

		Ok(())
	}

	fn do_remove_validator(validator_id: T::ValidatorId) -> DispatchResult {
		let mut validators = <Validators<T>>::get();

		// Ensuring that the post removal, target validator count doesn't go
		// below the minimum.
		ensure!(
			validators.len().saturating_sub(1) as u32 >= T::MinAuthorities::get(),
			Error::<T>::TooLowValidatorCount
		);

		validators.retain(|v| *v != validator_id);

		<Validators<T>>::put(validators);

		Self::deposit_event(Event::ValidatorRemovalInitiated(validator_id.clone()));
		log::debug!(target: LOG_TARGET, "Validator removal initiated.");

		Ok(())
	}

	Adds offline validators to a local cache for removal on new session.
	fn mark_for_removal(validator_id: T::ValidatorId) {
        <OfflineValidators<T>>::mutate(|v| v.push(validator_id));
        log::debug!(target: LOG_TARGET, "Offline validator marked for auto removal.");
    }

	// Removes offline validators from the validator set and clears the offline
	// cache. It is called in the session change hook and removes the validators
	// who were reported offline during the session that is ending. We do not
	// check for `MinAuthorities` here, because the offline validators will not
	// produce blocks and will have the same overall effect on the runtime.
	fn remove_offline_validators() {
		let validators_to_remove = <OfflineValidators<T>>::get();

		// Delete from active validator set.
		<Validators<T>>::mutate(|vs| vs.retain(|v| !validators_to_remove.contains(v)));
		log::debug!(
			target: LOG_TARGET,
			"Initiated removal of {:?} offline validators.",
			validators_to_remove.len()
		);

		// Clear the offline validator list to avoid repeated deletion.
		<OfflineValidators<T>>::put(Vec::<T::ValidatorId>::new());
	}


	fn do_add_profile(member: &T::AccountId, note : BoundedVec<u8, ConstU32<1000>>) -> 	DispatchResult{

		let encoded_payload = (member.clone(), note.clone()).encode();
		let id = frame_support::Hashable::blake2_128(&encoded_payload);
		let profile = Profile::<T>{id: id, reputation: 0u32, addr: member.clone(), note: note.clone()};
		Candidates::<T>::insert(profile.id, profile);
		Notes::<T>::insert(member.clone(), note);
		Self::deposit_event(Event::NewProfile(member.clone(), id.clone()));
		Ok(())
	}	
	// fn session_keys(
	// 	babe: BabeId,
	// 	grandpa: GrandpaId,
	// ) -> SessionKeys {
	// 	SessionKeys { babe, grandpa }
	// }
	// fn do_add_validator_from_account(newMems: Vec<T::AccountId>){
	// 	// like in pallet sessionconfig in chain_specs
	// 	let keys = newMems
	// 			.iter()
	// 			.map(|x| {
	// 				(
	// 					x.0.clone(),
	// 					x.0.clone(),
	// 					self.session_keys(x.2.clone(), x.3.clone()),
	// 				)
	// 			})
	// 			.collect::<Vec<_>>();

	// }

	fn do_add_validator2(newMems: T::ValidatorId) -> DispatchResult {
		// like in pallet sessionconfig in chain_specs
		// let keys = newMems
		// 		.iter()
		// 		.map(|x| {
		// 			(
		// 				x.0.clone(),
		// 				x.0.clone(),
		// 				self.session_keys(x.2.clone(), x.3.clone()),
		// 			)
		// 		})
		// 		.collect::<Vec<_>>();
		Ok(())

	}

	

	// fn get_new_validator_list() -> Vec<(T::AccountId, u64)>{
	// 	// const maxA : u32 = T::MaxAuthorities::get();
	// 	let mut acc_vec: Vec<(T::AccountId, u64)> = vec![];
	// 	for (account_id, boardInfo) in ScoreBoard::<T>::iter(){
		
	// 		acc_vec.push((account_id, boardInfo.score.clone().saturated_into::<u64>()));
	// 	}
		
	// 	// let mut my_vec: Vec<(T::AccountId, u64)> = Default::default();
	// 	// let bounded_authorities =
	// 	// 	Vec::<_, T::MaxAuthorities>::try_from(my_vec.to_vec())
	// 	// 		.expect("Initial number of authorities should be lower than T::MaxAuthorities");
	// 	return acc_vec;
	// 	// return my_vec;
	// }
	
}

// Provides the new set of validators to the session module when session is
// being rotated.
impl<T: Config> pallet_session::SessionManager<T::ValidatorId> for Pallet<T> {
	// Plan a new session and provide new validator set.
	fn new_session(_new_index: u32) -> Option<Vec<T::ValidatorId>> {
		// Remove any offline validators. This will only work when the runtime
		// also has the im-online pallet.
		// Self::remove_offline_validators();

		

		// log::debug!(target: LOG_TARGET, "New session called; updated validator set provided.");

		// Some(Self::validators())
		let x: Vec<(T::AccountId, u64)> = Self::committee();
		let mut y: Vec<T::ValidatorId> = vec![];

		// for (account_id, w) in x.into_iter(){
		// 	let a = ValidatorOf::<T>::convert(account_id).unwrap();
		// 	y.push(a);
			
		// }
		
		Some(y)
		
	}

	fn end_session(_end_index: u32) {}

	fn start_session(_start_index: u32) {}
}

impl<T: Config> EstimateNextSessionRotation<BlockNumberFor<T>> for Pallet<T> {
	fn average_session_length() -> BlockNumberFor<T> {
		Zero::zero()
	}

	fn estimate_current_session_progress(
		_now: BlockNumberFor<T>,
	) -> (Option<sp_runtime::Permill>, sp_weights::Weight) {
		(None, Zero::zero())
	}

	fn estimate_next_session_rotation(
		_now: BlockNumberFor<T>,
	) -> (Option<BlockNumberFor<T>>, sp_weights::Weight) {
		(None, Zero::zero())
	}
}

// Implementation of Convert trait to satisfy trait bounds in session pallet.
// // Here it just returns the same ValidatorId.
pub struct ValidatorOf<T>(sp_std::marker::PhantomData<T>);

impl<T: Config> Convert<T::ValidatorId, Option<T::ValidatorId>> for ValidatorOf<T> {
	fn convert(account: T::ValidatorId) -> Option<T::ValidatorId> {
		Some(account)
	}
}


impl<T: Config> ValidatorSet<T::AccountId> for Pallet<T> {
    type ValidatorId = T::ValidatorId;
    type ValidatorIdOf = T::ValidatorIdOf;

    fn session_index() -> sp_staking::SessionIndex {
        pallet_session::Pallet::<T>::current_index()
    }

    fn validators() -> Vec<Self::ValidatorId> {
        pallet_session::Pallet::<T>::validators()
    }
}

impl<T: Config> ValidatorSetWithIdentification<T::AccountId> for Pallet<T> {
    type Identification = T::ValidatorId;
    type IdentificationOf = ValidatorOf<T>;
}

// Offence reporting and unresponsiveness management.
impl<T: Config, O: Offence<(T::AccountId, T::AccountId)>>
ReportOffence<T::AccountId, (T::AccountId, T::AccountId), O> for Pallet<T>
{
    fn report_offence(_reporters: Vec<T::AccountId>, offence: O) -> Result<(), OffenceError> {
        let offenders = offence.offenders();

        for (v, _) in offenders.into_iter() {
            Self::mark_for_removal(v);
        }

        Ok(())
    }

    fn is_known_offence(
        _offenders: &[(T::AccountId, T::AccountId)],
        _time_slot: &O::TimeSlot,
    ) -> bool {
        false
    }
}




// impl<T: Config> pallet_session::SessionManager<T::ValidatorId> for Pallet<T> {
// 	// Plan a new session and provide new validator set.
// 	fn new_session(_new_index: u32) -> Option<Vec<T::ValidatorId>> {
// 		// Remove any offline validators. This will only work when the runtime
// 		// also has the im-online pallet.
// 		Self::remove_offline_validators();

// 		log::debug!(target: LOG_TARGET, "New session called; updated validator set provided.");

// 		Some(Self::validators())
// 	}

// 	fn end_session(_end_index: u32) {}

// 	fn start_session(_start_index: u32) {}
// }

impl<T: Config> pallet_babe::Committee<<T as frame_system::Config >::AccountId> for Pallet<T>{
	/// Trigger an epoch change, if any should take place. This should be called
	/// during every block, after initialization is done.
	fn get_new_committee() -> Vec<(T::AccountId, u64)>{
		
		// let mut acc_vec: Vec<(T::AccountId, u64)> = vec![];
		// for (account_id, boardInfo) in Committee::<T>::iter(){
		
		// 	// acc_vec.push((account_id, boardInfo.score.clone().saturated_into::<u64>()));
		// }

		let acc_vec: Vec<(T::AccountId, u64)> = Self::committee();
		

		return acc_vec;
		// return my_vec;
	}
}