use super::super::line;
use super::super::tests::{MOCHA, TODAY, style_of};
use super::*;

#[test]
fn key_values_are_greyed_but_not_urls_times_or_labels_in_mocha() {
    let text = "Call wait:figma rec:1w a:b:c é-t_2:x https://herdr.dev 10:30 Note: wait: :x _a:b a/b:c";

    for greyed in ["wait:figma", "rec:1w", "a:b:c", "é-t_2:x"] {
        assert_eq!(style_of(text, greyed), Style::new().fg(MOCHA.tertiary), "{greyed}");
    }
    for plain in ["https://herdr.dev", "10:30", "Note:", "wait:", ":x", "_a:b", "a/b:c"] {
        assert_eq!(style_of(text, plain), Style::new(), "{plain}");
    }
}

#[test]
fn a_due_date_is_bold_red_once_past_yellow_on_the_day_and_greyed_later_or_when_not_a_date_in_mocha() {
    let text = "Ship due:2026-09-25 due:2026-09-26 due:2026-09-27 due:someday";

    assert_eq!(style_of(text, "due:2026-09-25"), Style::new().bold().fg(MOCHA.alert));
    assert_eq!(style_of(text, "due:2026-09-26"), Style::new().fg(MOCHA.due_today));
    assert_eq!(style_of(text, "due:2026-09-27"), Style::new().fg(MOCHA.tertiary));
    assert_eq!(style_of(text, "due:someday"), Style::new().fg(MOCHA.tertiary));
}

#[test]
fn a_late_due_date_is_bold_in_every_theme_and_one_due_today_is_not() {
    let text = "Ship due:2026-09-25 due:2026-09-26";

    for theme in [Theme::LATTE, Theme::FRAPPE, Theme::MACCHIATO, Theme::MOCHA, Theme::TERMINAL, Theme::NONE] {
        let style_of = |token: &str| {
            let line = line(&theme, 1, &Todo::from_line(text), TODAY);
            line.iter().find(|span| span.content == token).expect(token).style
        };
        assert_eq!(style_of("due:2026-09-25"), Style::new().bold().fg(theme.alert));
        assert_eq!(style_of("due:2026-09-26"), Style::new().fg(theme.due_today));
    }
}
