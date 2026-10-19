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
    #[serde(default)]
    name: Name,
}

#[derive(Deserialize, Default)]
enum Name {
    #[serde(rename = "catppuccin-latte")]
    CatppuccinLatte,
    #[serde(rename = "catppuccin-frappe")]
    CatppuccinFrappe,
    #[serde(rename = "catppuccin-macchiato")]
    CatppuccinMacchiato,
    #[serde(rename = "catppuccin-mocha")]
    CatppuccinMocha,
    #[default]
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

/// The theme `path` asks for: the default when the file is absent, an error naming the path and the fault when it is refused.
pub fn theme(path: &Path) -> Result<Theme, String> {
    let text = match std::fs::read_to_string(path) {
        Ok(text) => text,
        Err(e) if e.kind() == ErrorKind::NotFound => return Ok(Theme::default()),
        Err(e) => return Err(format!("could not read {}: {e}", path.display())),
    };
    let config: Config = toml::from_str(&text).map_err(|e| format!("{}: {e}", path.display()))?;
    Ok(config.theme.name.theme())
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
        match theme(&written(name, text)) {
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

        assert!(theme(&path).is_ok());
    }

    #[test]
    fn the_terminal_name_is_read() {
        assert!(theme(&written("terminal", "[theme]\nname = \"terminal\"\n")).is_ok());
    }

    #[test]
    fn the_four_flavours_are_read_and_differ() {
        let flavours: Vec<Theme> = ["latte", "frappe", "macchiato", "mocha"]
            .iter()
            .map(|flavour| {
                let text = format!("[theme]\nname = \"catppuccin-{flavour}\"\n");
                theme(&written(flavour, &text)).unwrap()
            })
            .collect();

        for (i, flavour) in flavours.iter().enumerate() {
            assert!(flavours[i + 1..].iter().all(|other| other != flavour));
        }
    }

    #[test]
    fn a_malformed_file_is_refused_with_its_path_and_line() {
        let path = written("malformed", "[theme\n");

        let message = theme(&path).err().unwrap();

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
        assert!(refused("unknown-key", "[theme]\nicons = true\n").contains("icons"));
    }

    #[test]
    fn an_unknown_section_is_refused() {
        assert!(refused("unknown-section", "[colours]\nname = \"catppuccin-mocha\"\n").contains("colours"));
    }
}
