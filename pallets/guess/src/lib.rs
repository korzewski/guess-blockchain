#![cfg_attr(not(feature = "std"), no_std)]

use frame_support::{
	decl_module,
	decl_storage,
	decl_event,
	decl_error,
	dispatch::{DispatchError},
	codec::{Encode, Decode},
	traits::{Randomness},
	debug,
	ensure,
};
use frame_system::{ensure_signed};
use sp_std::{result::{Result}, prelude::{Vec}};
use sp_runtime::{
	RandomNumberGenerator,
	traits::{BlakeTwo256, Hash}
};

#[cfg(test)]
mod mock;

#[cfg(test)]
mod tests;

pub trait Config: frame_system::Config {
	type Event: From<Event<Self>> + Into<<Self as frame_system::Config>::Event>;
	type Randomness: Randomness<Self::Hash>;
}

type SessionIdType = u128;
type BetType = u32;
type RandomType = u32;

#[derive(Encode, Decode, Clone, PartialEq, Eq, Debug)]
pub struct Bet<AccountId> {
	account_id: AccountId,
	guess: RandomType,
	bet: BetType,
}

decl_storage! {
	trait Store for Module<T: Config> as Guess {
		SessionId: SessionIdType;
		SessionLength: T::BlockNumber = T::BlockNumber::from(10u8);
		Bets: map hasher(blake2_128_concat) SessionIdType => Vec<Bet<T::AccountId>>;
	}
}

decl_module! {
	pub struct Module<T: Config> for enum Call where origin: T::Origin {
		type Error = Error<T>;

		fn deposit_event() = default;

		#[weight = 1000]
		pub fn add_new_bet(origin, guess: RandomType, bet: BetType) {
			let account_id = ensure_signed(origin)?;
			let session_id = SessionId::get();

			let new_bet = Bet {
				account_id,
				guess,
				bet,
			};

			let mut session_bets = Bets::<T>::get(session_id);
			session_bets.push(new_bet.clone());

			Bets::<T>::insert(session_id, session_bets);

			Self::deposit_event(RawEvent::NewBet(session_id, new_bet));
		}

		fn on_finalize(block_number: T::BlockNumber) {
			if block_number % SessionLength::<T>::get() == T::BlockNumber::from(0u8) {
				match Self::finalize_the_session(block_number) {
					Ok(_) => (),
					Err(err) => {
						debug::info!("--- error --- finalize_the_session {:?}", err);
						()
					}
				}
			}
		}
	}
}

impl<T: Config> Module<T> {
	fn finalize_the_session(block_number: T::BlockNumber) -> Result<(), DispatchError> {
		debug::info!("--- finalize_the_session() - block: {:?}", block_number);
		
		let random_guess = Self::get_random_number();
		let session_id = Self::next_session_id()?;
		
		let session_bets = Bets::<T>::get(session_id);
		
		ensure!(session_bets.len() > 1, Error::<T>::NotEnoughBets);
		
		let mut winner = session_bets[0].clone();
		let mut winner_guess_diff = Self::get_guess_diff(random_guess, winner.guess);

		for bet in session_bets {
			let guess_diff = Self::get_guess_diff(random_guess, bet.guess);
			debug::info!("--- random_guess: {}, bet.guess: {}, guess_diff: {}", random_guess, bet.guess, guess_diff);
			
			if guess_diff < winner_guess_diff {
				winner = bet;
				winner_guess_diff = guess_diff;
			}
		}

		Self::deposit_event(RawEvent::NewWinner(session_id, random_guess, winner));

		Ok(())
	}

	fn get_guess_diff(a: RandomType, b: RandomType) -> RandomType {
		if a > b { a - b } else { b - a }
	}

	fn next_session_id() -> Result<SessionIdType, DispatchError> {
		let session_id = SessionId::get();
		let next_session_id = session_id.checked_add(1).ok_or(Error::<T>::SessionIdOverflow)?;
		SessionId::put(next_session_id);

		Ok(session_id)
	}

	fn get_random_number() -> RandomType {
		let random_seed = (
			T::Randomness::random_seed(),
			<frame_system::Module<T>>::extrinsic_index(),
		).encode();

		let random_seed = BlakeTwo256::hash(&random_seed);

		let mut rng = <RandomNumberGenerator<BlakeTwo256>>::new(random_seed);

		rng.pick_u32(99) + 1
	}
}

decl_event!(
	pub enum Event<T> where <T as frame_system::Config>::AccountId {
		NewBet(SessionIdType, Bet<AccountId>),
		NewWinner(SessionIdType, RandomType, Bet<AccountId>),
	}
);

decl_error! {
	pub enum Error for Module<T: Config> {
		SessionIdOverflow,
		NotEnoughBets,
	}
}