use crate::{Error, mock::*};
use frame_support::{assert_ok, assert_noop};

// #[test]
// fn it_works_for_default_value() {
// 	new_test_ext().execute_with(|| {
// 		// Dispatch a signed extrinsic.
// 		assert_ok!(Guess::do_something(Origin::signed(1), 42));
// 		// Read pallet storage and assert an expected result.
// 		assert_eq!(Guess::something(), Some(42));
// 	});
// }


// #[test]
// fn correct_error_for_none_value() {
// 	new_test_ext().execute_with(|| {
// 		// Ensure the expected error is thrown when no value is present.
// 		assert_noop!(
// 			Guess::cause_error(Origin::signed(1)),
// 			Error::<Test>::NoneValue,
// 		);
// 	});
// }

#[test]
fn next_session_id_works() {
	new_test_ext().execute_with(|| {
		let next_session_id = Guess::next_session_id().unwrap();
		assert_eq!(next_session_id, 0);

		let next_session_id = Guess::next_session_id().unwrap();
		assert_eq!(next_session_id, 1);
	});
}
