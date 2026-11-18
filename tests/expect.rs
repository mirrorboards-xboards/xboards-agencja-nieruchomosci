//! What a scenario expects of a write the database should refuse.

use std::fmt::Debug;

use xboards_agencja_nieruchomosci::xmeta_room::RoomError;

/// Expects the database to refuse the write with a message that contains `rule`: the name of a
/// CHECK, the text of a trigger's RAISE, or e.g. „FOREIGN KEY”.
#[track_caller]
pub fn refused<T: Debug>(result: Result<T, RoomError>, rule: &str) {
    match result {
        Err(error) if error.is_refusal_by(rule) => {}
        Err(error) => panic!("refused for another reason than „{rule}”: {error}"),
        Ok(value) => panic!("accepted ({value:?}), but the rule „{rule}” should refuse it"),
    }
}
