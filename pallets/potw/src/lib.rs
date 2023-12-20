#![cfg_attr(not(feature = "std"), no_std)]

// pub mod mock;
// mod tests;

use frame_support::{
	ensure,
	pallet_prelude::*,
	traits::{
		Currency, 
		tokens::{Balance}
	}
};
use frame_system::pallet_prelude::BlockNumberFor;

use log;
pub use pallet::*;

pub mod weights;
pub use weights::*;

use codec::{Decode, Encode, alloc::string::ToString};
use sp_runtime::{
	traits::{Convert, Verify, Zero, SaturatedConversion},
	// FixedPointOperand
};
use sp_staking::offence::{Offence, OffenceError, ReportOffence};
use rstd::{collections::btree_set::BTreeSet, prelude::*, marker::PhantomData};

pub const LOG_TARGET: &'static str = "pallet::proof-of-tangible-work";


// #[cfg(feature="std")]
// use frame_support::serde::{Deserialize, Serialize};



#[frame_support::pallet]
pub mod pallet {
	use super::*;
	
	use frame_system::pallet_prelude::*;
	use frame_support::traits::Currency;

	#[pallet::pallet]
	#[pallet::generate_store(pub(super) trait Store)]
	pub struct Pallet<T>(_);

	//#[cfg(feature = "std")]
	type AccountIdOf<T> = <T as frame_system::Config>::AccountId;
	type TokenBalanceOf<T> = <<T as Config>::Token as Currency<<T as frame_system::Config>::AccountId>>::Balance;
	type TaskCreditBalanceOf<T> = <<T as Config>::TaskCredit as Currency<<T as frame_system::Config>::AccountId>>::Balance;
	type ScoreBalanceOf<T> = <<T as Config>::Score as Currency<<T as frame_system::Config>::AccountId>>::Balance;
	type BoardInfoOf<T> = BoardInfo<T>;



	#[derive(Clone, Encode, Decode, PartialEq, RuntimeDebug, TypeInfo, MaxEncodedLen)]
	#[scale_info(skip_type_params(T))]
	pub struct Task<T:Config>{
		pub id: [u8; 16],
		pub credit: Option<TaskCreditBalanceOf<T>>,
		pub state: u8,
		pub owner: T::AccountId,
		pub description: BoundedVec<u8, ConstU32<100>>,
		// pub deadline
	}

	// #[derive(Clone, Encode, Decode, PartialEq, RuntimeDebug, TypeInfo, MaxEncodedLen)]
	// // #[cfg_attr(feature="std", derive(Serialize, Deserialize))]
	// pub enum State{
	// 	Active,
	// 	Closed
	// }

	// #[derive(PartialEq, Eq, Clone, Encode, Decode)]
	// pub enum CustomBalance<T, S, C>
	// where 
	// 	T: HasCompact,
	// 	S: HasCompact,
	// {
	// 	Token(T),
	// 	Score(S),
	// 	Credit(C)
	// }
	// #[cfg_attr(feature="std", derive(Debug))]
	#[derive(Encode, Decode, Default, PartialEq, Eq, TypeInfo, MaxEncodedLen)]
	#[scale_info(skip_type_params(T))]
	pub struct BoardInfo<T: Config> {
		member : T::AccountId,
		token : TokenBalanceOf<T>,
		score : ScoreBalanceOf<T>,
		taskcredit : TaskCreditBalanceOf<T>,
	}
	
	// type BoardInfoOf
	
	pub trait TokenManagement<T:Config> {
		// fn convert_to_taskCredit(from: T::&AccountId, amount: TokenBalanceOf<T>) -> DispatchResult;
		fn withdraw(owner: T::AccountId, amount: TokenBalanceOf<T>) -> DispatchResult;
		// fn transfer(from: &T::AccountId, to: &AccountId, value: TokenBalanceOf<T>) -> DispatchResult;
		fn deposit(owner: T::AccountId, amount: u32) -> DispatchResult;
	}

	pub trait TaskCreditDeposit<T:Config> {
		fn deposit(owner: T::AccountId, amount: u32) -> DispatchResult;
		// fn lock(owner: T::AccountId, amount: TaskCreditBalanceOf<T>) -> DispatchResult;
		// fn burn(owner: T::AccountId, amount: TaskCreditBalanceOf<T>) -> DispatchResult;
	}

	// pub trait ScoreManagement<T:Config> {
	// 	fn withdraw(owner: T::AccountId, amount: u32) -> DispatchResult;
	// }

	// pub trait TokenTest<T:Config>: Currency<T::AccountId> {
	// 	// fn transfer(source: &AccountId, dest: &AccountId, value Self)
	// }


	
	
	// pub trait TokenManage<T:Config>{
	// 	type Balance: Parameter
	// 		+ Member
	// 		+ AtLeast32BitUnsigned
	// 		+ Codec
	// 		+ Default
	// 		+ Copy
	// 		+ MaybeSerializeDeserialize
	// 		// + Debug
	// 		+ MaxEncodedLen
	// 		+ TypeInfo
	// 		+ FixedPointOperand;
	// 	fn withdraw(owner: T::AccountId, amount: Self::Balance);
	// }



	#[pallet::config]
	pub trait Config: frame_system::Config + pallet_transaction_payment::Config {
		/// Because this pallet emits events, it depends on the runtime's definition of an event.
		type RuntimeEvent: From<Event<Self>> + IsType<<Self as frame_system::Config>::RuntimeEvent>;

		type Token : Currency<Self::AccountId>;
	
		type Score : Currency<Self::AccountId>;
	    type TaskCredit : Currency<Self::AccountId>;

		// type ScoreBalance: Balance + FixedPointOperand + MaxEncodedLen + MaybeSerializeDeserialize + TypeInfo;
		// type ScoreNow: Get<ScoreBalanceOf<Self>>;
		// type Score2 : Balance<Self::AccountId>;
		// type TokenToTaskCredit: TokenToTaskCredit<Self>;
		// type TaskCreditDeposit: TaskCreditDeposit<Self>;
		// type TokenManagement: TokenManagement<Self>;
		// type ScoreManagement: ScoreManagement<Self>;
		// type State: State;
		// type TokenBalanceOf<T> = <<T as Config>::Token as Currency<<T as frame_systemSelf::AccountId<T>>>::Balance;
		// type ScoreBalanceOf<T> =  <<T as Config>::Score as Currency<Self::AccountId<T>>>::Balance;
		// type TaskCreditBalanceOf<T> = <TaskCredit<T> as Currency<Self::AccountId>>::Balance;
		// type TaskCredit: Currency<Self::AccountId> + ReservableCurrency<Self::AccountId>;

		// type CommonBalance<T> : Tuple(TokenBalanceOf<T>, ScoreBalanceOf<T>, TaskCreditBalanceOf<T>);

		type TokenToTaskCredit: Convert<TokenBalanceOf<Self>, TaskCreditBalanceOf<Self>>; // rate??
		// type LockCreditToTask;
		// type RewardTaskWinner: pallet_transaction_payment;


		// #[pallet::constant]
		// type MaxLength: Get<u32>;
		/// Type representing the weight of this pallet
		type WeightInfo: WeightInfo;

		
	}

	
	// The pallet's runtime storage items.
	// https://docs.substrate.io/main-docs/build/runtime-storage/
	#[pallet::storage]
	#[pallet::getter(fn gettasks)]
	pub(super) type Tasks<T: Config> = StorageMap<_, Twox64Concat, [u8; 16], Task<T>>;

	#[pallet::storage]
	#[pallet::getter(fn getscoreboard)]
	pub(super) type ScoreBoard<T: Config> = StorageMap
		<_, Twox64Concat, T::AccountId, BoardInfo<T>>;



	// Pallets use events to inform users when important changes are made.
	// https://docs.substrate.io/main-docs/build/events-errors/
	#[pallet::event]
	#[pallet::generate_deposit(pub(super) fn deposit_event)]
	pub enum Event<T: Config> {
		/// Event documentation should end with an array that provides descriptive names for event
		/// parameters. [something, who]
		
		NewTaskCreated{owner: T::AccountId, credit: Option<TaskCreditBalanceOf<T>>},
		DepositCredit{owner: T::AccountId, amt: Option<TaskCreditBalanceOf<T>>},
		TaskWinnerReward{task_id: u32, winner: T::AccountId},
		WithdrawScoreToToken{owner: T::AccountId, amt: Option<ScoreBalanceOf<T>>},

	}

	// Errors inform users , that something went wrong.
	#[pallet::error]
	pub enum Error<T> {
		/// Error names should be descriptive.
		NoneValue,
		/// Errors should have helpful documentation associated with them.
		StorageOverflow,
		DuplicateMember,

	}

	
	#[pallet::genesis_config]
	pub struct GenesisConfig<T: Config> {
		pub initial_tasks: Vec<(T::AccountId, [u8; 16], u8)>,
		pub initial_members: Vec<(T::AccountId, TokenBalanceOf<T>)>,
	}

	impl<T: Config> Default for GenesisConfig<T> {
		fn default() -> Self {
			Self { 
                initial_tasks: Default::default(),
				initial_members: Default::default(),
            }
		}
	}

	#[pallet::genesis_build]
	impl<T: Config> BuildGenesisConfig for GenesisConfig<T> {
		fn build(&self) {
			for (task, _state, _account) in &self.initial_tasks {
				            // assert!(Pallet::<T>::create_task().is_ok());
			    
			}
			for (_account, _balance) in &self.initial_members{
				<Pallet<T>>::do_register_for_scoreboard(_account, _balance.clone());
			}

		}
	}


	// Dispatchable functions allows users to interact with the pallet and invoke state changes.
	// These functions materialize as "extrinsics", which are often compared to transactions.
	// Dispatchable functions must be annotated with a weight and must return a DispatchResult.
	#[pallet::call]
	impl<T: Config> Pallet<T> {
		/// An example dispatchable that takes a singles value as a parameter, writes the value to
		/// storage and emits an event. This function must be dispatched by a signed extrinsic.

		#[pallet::weight(T::WeightInfo::do_something())]
		pub fn do_something(origin: OriginFor<T>, something: u32) -> DispatchResult {
			// Check that the extrinsic was signed and get the signer.
			// This function will return an error if the extrinsic is not signed.
			// https://docs.substrate.io/main-docs/build/origins/
			let who = ensure_signed(origin)?;
 
			// Update storage.
			// <Something<T>>::put(something);

			// Emit an event.
			// Self::deposit_event(Event::SomethingStored { something, who });
			// Return a successful DispatchResultWithPostInfo
			Ok(())
		}

		/// An example dispatchable that may throw a custom error.
		#[pallet::weight(T::WeightInfo::cause_error())]
		pub fn register_for_scoreboard(
			origin: OriginFor<T>, amt: TokenBalanceOf<T>
		) -> DispatchResult {
			let sender = ensure_signed(origin)?;

			Self::do_register_for_scoreboard(&sender, amt);
			Ok(())
		}


		#[pallet::weight(T::WeightInfo::cause_error())]
		pub fn create_task(
			origin: OriginFor<T>, 
			//peer_address: OpaquePeerId,
			// signature: T::Signature,
			task_description: BoundedVec<u8, ConstU32<100>>,
			// timestamp: u64,
		) -> DispatchResult {
			let sender = ensure_signed(origin)?;

			Self::do_create_task(&sender, task_description)?;
			Ok(())
		}

		

		#[pallet::weight(T::WeightInfo::cause_error())]
		pub fn convert_token_to_taskcredit(origin: OriginFor<T>, amt: TokenBalanceOf<T>)-> DispatchResult {
			let sender = ensure_signed(origin)?;
			
			Self::do_convert_tokens_to_credits(sender, amt)?;
			Ok(())
		}


		#[pallet::weight(T::WeightInfo::cause_error())]
		pub fn convert_score_to_token(origin: OriginFor<T>, amt: TokenBalanceOf<T>)-> DispatchResult {
			let sender = ensure_signed(origin)?;
			let _ = T::TokenToTaskCredit::convert(amt);
			Self::do_convert_score_to_token(sender, 100u32)?;
			Ok(())
		}
		
		#[pallet::weight(T::WeightInfo::cause_error())]
		pub fn reward_winner(origin: OriginFor<T>, amt: u32, winner: T::AccountId)-> DispatchResult {
			log::info!(target: LOG_TARGET, "rewarding the winners");
			let sender = ensure_signed(origin)?;
			T::TaskCredit::burn(amt.into());
			Self::do_reward_winner(winner, 100u32);
			Ok(())
		}
		
		#[pallet::weight(T::WeightInfo::cause_error())]
		pub fn get_score_balance(origin: OriginFor<T>)-> DispatchResult {
			let sender = ensure_signed(origin)?;
			// let score = T::ScoreNow::get();
			Ok(())
		}
		

	}

	impl<T: Config> Pallet<T> { 
		fn do_register_for_scoreboard(member: &T::AccountId, balance: TokenBalanceOf<T>)-> DispatchResult{
			let newSB = BoardInfo::<T> {member: member.clone(), token: balance.clone(), score: 0u32.into(), taskcredit : 0u32.into()};

			ensure!(!ScoreBoard::<T>::contains_key(&member), Error::<T>::DuplicateMember);

			ScoreBoard::<T>::insert(&member, newSB);
			Ok(())
		} 
		fn do_create_task(
			owner: &T::AccountId,
			task_description: BoundedVec<u8, ConstU32<100>>,
			// credit: Option<TaskCreditBalanceOf<T>>
		) -> DispatchResult{
		
			//let id = frame_support::Hashable::blake2_128(&(owner, &credit.unwrap_or_default() ,  task_description));// hashvalue of combined information
			let encoded_payload = (owner.clone(), task_description.clone()).encode();
			let id = frame_support::Hashable::blake2_128(&encoded_payload);// hashvalue of combined information
			let new_task = Task::<T>{ id, credit: None, state: 1u8, owner: owner.clone(), description: task_description};
			
			//Write new task to storage
			Tasks::<T>::insert(id, new_task);
			
			Self::deposit_event(Event::NewTaskCreated{owner: owner.clone(), credit: None});
			Ok(())
		}
		
		
		fn do_reward_winner(winner: T::AccountId, amount: u32)-> DispatchResult{
			
			T::Score::deposit_creating(&winner.clone(), amount.into());
			
			Ok(())
		}

		fn do_convert_tokens_to_credits(account: T::AccountId, amount: TokenBalanceOf<T>) -> DispatchResult {
			// You need to define the conversion rate or logic based on your requirements
			// let conversion_rate = 2 as u32;
			// T::TokenManagement::withdraw(account.clone(), amount)?;
			// T::TaskCreditDeposit::deposit(account.clone(), conversion_rate);
			Self::deposit_event(Event::DepositCredit{owner: account.clone(), amt: None});
			Ok(())
		}

		fn do_convert_score_to_token(account: T::AccountId, amount: u32) -> DispatchResult {
			// You need to define the conversion rate or logic based on your requirements
			// let conversion_rate = 2 as u32;
			// T::Score::burn(100u32.into());
			// T::Token::deposit_creating(&account.clone(), 100u32.into());
			
			Self::deposit_event(Event::WithdrawScoreToToken{owner: account.clone(), amt: None});
			Ok(())
		}
		
	}

	// impl<T: Config> TokenManagement<T> for Pallet<T> {
	// 	// Implement methods or associated types for the Token type
	// 	// For example:
	// 	// fn convert_to_taskCredit(from: T::AccountId, amount: TokenBalanceOf<T>) -> DispatchResult {
	// 	//     // Your implementation here
	// 	//     Ok(())
	// 	// }
	// 	// or
	// 	// type Balance = T::Token::Balance;
	// 	// invoke when convert score -> token
	// 	fn deposit(owner: T::AccountId, amount: u32) -> DispatchResult{
	// 		// Your implementation here + balance ??
	// 		Ok(())
	// 	}

	// 	fn withdraw(owner: T::AccountId, amount: TokenBalanceOf<T>) -> DispatchResult{
	// 		Ok(())
	// 	}
		
	// }
	// impl<T: Config> TaskCreditDeposit<T> for Pallet<T> {
	// 	// Implement methods or associated types for the Token type
	// 	// For example:
	// 	fn deposit(owner: T::AccountId, amount: u32) -> DispatchResult{
	// 		// Your implementation here + balance ??
	// 		Ok(())
	// 	}

		
	// 	// or
	// 	// type Balance = T::Token::Balance;

	// }

	// impl<T: Config> TokenTest<T::AccountId>:Currency<AccountId>  {
	// 	fn transfer(
	// 		source: &AccountId,
	// 		dest: &AccountId,
	// 		value: Self::Balance,
	// 		existence_requirement: ExistenceRequirement,
	// 	) -> DispatchResult{
	// 		Ok(())
	// 	}
	// }
}



