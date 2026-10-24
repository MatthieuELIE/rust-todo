use std::collections::HashMap;

use super::*;
use crate::todo::WAIT;

impl App {
    /// Names completing the popup's tag, from the task file and `done.txt`, with their counts in the task file, most used first; and the row picked.
    pub fn completions(&self) -> (Vec<(String, usize)>, usize) {
        let Focus::Popup(popup) = &self.focus else {
            return Default::default();
        };
        let Some(tag) = popup.editor.tag() else {
            return Default::default();
        };
        let (sigil, names): (&str, fn(&Todo) -> Vec<&str>) = match tag.chars().next() {
            Some('+') => ("+", Todo::projects),
            Some('@') => ("@", Todo::contexts),
            _ => (WAIT, Todo::waits),
        };
        let mut counts = HashMap::new();
        for name in self.todos.iter().flat_map(names) {
            *counts.entry(format!("{sigil}{name}")).or_insert(0) += 1;
        }
        for name in self.history.iter().flat_map(names) {
            counts.entry(format!("{sigil}{name}")).or_insert(0);
        }
        let tag = tag.to_lowercase();
        let mut names: Vec<(String, usize)> = counts.into_iter().filter(|(name, _)| name.to_lowercase().starts_with(&tag)).collect();
        names.sort_by_key(|(name, count)| (Reverse(*count), name.to_lowercase(), name.clone()));
        let selected = popup.selected.min(names.len().saturating_sub(1));
        (names, selected)
    }
}

#[cfg(test)]
mod tests;
