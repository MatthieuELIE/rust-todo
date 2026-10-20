use ratatui::Frame;
use ratatui::layout::Rect;
use ratatui::style::Stylize;
use ratatui::text::{Line, Span};
use ratatui::widgets::Paragraph;

use super::style::tag_style;
use crate::editor::Mode;
use crate::theme::Theme;
use crate::tui::{App, Focus};

/// Draws the status bar: mode block, active filters, then the mode's keys when they fit; on the right the message, or the keys of a command under way.
pub(super) fn draw_status(frame: &mut Frame, app: &App, area: Rect) {
    let theme = &app.theme;
    let (mode, keys) = match &app.focus {
        Focus::Search => ("SEARCH", "⏎ keep · esc clear"),
        Focus::Popup(popup) if popup.picker.is_some() => ("DATE", "hjkl move · H/L month · ⏎ pick · esc close"),
        Focus::Popup(popup) => match popup.editor.mode {
            Mode::Insert if app.completions().0.is_empty() => ("INSERT", "esc normal · ⏎ save"),
            Mode::Insert => ("INSERT", "esc normal · ⏎ save · tab complete"),
            Mode::Normal => ("NORMAL", "i insert · p priority · ⏎ save · esc cancel"),
        },
        Focus::Panel => ("PANEL", "j/k filter · esc all tasks · tab back"),
        Focus::List | Focus::Help(_) if app.in_history => ("LIST", "r reopen · H back · ? help"),
        Focus::List | Focus::Help(_) => ("LIST", "⏎ edit · o add · x done · p priority · ? help"),
    };
    let filters = if matches!(app.focus, Focus::Search) {
        format!("/{}▌", app.search)
    } else {
        let search = (!app.search.is_empty()).then(|| format!("/{}", app.search));
        [app.filter.clone(), search].into_iter().flatten().collect::<Vec<_>>().join("  ")
    };
    let mut status = Line::from_iter([mode_block(theme, mode), " ".into()]);
    for (i, word) in filters.split(' ').enumerate() {
        status.extend([
            if i > 0 { " " } else { "" }.into(),
            Span::styled(word.to_string(), tag_style(theme, word)),
        ]);
    }
    let mut hints = Line::from(if filters.is_empty() { "" } else { "  " });
    for (i, hint) in keys.split(" · ").enumerate() {
        let (key, action) = hint.split_once(' ').unwrap_or_default();
        hints.extend([
            if i > 0 { " · " } else { "" }.fg(theme.separator),
            key.bold(),
            format!(" {action}").fg(theme.tertiary),
        ]);
    }
    let pending = app.pending();
    let reserved = if pending.is_empty() { 0 } else { pending.len() + 2 };
    if app.message.is_none() && status.width() + hints.width() + reserved <= area.width as usize {
        status.extend(hints);
    }
    frame.render_widget(Paragraph::new(status), area);
    if let Some(right) = app.message.as_deref().or((!pending.is_empty()).then_some(&pending)) {
        let colour = if app.refused { theme.alert } else { theme.secondary };
        frame.render_widget(Paragraph::new(format!("{right} ").fg(colour)).right_aligned(), area);
    }
}

/// A mode's name in bold on the accent, as the status bar shows it.
fn mode_block(theme: &Theme, mode: &str) -> Span<'static> {
    format!(" {mode} ").bold().fg(theme.on_accent).bg(theme.accent)
}

#[cfg(test)]
mod tests;
