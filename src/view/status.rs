use ratatui::Frame;
use ratatui::layout::Rect;
use ratatui::style::Stylize;
use ratatui::text::{Line, Span};
use ratatui::widgets::Paragraph;

use super::style::{mode_block, tag_style};
use super::toast::toast_span;
use crate::editor::Mode;
use crate::tui::{App, Focus};

/// Draws the status bar: mode block and active filters on the left, or the toast in their place, the keys of a command under way and `? help` on the right.
pub(super) fn draw_status(frame: &mut Frame, app: &App, area: Rect) {
    let theme = &app.theme;
    let mode = match &app.focus {
        Focus::Search => "SEARCH",
        Focus::Popup(popup) if popup.picker.is_some() => "DATE",
        Focus::Popup(popup) => match popup.editor.mode {
            Mode::Insert => "INSERT",
            Mode::Normal => "NORMAL",
        },
        Focus::Panel => "PANEL",
        Focus::List => "LIST",
        Focus::Help(_) => "HELP",
    };
    let filters = if matches!(app.focus, Focus::Search) {
        format!("/{}▌", app.search)
    } else {
        let search = (!app.search.is_empty()).then(|| format!("/{}", app.search));
        [app.filter.clone(), search].into_iter().flatten().collect::<Vec<_>>().join("  ")
    };
    let mut status = Line::from_iter([mode_block(theme, mode, true), " ".into()]);
    let mut right = Line::from_iter([app.pending().fg(theme.accent), " ".into()]);
    if !matches!(mode, "INSERT" | "SEARCH" | "HELP") {
        right.extend(["?".bold(), " help ".fg(theme.tertiary)]);
    }
    if let Some(toast) = app.toast.as_ref().filter(|_| !matches!(app.focus, Focus::Search)) {
        let room = (area.width as usize).saturating_sub(status.width() + right.width() + 1);
        status.push_span(toast_span(theme, app.icons, toast, room));
    } else {
        for (i, word) in filters.split(' ').enumerate() {
            status.extend([
                if i > 0 { " " } else { "" }.into(),
                Span::styled(word.to_string(), tag_style(theme, word)),
            ]);
        }
    }
    frame.render_widget(Paragraph::new(status), area);
    frame.render_widget(Paragraph::new(right).right_aligned(), area);
}

#[cfg(test)]
mod tests;
