use std::ffi::OsString;
use std::io::ErrorKind;
use std::path::{Path, PathBuf};

use serde::Deserialize;

use crate::view::Theme;

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

/// The theme `path` asks for, `no_color` deciding when it names none, and whether it asks for icons: an error naming the path and the fault when the file is refused.
pub fn theme(path: &Path, no_color: bool) -> Result<(Theme, bool), String> {
    let text = match std::fs::read_to_string(path) {
        Ok(text) => text,
        Err(e) if e.kind() == ErrorKind::NotFound => return Ok((choose(no_color, None), false)),
        Err(e) => return Err(format!("could not read {}: {e}", path.display())),
    };
    let config: Config = toml::from_str(&text).map_err(|e| format!("{}: {e}", path.display()))?;
    Ok((choose(no_color, config.theme.name), config.theme.icons))
}

#[cfg(test)]
mod tests {
    use super::*;

    fn written(name: &str, text: &str) -> PathBuf {
        let folder = std::env::temp_dir().join(format!("todo-config-{}-{name}", std::process::id()));
        std::fs::create_dir_all(&folder).unwrap();
        let path = folder.join("config.toml");
        std::fs::write(&path, text).unwrap();
        path
    }

    fn refused(name: &str, text: &str) -> String {
        match theme(&written(name, text), false) {
            Ok(_) => panic!("{text:?} should be refused"),
            Err(message) => message,
        }
    }

    #[test]
    fn the_path_follows_xdg_config_home_then_the_home_folder() {
        assert_eq!(path(Some("/x".into()), Some("/h".into())), PathBuf::from("/x/todo/config.toml"));
        assert_eq!(path(None, Some("/h".into())), PathBuf::from("/h/.config/todo/config.toml"));
        assert_eq!(path(Some("".into()), Some("/h".into())), PathBuf::from("/h/.config/todo/config.toml"));
    }

    #[test]
    fn an_absent_file_gives_the_default_theme() {
        let path = std::env::temp_dir()
            .join(format!("todo-config-{}-absent", std::process::id()))
            .join("config.toml");

        assert!(theme(&path, false).is_ok());
    }

    #[test]
    fn the_terminal_name_is_read() {
        assert!(theme(&written("terminal", "[theme]\nname = \"terminal\"\n"), false).is_ok());
    }

    #[test]
    fn the_four_flavours_are_read_and_differ() {
        let flavours: Vec<Theme> = ["latte", "frappe", "macchiato", "mocha"]
            .iter()
            .map(|flavour| {
                let text = format!("[theme]\nname = \"catppuccin-{flavour}\"\n");
                theme(&written(flavour, &text), false).unwrap().0
            })
            .collect();

        for (i, flavour) in flavours.iter().enumerate() {
            assert!(flavours[i + 1..].iter().all(|other| other != flavour));
        }
    }

    #[test]
    fn a_malformed_file_is_refused_with_its_path_and_line() {
        let path = written("malformed", "[theme\n");

        let message = theme(&path, false).err().unwrap();

        assert!(message.starts_with(&path.display().to_string()));
        assert!(message.contains("line 1"));
    }

    #[test]
    fn an_unknown_name_is_refused_listing_the_valid_names() {
        let message = refused("unknown-name", "[theme]\nname = \"nord\"\n");

        assert!(message.contains("line 2"));
        assert!(message.contains("nord"));
        for name in [
            "catppuccin-latte",
            "catppuccin-frappe",
            "catppuccin-macchiato",
            "catppuccin-mocha",
            "terminal",
        ] {
            assert!(message.contains(name), "{name} missing from {message}");
        }
    }

    #[test]
    fn an_unknown_key_is_refused() {
        assert!(refused("unknown-key", "[theme]\ncolour = true\n").contains("colour"));
    }

    #[test]
    fn icons_are_read_and_default_to_false() {
        assert!(theme(&written("icons-on", "[theme]\nicons = true\n"), false).unwrap().1);
        assert!(!theme(&written("icons-off", "[theme]\nname = \"terminal\"\n"), false).unwrap().1);
    }

    #[test]
    fn icons_that_are_not_a_boolean_are_refused_with_the_path_and_line() {
        let path = written("icons-text", "[theme]\nicons = \"yes\"\n");

        let message = theme(&path, false).err().unwrap();

        assert!(message.starts_with(&path.display().to_string()));
        assert!(message.contains("line 2"));
    }

    #[test]
    fn an_unknown_section_is_refused() {
        assert!(refused("unknown-section", "[colours]\nname = \"catppuccin-mocha\"\n").contains("colours"));
    }

    #[test]
    fn no_color_set_and_not_empty_asks_for_no_colour() {
        assert!(!no_color(None));
        assert!(!no_color(Some("".into())));
        assert!(no_color(Some("1".into())));
    }

    #[test]
    fn a_name_wins_over_no_color_and_without_one_no_color_gives_none() {
        for no_color in [false, true] {
            assert!(choose(no_color, Some(Name::CatppuccinMocha)) == Theme::MOCHA);
            assert!(choose(no_color, Some(Name::Terminal)) == Theme::TERMINAL);
        }
        assert!(choose(true, None) == Theme::NONE);
        assert!(choose(false, None) == Theme::TERMINAL);
    }

    #[test]
    fn a_file_without_a_name_follows_no_color_and_one_with_a_name_does_not() {
        let nameless = written("nameless", "[theme]\n");
        let named = written("named", "[theme]\nname = \"terminal\"\n");
        let absent = std::env::temp_dir()
            .join(format!("todo-config-{}-nofile", std::process::id()))
            .join("config.toml");

        assert!(theme(&nameless, true).unwrap().0 == Theme::NONE);
        assert!(theme(&absent, true).unwrap().0 == Theme::NONE);
        assert!(theme(&named, true).unwrap().0 == Theme::TERMINAL);
        assert!(theme(&nameless, false).unwrap().0 == Theme::TERMINAL);
    }
}
