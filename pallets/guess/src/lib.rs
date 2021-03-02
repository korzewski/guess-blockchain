#![cfg_attr(not(feature = "std"), no_std)]

use frame_support::{
	decl_module,
	decl_storage,
	decl_event,
	decl_error,
	dispatch::{DispatchError},
	codec::{Encode, Decode},
	traits::{Randomness},
};
use frame_system::{ensure_signed};
use sp_std::{result::{Result}};
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

type SessionId = u128;
type Bet = u32;
type GuessValue = u8;
type RandomValue = u32;

#[derive(Encode, Decode, Default, Clone)]
struct Session {
	id: SessionId,
	random_value: RandomValue,
	first_block: u32,
	last_block: u32,
}

decl_storage! {
	trait Store for Module<T: Config> as Guess {
		Sessions: map hasher(blake2_128_concat) SessionId => Option<Session>;
		Bets: double_map hasher(blake2_128_concat) SessionId, hasher(blake2_128_concat) T::AccountId => Option<(GuessValue, Bet)>;

		CurrentSessionId: SessionId;
		NextSessionId: SessionId;

		TemporaryRandomTest: map hasher(blake2_128_concat) RandomValue => u8;
	}
}

decl_event!(
	pub enum Event<T> where <T as frame_system::Config>::AccountId {
		NewSessionCreated(AccountId, SessionId, RandomValue),
	}
);

decl_error! {
	pub enum Error for Module<T: Config> {
		SessionIdOverflow,
	}
}

decl_module! {
	pub struct Module<T: Config> for enum Call where origin: T::Origin {
		type Error = Error<T>;

		fn deposit_event() = default;

		#[weight = 1000]
		pub fn join_to_session(origin, guess: GuessValue, bet: Bet) {
			let sender = ensure_signed(origin)?;

			let session = Self::get_current_session(&sender)?;
			Bets::<T>::insert(session.id, sender, (guess, bet));
		}
	}
}

impl<T: Config> Module<T> {
	fn create_new_session(sender: &T::AccountId) -> Result<Session, DispatchError> {
		let session = Session {
			id: Self::next_session_id()?,
			random_value: Self::get_random_number(sender),
			// TODO - GET CURRENT BLOCK ID
			first_block: 0,
			last_block: 0,
		};

		Sessions::insert(session.id, session.clone());
		CurrentSessionId::put(session.id);

		TemporaryRandomTest::mutate(session.random_value, |x| *x += 1);

		Ok(session)
	}

	fn get_current_session(sender: &T::AccountId) -> Result<Session, DispatchError> {
		let current_session_id = CurrentSessionId::get();

		let session = match Sessions::get(current_session_id) {
			Some(x) => x,
			None => Self::create_new_session(sender)?
		};

		Ok(session)
	}

	fn next_session_id() -> Result<SessionId, DispatchError> {
		let session_id = NextSessionId::get();
		let next_session_id = session_id.checked_add(1).ok_or(Error::<T>::SessionIdOverflow)?;
		NextSessionId::put(next_session_id);

		Ok(session_id)
	}

	fn get_random_number(sender: &T::AccountId) -> RandomValue {
		let random_seed = (
			T::Randomness::random_seed(),
			sender,
			<frame_system::Module<T>>::extrinsic_index(),
		).encode();

		let random_seed = BlakeTwo256::hash(&random_seed);

		let mut rng = <RandomNumberGenerator<BlakeTwo256>>::new(random_seed);

		rng.pick_u32(99) + 1
	}
}
