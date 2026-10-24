use super::*;
use time::Month;

/// `date` moved by a date picker key: a day with `h` `l`, a week with `k` `j`, a month with `H` `L`, arrows as their letters.
pub(super) fn shift(date: Date, code: KeyCode) -> Date {
    match code {
        KeyCode::Char('h') | KeyCode::Left => date.previous_day().unwrap_or(date),
        KeyCode::Char('l') | KeyCode::Right => date.next_day().unwrap_or(date),
        KeyCode::Char('k') | KeyCode::Up => date.checked_sub(time::Duration::WEEK).unwrap_or(date),
        KeyCode::Char('j') | KeyCode::Down => date.checked_add(time::Duration::WEEK).unwrap_or(date),
        KeyCode::Char('H') => add_months(date, -1),
        KeyCode::Char('L') => add_months(date, 1),
        _ => date,
    }
}

/// `date` moved by `months`, its day brought back to the last one of a shorter month.
fn add_months(date: Date, months: i32) -> Date {
    let index = date.year() * 12 + i32::from(u8::from(date.month())) - 1 + months;
    let year = index.div_euclid(12);
    let month = Month::try_from(index.rem_euclid(12) as u8 + 1).expect("a month is 1 to 12");
    Date::from_calendar_date(year, month, date.day().min(month.length(year))).unwrap_or(date)
}

#[cfg(test)]
mod tests;
