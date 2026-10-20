use ratatui::Frame;
use ratatui::layout::{Constraint, Layout};
use ratatui::style::{Color, Stylize};
use ratatui::text::Line;
use ratatui::widgets::{ListState, Paragraph, Wrap};
use time::Date;

use crate::theme::Theme;
use crate::tui::{App, Focus};
use details::{DETAILS_HEIGHT, draw_details, label, text_of};
use help::draw_help;
use list::draw_list;
use panel::{PANEL_WIDTH, draw_panel};
use popup::draw_popup;
use status::draw_status;

/// Fewest rows the list keeps inside its card; on a screen too low for them and the detail card, that card is hidden.
const MIN_LIST_ROWS: u16 = 5;

/// Draws the filter panel and the task list above the status bar, due dates against `today`; `scroll` keeps the list's offset between frames.
pub fn draw(frame: &mut Frame, app: &App, scroll: &mut ListState, today: Date) {
    let theme = &app.theme;
    let [main_area, status_area] = Layout::vertical([Constraint::Fill(1), Constraint::Length(1)]).areas(frame.area());
    let [panel_area, list_area] = Layout::horizontal([Constraint::Length(PANEL_WIDTH), Constraint::Fill(1)])
        .spacing(1)
        .areas(main_area);

    draw_panel(frame, app, panel_area);

    if list_area.height >= MIN_LIST_ROWS + 2 + 1 + DETAILS_HEIGHT {
        let spare = list_area.height - (MIN_LIST_ROWS + 2 + 1 + DETAILS_HEIGHT);
        let text_width = list_area.width.saturating_sub(2 + label(theme, "Text").width() as u16);
        let text_rows = app.selected_task().map_or(1, |todo| {
            text_of(theme, todo, (!todo.done).then_some(today)).line_count(text_width) as u16
        });
        let height = DETAILS_HEIGHT + text_rows.saturating_sub(1).min(spare);
        let [list_area, details_area] = Layout::vertical([Constraint::Fill(1), Constraint::Length(height)])
            .spacing(1)
            .areas(list_area);
        draw_list(frame, app, list_area, scroll, today);
        draw_details(frame, theme, app.icons, app.selected_task(), details_area, today);
    } else {
        draw_list(frame, app, list_area, scroll, today);
    }
    draw_status(frame, app, status_area);
    match &app.focus {
        Focus::Help(scroll) => draw_help(frame, app, *scroll, main_area),
        Focus::Popup(popup) => draw_popup(frame, app, popup, main_area, today),
        _ => {}
    }
    for cell in frame.buffer_mut().content.iter_mut() {
        if cell.bg == Color::Reset {
            cell.bg = theme.background;
        }
        if cell.fg == Color::Reset {
            cell.fg = theme.primary;
        }
    }
}

/// Draws an error that stops the list from opening, and how to leave.
pub fn draw_error(frame: &mut Frame, theme: &Theme, message: &str) {
    let text = vec![
        Line::from(format!(" {message}")).fg(theme.alert),
        Line::from(" press any key to quit").fg(theme.tertiary),
    ];
    frame.render_widget(Paragraph::new(text).wrap(Wrap { trim: false }), frame.area());
}

mod details;
mod help;
mod list;
mod panel;
mod popup;
mod status;
mod style;

pub use help::HELP;
pub use list::{line, number};

#[cfg(test)]
mod tests;
