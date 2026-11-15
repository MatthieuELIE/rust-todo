use crate::todo::Todo;
use std::fmt;
use std::fs;
use std::io::{self, Read, Seek, SeekFrom, Write};
use std::os::unix::fs::{OpenOptionsExt, PermissionsExt};
use std::path::{Path, PathBuf};

const MAX_BYTES: u64 = 10 * 1024 * 1024;

/// Read the raw text of the todo file, empty if the file is missing, refused beyond 10 MiB.
pub fn read(path: &Path) -> io::Result<String> {
    let file = match fs::File::open(path) {
        Err(e) if e.kind() == io::ErrorKind::NotFound => return Ok(String::new()),
        file => file?,
    };

    let mut bytes = Vec::new();
    file.take(MAX_BYTES + 1).read_to_end(&mut bytes)?;
    if bytes.len() as u64 > MAX_BYTES {
        return Err(io::Error::new(io::ErrorKind::InvalidData, "larger than 10 MiB"));
    }

    String::from_utf8(bytes).map_err(|_| io::Error::new(io::ErrorKind::InvalidData, "stream did not contain valid UTF-8"))
}

/// Parse the tasks of a todo file's text, its blank lines dropped.
pub fn parse(text: &str) -> Vec<Todo> {
    let lines = text.split('\n').map(|line| line.trim_end_matches('\r'));
    lines.filter(|line| !line.trim().is_empty()).map(Todo::from_line).collect()
}

/// Load the todo list from a file along with its raw text, both empty if the file is missing.
pub fn load(path: &Path) -> io::Result<(String, Vec<Todo>)> {
    let text = read(path)?;
    let todos = parse(&text);
    Ok((text, todos))
}

/// Save the todo list to a file, overwriting any existing content, and return the text written.
pub fn save(path: &Path, todos: &[Todo]) -> io::Result<String> {
    write(path, todos.iter().map(|todo| todo.to_line() + "\n").collect())
}

/// Replace the content of a file with `body` through a sibling file renamed over it, so a crash leaves no half-written file, and return it.
pub fn write(path: &Path, body: String) -> io::Result<String> {
    let path = fs::canonicalize(path).unwrap_or_else(|_| match fs::read_link(path) {
        Ok(target) => path.with_file_name(target),
        Err(_) => path.to_path_buf(),
    });

    let mut tmp = path.clone().into_os_string();
    tmp.push(format!(".{}.tmp", std::process::id()));
    let tmp = PathBuf::from(tmp);

    let permissions = match fs::OpenOptions::new().write(true).open(&path) {
        Ok(file) => Some(file.metadata()?.permissions()),
        Err(e) if e.kind() == io::ErrorKind::NotFound => None,
        Err(e) => return Err(e),
    };

    let written = (|| {
        let mut file = fs::OpenOptions::new().write(true).create_new(true).mode(0o600).open(&tmp)?;

        if let Some(permissions) = permissions {
            file.set_permissions(fs::Permissions::from_mode(permissions.mode() & 0o700))?;
        }

        file.write_all(body.as_bytes())?;
        file.sync_all()?;
        fs::rename(&tmp, &path)
    })();
    if written.is_err() {
        let _ = fs::remove_file(&tmp);
    }
    written.map(|()| body)
}

/// Add the tasks to the end of a file, created if it is missing, without rewriting what it holds.
pub fn append(path: &Path, todos: &[Todo]) -> io::Result<()> {
    let mut body: String = todos.iter().map(|todo| todo.to_line() + "\n").collect();

    let mut file = fs::OpenOptions::new().read(true).append(true).create(true).mode(0o600).open(path)?;
    let metadata = file.metadata()?;
    let mode = metadata.permissions().mode();

    if metadata.len() > 0 {
        let mut last = [0u8];
        file.seek(SeekFrom::End(-1))?;
        file.read_exact(&mut last)?;

        if last[0] != b'\n' {
            body.insert(0, '\n');
        }
    }

    if mode & 0o077 != 0 {
        file.set_permissions(fs::Permissions::from_mode(mode & 0o700))?;
    }

    file.write_all(body.as_bytes())?;
    file.sync_all()
}

/// Remove from a file its task `number` when it reads as `line`, else its last line that does, the others kept as written, and tell whether there was one.
pub fn remove_task(path: &Path, number: Option<usize>, line: &str) -> io::Result<bool> {
    let text = read(path)?;
    let mut lines: Vec<&str> = text.split_inclusive('\n').collect();
    let reads = |i: &usize| Todo::from_line(lines[*i].trim_end_matches(['\n', '\r'])).to_line() == line;
    let mut tasks = (0..lines.len()).filter(|&i| !lines[i].trim().is_empty());
    let numbered = number.and_then(|number| tasks.clone().nth(number - 1)).filter(reads);
    let Some(index) = numbered.or_else(|| tasks.rfind(reads)) else {
        return Ok(false);
    };
    lines.remove(index);
    write(path, lines.concat()).map(|_| true)
}

/// Whether both paths lead to one existing file.
pub fn same_file(a: &Path, b: &Path) -> bool {
    fs::canonicalize(a).is_ok_and(|a| fs::canonicalize(b).is_ok_and(|b| a == b))
}

/// Why `move_to_done` stopped: the task file is `done.txt`, or a write failed on `path`, `moved` telling that `done.txt` already holds the tasks.
#[derive(Debug)]
pub enum MoveError {
    SameFile,
    Failed { path: PathBuf, error: io::Error, moved: bool },
}

impl fmt::Display for MoveError {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        match self {
            Self::SameFile => write!(f, "the task file is done.txt itself"),
            Self::Failed { error, .. } => write!(f, "{error}"),
        }
    }
}

fn writable(path: &Path) -> io::Result<()> {
    fs::OpenOptions::new().write(true).open(path).map(drop)
}

/// Save `todos` to `path` once the tasks `gone` are appended to `done_path`, destination first, and return the text saved.
/// Refuses a `path` that is `done_path` when a task moves (`gone`, or `back` for tasks leaving `done_path`), and checks `path`, then `done_path` when it `reopens`, open for writing before writing either.
pub fn move_to_done(path: &Path, done_path: &Path, todos: &[Todo], gone: &[Todo], back: bool, reopens: bool) -> Result<String, MoveError> {
    let failed = |at: &Path, error, moved| MoveError::Failed {
        path: at.to_path_buf(),
        error,
        moved,
    };

    if (!gone.is_empty() || back) && same_file(path, done_path) {
        return Err(MoveError::SameFile);
    }

    if reopens {
        writable(done_path).map_err(|e| failed(done_path, e, false))?;
    }

    if !gone.is_empty() {
        writable(path).map_err(|e| failed(path, e, false))?;
        append(done_path, gone).map_err(|e| failed(done_path, e, false))?;
    }

    save(path, todos).map_err(|e| failed(path, e, !gone.is_empty()))
}

#[cfg(test)]
mod tests;
