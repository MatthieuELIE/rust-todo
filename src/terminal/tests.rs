use std::io;

use super::reset_after;

const DISABLE_BRACKETED_PASTE: &[u8] = b"\x1b[?2004l";
const DEFAULT_CURSOR_SHAPE: &[u8] = b"\x1b[0 q";

#[test]
fn a_loop_that_ends_well_writes_the_reset_sequences_and_succeeds() {
    let mut out = Vec::new();

    let result = reset_after(&mut out, Ok(()));

    assert!(result.is_ok());
    assert_eq!(out, [DISABLE_BRACKETED_PASTE, DEFAULT_CURSOR_SHAPE].concat());
}

#[test]
fn a_loop_that_ends_on_an_error_still_writes_the_reset_sequences() {
    let mut out = Vec::new();

    let _ = reset_after(&mut out, Err(io::Error::other("draw failed")));

    assert_eq!(out, [DISABLE_BRACKETED_PASTE, DEFAULT_CURSOR_SHAPE].concat());
}

#[test]
fn the_error_that_comes_back_is_the_loops_not_the_resets() {
    struct Broken;

    impl io::Write for Broken {
        fn write(&mut self, _: &[u8]) -> io::Result<usize> {
            Err(io::Error::other("write failed"))
        }

        fn flush(&mut self) -> io::Result<()> {
            Err(io::Error::other("flush failed"))
        }
    }

    let result = reset_after(&mut Broken, Err(io::Error::other("draw failed")));

    assert_eq!(result.unwrap_err().to_string(), "draw failed");
}
