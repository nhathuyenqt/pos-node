#![cfg_attr(not(feature = "std"), no_std)]

/// Edit this file to define custom logic or remove it if it is not needed.
/// Learn more about FRAME and the core library of Substrate FRAME pallets:
/// <https://docs.substrate.io/reference/frame-pallets/>
pub use pallet::*;
<<<<<<< HEAD
=======
use sp_std::prelude::*;
>>>>>>> 9671047a89e4df39bd788c00a2961463d5feb263

#[cfg(test)]
mod mock;

#[cfg(test)]
mod tests;

#[cfg(feature = "runtime-benchmarks")]
mod benchmarking;
<<<<<<< HEAD

#[frame_support::pallet]
pub mod pallet {
	use frame_support::pallet_prelude::*;
	use frame_system::pallet_prelude::*;

	#[pallet::pallet]
=======
pub mod weights;
pub use weights::*;



#[frame_support::pallet]
pub mod pallet {
	use super::*;
	use frame_support::pallet_prelude::*;
	use frame_system::pallet_prelude::*;
	

	#[pallet::pallet]
	// #[pallet::generate_store(pub(super) trait Store)]
	#[pallet::without_storage_info]
>>>>>>> 9671047a89e4df39bd788c00a2961463d5feb263
	pub struct Pallet<T>(_);

	/// Configure the pallet by specifying the parameters and types on which it depends.
	#[pallet::config]
	pub trait Config: frame_system::Config {
		/// Because this pallet emits events, it depends on the runtime's definition of an event.
		type RuntimeEvent: From<Event<Self>> + IsType<<Self as frame_system::Config>::RuntimeEvent>;
<<<<<<< HEAD
	}

	// The pallet's runtime storage items.
	// https://docs.substrate.io/main-docs/build/runtime-storage/
	#[pallet::storage]
	#[pallet::getter(fn something)]
	// Learn more about declaring storage items:
	// https://docs.substrate.io/main-docs/build/runtime-storage/#declaring-storage-items
	pub type Something<T> = StorageValue<_, u32>;
=======
		/// Type representing the weight of this pallet
		type WeightInfo: WeightInfo;
	}

	
	// //add 8 Nov 2023
	#[derive(Encode, Decode, Clone, PartialEq,  TypeInfo, MaxEncodedLen)]
	pub struct UserInfo{
		pub username: [u8; 256],
		pub id: i64,
		//About me
		pub about_me: [u8; 256]
	}


	// The pallet's runtime storage items.
	// https://docs.substrate.io/main-docs/build/runtime-storage/
	
	#[pallet::storage]
	#[pallet::getter(fn info)]
		pub type AccountToUserInfo<T: Config> = StorageMap<_, Blake2_128Concat, T::AccountId, UserInfo, OptionQuery>;

	// Learn more about declaring storage items:
	// https://docs.substrate.io/main-docs/build/runtime-storage/#declaring-storage-items
	// pub type Something<T> = StorageValue<_, u32>;
>>>>>>> 9671047a89e4df39bd788c00a2961463d5feb263

	// Pallets use events to inform users when important changes are made.
	// https://docs.substrate.io/main-docs/build/events-errors/
	#[pallet::event]
	#[pallet::generate_deposit(pub(super) fn deposit_event)]
	pub enum Event<T: Config> {
		/// Event documentation should end with an array that provides descriptive names for event
		/// parameters. [something, who]
<<<<<<< HEAD
		SomethingStored { something: u32, who: T::AccountId },
=======
		UserCreated { user: T::AccountId}
>>>>>>> 9671047a89e4df39bd788c00a2961463d5feb263
	}

	// Errors inform users that something went wrong.
	#[pallet::error]
	pub enum Error<T> {
		/// Error names should be descriptive.
		NoneValue,
		/// Errors should have helpful documentation associated with them.
		StorageOverflow,
	}

	// Dispatchable functions allows users to interact with the pallet and invoke state changes.
	// These functions materialize as "extrinsics", which are often compared to transactions.
	// Dispatchable functions must be annotated with a weight and must return a DispatchResult.
	#[pallet::call]
	impl<T: Config> Pallet<T> {
		/// An example dispatchable that takes a singles value as a parameter, writes the value to
		/// storage and emits an event. This function must be dispatched by a signed extrinsic.
		#[pallet::call_index(0)]
<<<<<<< HEAD
		#[pallet::weight(10_000 + T::DbWeight::get().writes(1).ref_time())]
=======
		#[pallet::weight(T::WeightInfo::do_something())]
>>>>>>> 9671047a89e4df39bd788c00a2961463d5feb263
		pub fn do_something(origin: OriginFor<T>, something: u32) -> DispatchResult {
			// Check that the extrinsic was signed and get the signer.
			// This function will return an error if the extrinsic is not signed.
			// https://docs.substrate.io/main-docs/build/origins/
			let who = ensure_signed(origin)?;

			// Update storage.
<<<<<<< HEAD
			<Something<T>>::put(something);

			// Emit an event.
			Self::deposit_event(Event::SomethingStored { something, who });
=======
			// <Something<T>>::put(something);

			// Emit an event. add by X
			// Self::deposit_event(Event::SomethingStored { something, who });
>>>>>>> 9671047a89e4df39bd788c00a2961463d5feb263
			// Return a successful DispatchResultWithPostInfo
			Ok(())
		}

		/// An example dispatchable that may throw a custom error.
<<<<<<< HEAD
		#[pallet::call_index(1)]
		#[pallet::weight(10_000 + T::DbWeight::get().reads_writes(1,1).ref_time())]
		pub fn cause_error(origin: OriginFor<T>) -> DispatchResult {
			let _who = ensure_signed(origin)?;

			// Read a value from storage.
			match <Something<T>>::get() {
				// Return an error if the value has not been set.
				None => return Err(Error::<T>::NoneValue.into()),
				Some(old) => {
					// Increment the value read from storage; will error in the event of overflow.
					let new = old.checked_add(1).ok_or(Error::<T>::StorageOverflow)?;
					// Update the value in storage with the incremented result.
					<Something<T>>::put(new);
					Ok(())
				},
			}
		}
=======
		// #[pallet::call_index(1)]
		// #[pallet::weight(T::WeightInfo::cause_error())]
		// pub fn cause_error(origin: OriginFor<T>) -> DispatchResult {
		// 	let _who = ensure_signed(origin)?;

		// 	// Read a value from storage.
		// 	match <Something<T>>::get() {
		// 		// Return an error if the value has not been set.
		// 		None => return Err(Error::<T>::NoneValue.into()),
		// 		Some(old) => {
		// 			// Increment the value read from storage; will error in the event of overflow.
		// 			let new = old.checked_add(1).ok_or(Error::<T>::StorageOverflow)?;
		// 			// Update the value in storage with the incremented result.
		// 			<Something<T>>::put(new);
		// 			Ok(())
		// 		},
		// 	}
		// }
		#[pallet::call_index(1)]
		#[pallet::weight(10_000 + T::DbWeight::get().writes(1).ref_time())]
		pub fn register_user(origin: OriginFor<T>, username: [u8; 256], id: i64, about_me: [u8; 256]) -> DispatchResult{
			let sender = ensure_signed(origin)?;
			//Define new user
			let new_user = UserInfo{ username, id, about_me};
			// change state of storage mapping by adding user
			<AccountToUserInfo<T>>::insert(&sender, new_user);
			//emit an event
			Self::deposit_event(Event::<T>::UserCreated{user: sender});
			Ok(())
		}

>>>>>>> 9671047a89e4df39bd788c00a2961463d5feb263
	}
}
