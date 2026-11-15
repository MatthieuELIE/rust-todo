use std::ffi::OsString;
use std::io::ErrorKind;
use std::path::{Path, PathBuf};

use serde::Deserialize;

use crate::theme::Theme;

#[derive(Deserialize, Default)]
#[serde(deny_unknown_fields)]
struct Config {
    #[serde(default)]
    theme: Section,
}

#[derive(Deserialize, Default)]
#[serde(deny_unknown_fields)]
struct Section {
    name: Option<Name>,
    #[serde(default)]
    icons: bool,
}

#[derive(Deserialize)]
pub enum Name {
    #[serde(rename = "catppuccin-latte")]
    CatppuccinLatte,
    #[serde(rename = "catppuccin-frappe")]
    CatppuccinFrappe,
    #[serde(rename = "catppuccin-macchiato")]
    CatppuccinMacchiato,
    #[serde(rename = "catppuccin-mocha")]
    CatppuccinMocha,
    #[serde(rename = "terminal")]
    Terminal,
}

impl Name {
    fn theme(&self) -> Theme {
        match self {
            Name::CatppuccinLatte => Theme::LATTE,
            Name::CatppuccinFrappe => Theme::FRAPPE,
            Name::CatppuccinMacchiato => Theme::MACCHIATO,
            Name::CatppuccinMocha => Theme::MOCHA,
            Name::Terminal => Theme::TERMINAL,
        }
    }
}

/// Where `config.toml` lives: in `$XDG_CONFIG_HOME/todo/`, or `~/.config/todo/` when the variable is absent.
pub fn path(xdg_config_home: Option<OsString>, home: Option<PathBuf>) -> PathBuf {
    let config_home = xdg_config_home
        .filter(|value| !value.is_empty())
        .map(PathBuf::from)
        .unwrap_or_else(|| home.unwrap_or_default().join(".config"));
    config_home.join("todo").join("config.toml")
}

/// Whether `NO_COLOR` asks for no colour: set and not empty.
pub fn no_color(value: Option<OsString>) -> bool {
    value.is_some_and(|value| !value.is_empty())
}

/// The theme to start with: the config's `name` when it gives one, else no colour when `no_color`, else the terminal's own.
pub fn choose(no_color: bool, name: Option<Name>) -> Theme {
    match name {
        Some(name) => name.theme(),
        None if no_color => Theme::NONE,
        None => Theme::default(),
    }
}

/// `message` with each control character but the line breaks and tabs replaced, so that a line of the file cannot drive the terminal.
fn cleaned(message: &str) -> String {
    message.replace(|c| crate::todo::is_control(c) && !matches!(c, '\n' | '\t'), "\u{fffd}")
}

/// The theme `path` asks for, `no_color` deciding when it names none, and whether it asks for icons: an error naming the path and the fault when the file is refused.
pub fn theme(path: &Path, no_color: bool) -> Result<(Theme, bool), String> {
    let text = match std::fs::read_to_string(path) {
        Ok(text) => text,
        Err(e) if e.kind() == ErrorKind::NotFound => return Ok((choose(no_color, None), false)),
        Err(e) => return Err(format!("could not read {}: {e}", path.display())),
    };

    let config: Config = toml::from_str(&text).map_err(|e| cleaned(&format!("{}: {e}", path.display())))?;
    Ok((choose(no_color, config.theme.name), config.theme.icons))
}

#[cfg(test)]
mod tests;
