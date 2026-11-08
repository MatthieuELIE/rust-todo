#![cfg(unix)]

#[allow(dead_code)]
mod common;

use std::io::{Read, Write};
use std::os::unix::fs::PermissionsExt;
use std::path::{Path, PathBuf};
use std::process::{Child, ChildStdin, Command, ExitStatus, Stdio};
use std::sync::{Arc, Mutex};
use std::thread::JoinHandle;
use std::time::{Duration, Instant};

use common::folder;

const DEADLINE: Duration = Duration::from_secs(10);

/// `todo` running in a terminal of 109×36 that `script` opens, its output gathered as it comes.
struct Pty {
    child: Child,
    stdin: ChildStdin,
    output: Arc<Mutex<Vec<u8>>>,
    reader: Option<JoinHandle<()>>,
}

impl Drop for Pty {
    fn drop(&mut self) {
        let _ = self.child.kill();
        let _ = self.child.wait();
    }
}

fn sized_command(args: &[&str]) -> Vec<String> {
    let run = r#"/bin/stty cols 109 rows 36; exec "$0" "$@""#;

    [
        vec![
            "/bin/sh".to_string(),
            "-c".to_string(),
            run.to_string(),
            env!("CARGO_BIN_EXE_todo").to_string(),
        ],
        args.iter().map(|arg| arg.to_string()).collect(),
    ]
    .concat()
}

fn script(command: &[String]) -> Command {
    let mut script = Command::new("/usr/bin/script");

    if cfg!(target_os = "macos") {
        script.args(["-q", "/dev/null"]).args(command);
    } else {
        let quoted: Vec<String> = command.iter().map(|word| format!("'{}'", word.replace('\'', r"'\''"))).collect();
        script.args(["-qec", &quoted.join(" "), "/dev/null"]);
    }

    script
}

impl Pty {
    fn open(dir: &Path, args: &[&str], no_color: bool) -> Pty {
        let mut command = script(&sized_command(args));
        command
            .env_clear()
            .env("PATH", format!("{}:/bin", dir.join("bin").display()))
            .env("SHELL", "/bin/sh")
            .env("TERM", "xterm-256color")
            .env("HOME", dir)
            .env("TODO_FILE", dir.join("todo.txt"))
            .env("XDG_CONFIG_HOME", dir.join("config"));

        if no_color {
            command.env("NO_COLOR", "1");
        }

        let mut child = command
            .stdin(Stdio::piped())
            .stdout(Stdio::piped())
            .stderr(Stdio::null())
            .spawn()
            .unwrap();
        let stdin = child.stdin.take().unwrap();
        let mut stdout = child.stdout.take().unwrap();
        let output = Arc::new(Mutex::new(Vec::new()));
        let filled = Arc::clone(&output);
        let reader = std::thread::spawn(move || {
            let mut chunk = [0; 4096];
            while let Ok(read) = stdout.read(&mut chunk)
                && read > 0
            {
                filled.lock().unwrap().extend_from_slice(&chunk[..read]);
            }
        });

        Pty {
            child,
            stdin,
            output,
            reader: Some(reader),
        }
    }

    fn screen(&self) -> String {
        String::from_utf8_lossy(&self.output.lock().unwrap()).into_owned()
    }

    fn wait_for(&self, text: &str) {
        let start = Instant::now();

        while !self.screen().contains(text) {
            assert!(start.elapsed() < DEADLINE, "{text:?} never showed in {:?}", self.screen());
            std::thread::sleep(Duration::from_millis(20));
        }
    }

    fn press(&mut self, keys: &str) {
        self.stdin.write_all(keys.as_bytes()).unwrap();
        self.stdin.flush().unwrap();
    }

    fn finish(mut self) -> (ExitStatus, String) {
        let start = Instant::now();

        loop {
            if let Some(status) = self.child.try_wait().unwrap() {
                self.reader.take().unwrap().join().unwrap();

                return (status, self.screen());
            }

            if start.elapsed() > DEADLINE {
                panic!("todo did not exit: {:?}", self.screen());
            }

            std::thread::sleep(Duration::from_millis(20));
        }
    }
}

fn dir_with(name: &str, tasks: &str) -> PathBuf {
    let dir = folder(name);
    std::fs::create_dir_all(dir.join("bin")).unwrap();
    std::fs::create_dir_all(dir.join("config/todo")).unwrap();
    std::fs::write(dir.join("todo.txt"), tasks).unwrap();

    dir
}

fn colours(output: &str) -> bool {
    ["\x1b[38;", "\x1b[48;", "\x1b[39m", "\x1b[49m"].iter().any(|code| output.contains(code))
}

#[test]
fn the_list_opens_full_screen_on_the_tasks_and_q_leaves_it() {
    let dir = dir_with("pty-list", "(A) call the bank\n");
    let mut pty = Pty::open(&dir, &[], false);

    pty.wait_for("bank");
    pty.press("q");
    let (status, output) = pty.finish();

    assert!(status.success(), "{output:?}");
    assert!(output.contains("\x1b[?1049h") && output.contains("\x1b[?1049l"), "{output:?}");
}

#[test]
fn a_refused_config_is_shown_full_screen_until_a_key_is_pressed() {
    let dir = dir_with("pty-error", "call the bank\n");
    std::fs::write(dir.join("config/todo/config.toml"), "[theme]\nname = \"mauve\"\n").unwrap();
    let mut pty = Pty::open(&dir, &[], false);

    pty.wait_for("press");
    pty.press("x");
    let (status, output) = pty.finish();

    assert!(!status.success(), "{output:?}");
    assert!(output.contains("mauve"), "{output:?}");
}

#[test]
fn yy_hands_the_task_line_to_pbcopy_and_says_it_was_copied() {
    let dir = dir_with("pty-copy", "(A) call the bank\n");
    let pbcopy = dir.join("bin/pbcopy");
    std::fs::write(&pbcopy, format!("#!/bin/sh\nexec cat > '{}'\n", dir.join("clipboard").display())).unwrap();
    std::fs::set_permissions(&pbcopy, std::fs::Permissions::from_mode(0o755)).unwrap();
    let mut pty = Pty::open(&dir, &[], false);

    pty.wait_for("bank");
    pty.press("yy");
    pty.wait_for("copied");
    pty.press("q");
    pty.finish();

    assert_eq!(std::fs::read_to_string(dir.join("clipboard")).unwrap(), "(A) call the bank");
}

#[test]
fn todo_list_on_a_terminal_under_no_color_keeps_the_bold_and_has_no_colour() {
    let dir = dir_with("pty-no-color", "(A) call the bank\n");

    let (status, output) = Pty::open(&dir, &["list"], true).finish();

    assert!(status.success(), "{output:?}");
    assert!(output.contains("\x1b[1m A "), "{output:?}");
    assert!(!colours(&output), "{output:?}");
}

#[test]
fn a_theme_named_in_the_config_colours_todo_list_on_a_terminal_despite_no_color() {
    let dir = dir_with("pty-named", "(A) call the bank\n");
    std::fs::write(dir.join("config/todo/config.toml"), "[theme]\nname = \"catppuccin-mocha\"\n").unwrap();

    let (status, output) = Pty::open(&dir, &["list"], true).finish();

    assert!(status.success(), "{output:?}");
    assert!(colours(&output), "{output:?}");
}
