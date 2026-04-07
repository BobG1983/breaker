//! Async file-based logging via a background thread + mpsc channel.
//!
//! `RunLog` provides non-blocking `write_line` and `write_lines` methods that
//! send log operations to a background writer thread. `flush` blocks until all
//! pending writes are complete, and `shutdown` cleanly terminates the background
//! thread.

use std::{
    io::{self, BufWriter, Write},
    path::{Path, PathBuf},
    sync::{Arc, Mutex, mpsc},
    time::{SystemTime, UNIX_EPOCH},
};

use super::output_dir::civil_from_days;

/// Internal channel message type for the background writer thread.
enum LogOp {
    /// Write a single line (with trailing newline) to the log file.
    WriteLine(String),
    /// Flush all pending writes and signal completion via the sender.
    Flush(mpsc::Sender<()>),
    /// Shut down the background writer thread.
    Shutdown,
}

/// Async log writer backed by a background IO thread.
///
/// Wraps `Arc<mpsc::Sender<LogOp>>` so it is `Clone` and `Send`.
/// All clones share the same underlying channel and log file.
#[derive(Clone)]
pub struct RunLog {
    sender: Arc<mpsc::Sender<LogOp>>,
    path: PathBuf,
    join_handle: Arc<Mutex<Option<std::thread::JoinHandle<()>>>>,
}

impl RunLog {
    /// Creates a new `RunLog` that writes to the given `path`.
    ///
    /// Creates intermediate parent directories if needed, creates the log file,
    /// and spawns the background writer thread.
    ///
    /// # Errors
    ///
    /// Returns an I/O error if directory creation or file creation fails.
    pub fn new(path: &Path) -> io::Result<Self> {
        if let Some(parent) = path.parent() {
            std::fs::create_dir_all(parent)?;
        }
        let file = std::fs::File::create(path)?;
        let (sender, receiver) = mpsc::channel();

        let handle = std::thread::spawn(move || {
            let mut writer = BufWriter::new(file);
            loop {
                match receiver.recv() {
                    Ok(LogOp::WriteLine(text)) => {
                        drop(writeln!(writer, "{text}"));
                    }
                    Ok(LogOp::Flush(responder)) => {
                        drop(writer.flush());
                        let _ = responder.send(());
                    }
                    Ok(LogOp::Shutdown) => {
                        drop(writer.flush());
                        break;
                    }
                    Err(_) => {
                        // All senders dropped — flush and exit.
                        drop(writer.flush());
                        break;
                    }
                }
            }
        });

        Ok(Self {
            sender: Arc::new(sender),
            path: path.to_path_buf(),
            join_handle: Arc::new(Mutex::new(Some(handle))),
        })
    }

    /// Sends a single line to the background writer (non-blocking).
    ///
    /// Silent no-op if the channel is disconnected (after shutdown).
    pub fn write_line(&self, line: &str) {
        drop(self.sender.send(LogOp::WriteLine(line.to_owned())));
    }

    /// Sends multiple lines to the background writer (non-blocking).
    ///
    /// Silent no-op if the channel is disconnected (after shutdown).
    pub fn write_lines(&self, lines: impl IntoIterator<Item = impl AsRef<str>>) {
        for line in lines {
            drop(self.sender.send(LogOp::WriteLine(line.as_ref().to_owned())));
        }
    }

    /// Returns the path to the log file.
    #[must_use]
    pub fn path(&self) -> &Path {
        &self.path
    }

    /// Blocks until all pending writes are written to disk.
    ///
    /// No-op if the channel is closed (after shutdown).
    pub fn flush(&self) {
        let (tx, rx) = mpsc::channel();
        if self.sender.send(LogOp::Flush(tx)).is_ok() {
            // Block until the background thread acknowledges the flush.
            let _ = rx.recv();
        }
        // If send failed (channel disconnected after shutdown), return immediately.
    }

    /// Sends shutdown signal, joins the background thread; flushes implicitly.
    pub fn shutdown(self) {
        drop(self.sender.send(LogOp::Shutdown));
        if let Ok(mut guard) = self.join_handle.lock()
            && let Some(handle) = guard.take()
        {
            drop(handle.join());
        }
    }
}

/// Pure function returning `<base_dir>/<timestamp>.log`.
#[must_use]
pub fn log_file_path(base_dir: &Path, timestamp: &str) -> PathBuf {
    base_dir.join(format!("{timestamp}.log"))
}

/// Returns `<base_dir>/<timestamp>.log` if it does not exist on disk; otherwise
/// tries `<timestamp>-1.log`, `<timestamp>-2.log`, etc. until finding a path
/// that does not exist.
#[must_use]
pub fn resolve_log_file_path(base_dir: &Path, timestamp: &str) -> PathBuf {
    let base_path = log_file_path(base_dir, timestamp);
    if !base_path.exists() {
        return base_path;
    }
    let mut suffix = 1u32;
    loop {
        let candidate = base_dir.join(format!("{timestamp}-{suffix}.log"));
        if !candidate.exists() {
            return candidate;
        }
        suffix += 1;
    }
}

/// Returns a filesystem-safe ISO 8601 timestamp in `YYYY-MM-DDTHH-MM-SS` format (UTC).
#[must_use]
pub fn now_timestamp() -> String {
    let now = SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .unwrap_or_default();
    let total_secs = now.as_secs();
    let days_since_epoch = i64::try_from(total_secs / 86400).unwrap_or(0);
    let secs_of_day = total_secs % 86400;
    let hours = secs_of_day / 3600;
    let minutes = (secs_of_day % 3600) / 60;
    let seconds = secs_of_day % 60;
    let (year, month, day) = civil_from_days(days_since_epoch);
    format!("{year:04}-{month:02}-{day:02}T{hours:02}-{minutes:02}-{seconds:02}")
}

/// Creates a [`RunLog`] with a non-colliding log file path under `base_dir`.
///
/// # Errors
///
/// Returns an I/O error if directory creation or file creation fails.
pub fn create_run_log(base_dir: &Path) -> io::Result<RunLog> {
    let timestamp = now_timestamp();
    let path = resolve_log_file_path(base_dir, &timestamp);
    RunLog::new(path.as_path())
}

/// Pure function returning `"Full log available at: <path>"`.
#[must_use]
pub fn format_log_path_message(path: &Path) -> String {
    format!("Full log available at: {}", path.display())
}
