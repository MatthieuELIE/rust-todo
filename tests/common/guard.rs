use std::ops::Deref;
use std::path::{Path, PathBuf};

pub struct Folder(PathBuf);

impl Folder {
    pub fn new(name: &str) -> Folder {
        let path = std::env::temp_dir().join(format!("todo-{}-{name}", std::process::id()));
        let _ = std::fs::remove_dir_all(&path);
        std::fs::create_dir(&path).unwrap();
        Folder(path)
    }
}

impl Deref for Folder {
    type Target = Path;

    fn deref(&self) -> &Path {
        &self.0
    }
}

impl AsRef<Path> for Folder {
    fn as_ref(&self) -> &Path {
        &self.0
    }
}

impl Drop for Folder {
    fn drop(&mut self) {
        make_writable(&self.0);
        if let Err(error) = std::fs::remove_dir_all(&self.0) {
            eprintln!("could not remove {}: {error}", self.0.display());
        }
    }
}

fn make_writable(folder: &Path) {
    use std::os::unix::fs::PermissionsExt;
    let _ = std::fs::set_permissions(folder, std::fs::Permissions::from_mode(0o755));
    let Ok(entries) = std::fs::read_dir(folder) else { return };

    for entry in entries.flatten() {
        if entry.file_type().is_ok_and(|kind| kind.is_dir()) {
            make_writable(&entry.path());
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn a_dropped_folder_no_longer_exists() {
        let folder = Folder::new("guard-drop");
        let path = folder.to_path_buf();
        std::fs::write(folder.join("file"), "x").unwrap();

        drop(folder);

        assert!(!path.exists());
    }

    #[test]
    fn a_dropped_folder_holding_a_read_only_folder_no_longer_exists() {
        use std::os::unix::fs::PermissionsExt;
        let folder = Folder::new("guard-locked");
        let path = folder.to_path_buf();
        let locked = folder.join("locked");
        std::fs::create_dir(&locked).unwrap();
        std::fs::write(locked.join("file"), "x").unwrap();
        std::fs::set_permissions(&locked, std::fs::Permissions::from_mode(0o555)).unwrap();

        drop(folder);

        assert!(!path.exists());
    }

    #[test]
    fn a_folder_is_removed_when_its_test_panics() {
        let path = std::env::temp_dir().join(format!("todo-{}-guard-panic", std::process::id()));

        let outcome = std::panic::catch_unwind(|| {
            let folder = Folder::new("guard-panic");
            assert!(folder.exists());
            panic!("the test fails");
        });

        assert!(outcome.is_err());
        assert!(!path.exists());
    }
}
