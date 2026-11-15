use super::*;
use crate::guard::Folder;

fn written(name: &str, text: &str) -> (Folder, PathBuf) {
    let folder = Folder::new(&format!("config-{name}"));
    let path = folder.join("config.toml");
    std::fs::write(&path, text).unwrap();
    (folder, path)
}

fn refused(name: &str, text: &str) -> String {
    match theme(&written(name, text).1, false) {
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
    let folder = Folder::new("config-absent");

    assert!(theme(&folder.join("config.toml"), false).is_ok());
}

#[test]
fn a_file_that_cannot_be_read_is_refused_with_its_path() {
    let folder = Folder::new("config-folder");
    let path = folder.join("config.toml");
    std::fs::create_dir(&path).unwrap();

    let message = theme(&path, false).err().unwrap();

    assert!(message.starts_with(&format!("could not read {}: ", path.display())), "{message:?}");
}

#[test]
fn the_terminal_name_is_read() {
    assert!(theme(&written("terminal", "[theme]\nname = \"terminal\"\n").1, false).is_ok());
}

#[test]
fn the_four_flavours_are_read_and_differ() {
    let flavours: Vec<Theme> = ["latte", "frappe", "macchiato", "mocha"]
        .iter()
        .map(|flavour| {
            let text = format!("[theme]\nname = \"catppuccin-{flavour}\"\n");
            theme(&written(flavour, &text).1, false).unwrap().0
        })
        .collect();

    for (i, flavour) in flavours.iter().enumerate() {
        assert!(flavours[i + 1..].iter().all(|other| other != flavour));
    }
}

#[test]
fn a_malformed_file_is_refused_with_its_path_and_line() {
    let (_folder, path) = written("malformed", "[theme\n");

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
    assert!(theme(&written("icons-on", "[theme]\nicons = true\n").1, false).unwrap().1);
    assert!(!theme(&written("icons-off", "[theme]\nname = \"terminal\"\n").1, false).unwrap().1);
}

#[test]
fn icons_that_are_not_a_boolean_are_refused_with_the_path_and_line() {
    let (_folder, path) = written("icons-text", "[theme]\nicons = \"yes\"\n");

    let message = theme(&path, false).err().unwrap();

    assert!(message.starts_with(&path.display().to_string()));
    assert!(message.contains("line 2"));
}

#[test]
fn a_refusal_quoting_an_escape_is_cleaned() {
    for (name, text) in [
        ("esc-value", "[theme]\nname = \"\\u001b[31mEVIL\"\n"),
        ("esc-key", "[theme]\n\"\\u001b[31mkey\" = 1\n"),
        ("esc-comment", "[theme]\n# \u{1b}[31m\nname = 1\n"),
    ] {
        let message = refused(name, text);

        assert!(!message.contains('\u{1b}'), "{text:?} gave {message:?}");
        assert!(message.contains('\u{fffd}'), "{text:?} gave {message:?}");
    }
}

#[test]
fn a_cleaned_refusal_keeps_its_lines() {
    let message = refused("lines", "[theme\n");

    assert!(message.contains("\n  |\n1 | [theme\n"), "{message:?}");
}

#[test]
fn a_cleaned_refusal_keeps_its_tabs() {
    let message = refused("tabs", "[theme]\n\tname = 1\n");

    assert!(message.contains("2 | \tname = 1\n"), "{message:?}");
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
    let (_nameless_folder, nameless) = written("nameless", "[theme]\n");
    let (_named_folder, named) = written("named", "[theme]\nname = \"terminal\"\n");
    let absent_folder = Folder::new("config-nofile");
    let absent = absent_folder.join("config.toml");

    assert!(theme(&nameless, true).unwrap().0 == Theme::NONE);
    assert!(theme(&absent, true).unwrap().0 == Theme::NONE);
    assert!(theme(&named, true).unwrap().0 == Theme::TERMINAL);
    assert!(theme(&nameless, false).unwrap().0 == Theme::TERMINAL);
}
