use super::*;
use time::macros::date;

#[test]
fn a_styled_line_replaces_the_control_characters_of_the_task() {
    let todo = Todo::from_line("Pay \u{1b}[2Jrent");

    let styled = styled(&view::line(&Theme::default(), 1, &todo, date!(2026 - 09 - 26)));

    assert!(styled.contains("\u{fffd}[2Jrent"));
    assert!(!styled.contains("\u{1b}[2J"));
}

#[test]
fn a_styled_line_leaves_its_number_out() {
    for line in ["(A) Pay rent", "x 2026-09-03 Pay rent"] {
        let styled = styled(&view::line(&Theme::default(), 7, &Todo::from_line(line), date!(2026 - 09 - 26)));

        assert!(!styled.contains(&view::number(7)));
        assert!(styled.contains("rent"));
    }
}

#[test]
fn a_styled_line_replaces_the_bidi_and_zero_width_characters_but_keeps_accents_and_emoji() {
    let todo = Todo::from_line("Pay\u{202e} re\u{200b}nt é 🏠");

    let styled = styled(&view::line(&Theme::default(), 1, &todo, date!(2026 - 09 - 26)));

    assert!(styled.contains("Pay\u{fffd} re\u{fffd}nt é 🏠"));
}
