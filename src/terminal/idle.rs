use std::fs;
use std::io;
use std::path::Path;
use std::time::SystemTime;

/// What `metadata()` says of the task file: its modification date and size, `None` when it is missing.
pub type Stamp = Option<(SystemTime, u64)>;

/// Takes the stamp of the file at `path`.
pub fn stamp_of(path: &Path) -> io::Result<Stamp> {
    match fs::metadata(path) {
        Err(e) if e.kind() == io::ErrorKind::NotFound => Ok(None),
        Err(e) => Err(e),
        Ok(metadata) => Ok(Some((metadata.modified()?, metadata.len()))),
    }
}

/// Whether the file must be read: it has no stamp yet, its stamp changed, or the stamp could not be taken.
pub fn must_read(last: &Option<Stamp>, now: &io::Result<Stamp>) -> bool {
    match now {
        Err(_) => true,
        Ok(now) => last.as_ref() != Some(now),
    }
}

/// The stamp to compare the next turn with: none when it could not be taken or the file read badly, so the next turn reads again.
pub fn remembered(now: io::Result<Stamp>, read_failed: bool) -> Option<Stamp> {
    now.ok().filter(|_| !read_failed)
}

/// Whether the screen must be drawn again: a key or paste came, the file was reread, the toast expired or the day changed.
pub fn must_draw(event: bool, reread: bool, toast_expired: bool, new_day: bool) -> bool {
    event || reread || toast_expired || new_day
}

#[cfg(test)]
mod tests;
