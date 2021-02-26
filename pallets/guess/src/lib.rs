#![cfg_attr(not(feature = "std"), no_std)]

use frame_support::{
	decl_module,
	decl_storage,
	decl_event,
	decl_error,
	dispatch::{DispatchError},
	codec::{Encode, Decode},
};
use frame_system::{ensure_signed};
use sp_std::{result::{Result}};

#[cfg(test)]
mod mock;

#[cfg(test)]
mod tests;

pub trait Config: frame_system::Config {
	type Event: From<Event<Self>> + Into<<Self as frame_system::Config>::Event>;
}

type SessionId = u128;
type Bet = u32;
type GuessValue = u8;

#[derive(Encode, Decode, Default, Clone)]
struct Session {
	random_value: u8,
	first_block: u32,
	last_block: u32,
}

decl_storage! {
	trait Store for Module<T: Config> as Guess {
		Sessions: map hasher(blake2_128_concat) SessionId => Option<Session>;
		Bets: double_map hasher(blake2_128_concat) SessionId, hasher(blake2_128_concat) T::AccountId => Option<(GuessValue, Bet)>;

		NextSessionId: SessionId;
		CurrentSessionId get(fn session_id): u128;
	}
}

decl_event!(
	pub enum Event<T> where <T as frame_system::Config>::AccountId {
		SomethingStored(u32, AccountId),
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
			let account_id = ensure_signed(origin)?;

			let (session_id, _session) = Self::get_current_session()?;
			Bets::<T>::insert(session_id, account_id, (guess, bet));
		}
	}
}

impl<T: Config> Module<T> {
	fn create_new_session() -> Result<Session, DispatchError> {
		let session_id = Self::next_session_id()?;
		let session = Session {
			// TODO - GENERATE RANDOM 1 -> 100
			random_value: 8,
			// TODO - GET CURRENT BLOCK ID
			first_block: 0,
			last_block: 0,
		};

		Sessions::insert(session_id, session.clone());
		Ok(session)
	}

	fn get_current_session() -> Result<(SessionId, Session), DispatchError> {
		let id = CurrentSessionId::get();

		let session = Sessions::get(id)
			.unwrap_or(Self::create_new_session()?);

		Ok((id, session))
	}

	fn next_session_id() -> Result<SessionId, DispatchError> {
		let session_id = NextSessionId::get();
		let next_session_id = session_id.checked_add(1).ok_or(Error::<T>::SessionIdOverflow)?;
		NextSessionId::put(next_session_id);

		Ok(session_id)
	}
}
