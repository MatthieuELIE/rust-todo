use std::io;
use std::time::{Duration, SystemTime};

use super::{must_draw, must_read, remembered};

fn stamp(secs: u64, len: u64) -> Option<(SystemTime, u64)> {
    Some((SystemTime::UNIX_EPOCH + Duration::from_secs(secs), len))
}

#[test]
fn the_first_turn_reads_the_file() {
    assert!(must_read(&None, &Ok(stamp(1, 10))));
}

#[test]
fn a_file_with_the_same_date_and_size_is_not_read() {
    assert!(!must_read(&Some(stamp(1, 10)), &Ok(stamp(1, 10))));
}

#[test]
fn a_new_date_or_a_new_size_reads_the_file() {
    assert!(must_read(&Some(stamp(1, 10)), &Ok(stamp(2, 10))));
    assert!(must_read(&Some(stamp(1, 10)), &Ok(stamp(1, 11))));
}

#[test]
fn a_file_that_appears_or_goes_is_read() {
    assert!(must_read(&Some(None), &Ok(stamp(1, 10))));
    assert!(must_read(&Some(stamp(1, 10)), &Ok(None)));
}

#[test]
fn a_file_still_missing_is_not_read() {
    assert!(!must_read(&Some(None), &Ok(None)));
}

#[test]
fn a_failed_metadata_is_a_failed_read() {
    assert!(must_read(&Some(stamp(1, 10)), &Err(io::Error::other("denied"))));
}

#[test]
fn a_stamp_is_remembered_once_the_file_read_well() {
    assert_eq!(remembered(Ok(stamp(1, 10)), false), Some(stamp(1, 10)));
}

#[test]
fn a_stamp_is_forgotten_when_the_read_failed_so_the_next_turn_reads_again() {
    assert_eq!(remembered(Ok(stamp(1, 10)), true), None);
}

#[test]
fn a_failed_metadata_is_forgotten() {
    assert_eq!(remembered(Err(io::Error::other("denied")), false), None);
}

#[test]
fn nothing_happening_draws_nothing() {
    assert!(!must_draw(false, false, false, false));
}

#[test]
fn each_of_an_event_a_reload_an_expired_toast_and_a_new_day_draws() {
    assert!(must_draw(true, false, false, false));
    assert!(must_draw(false, true, false, false));
    assert!(must_draw(false, false, true, false));
    assert!(must_draw(false, false, false, true));
}
