use ratatui::style::Stylize;
use ratatui::text::Span;

use super::style::icon;
use crate::theme::Theme;
use crate::toast::{Kind, Toast};

/// The toast as one span, coloured by its kind and cut to `room` characters with `…` when it is longer.
pub(super) fn toast_span(theme: &Theme, icons: bool, toast: &Toast, room: usize) -> Span<'static> {
    let kind = toast.notice.kind();
    let colour = match kind {
        Kind::Success => theme.success,
        Kind::Info => theme.secondary,
        Kind::Refusal => theme.alert,
    };
    let text = format!("{}{}", icon(icons, glyph(kind)), toast.notice.text());
    cut(&text, room).fg(colour)
}

/// The Nerd Font icon of a kind.
fn glyph(kind: Kind) -> char {
    match kind {
        Kind::Success => '\u{f058}',
        Kind::Info => '\u{f05a}',
        Kind::Refusal => '\u{f057}',
    }
}

/// `text` as it is when it fits in `room` characters, else its start and `…`.
fn cut(text: &str, room: usize) -> String {
    if text.chars().count() <= room {
        return text.to_string();
    }
    let kept: String = text.chars().take(room.saturating_sub(1)).collect();
    format!("{kept}…")
}

#[cfg(test)]
mod tests;
