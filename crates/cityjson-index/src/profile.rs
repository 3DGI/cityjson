use std::fs::File;
use std::io::{BufRead, BufReader};
use std::path::{Path, PathBuf};
use std::time::{Instant, SystemTime, UNIX_EPOCH};

use cityjson_lib::{Error, Result};
use serde::Serialize;

#[derive(Debug, Clone, Serialize)]
pub struct MemorySnapshot {
    pub current_rss_bytes: u64,
    pub process_peak_rss_bytes: u64,
    /// Deprecated compatibility field. This is the Linux `VmHWM` value for the
    /// process lifetime, not a per-operation peak.
    pub peak_rss_bytes: u64,
}

#[derive(Debug, Clone, Serialize)]
pub struct ProfileStage {
    pub name: String,
    pub elapsed_ns: u64,
    pub memory_start: MemorySnapshot,
    pub memory_end: MemorySnapshot,
}

#[derive(Debug, Clone, Serialize)]
pub struct CommandProfile {
    pub binary_version: String,
    pub command: String,
    pub dataset_path: Option<PathBuf>,
    pub index_path: Option<PathBuf>,
    pub worker_count: Option<usize>,
    pub platform: String,
    pub cpu_count: usize,
    pub started_at_ns: u64,
    pub ended_at_ns: u64,
    pub success: bool,
    pub error: Option<String>,
    pub stages: Vec<ProfileStage>,
    pub memory_start: MemorySnapshot,
    pub memory_end: MemorySnapshot,
}

pub struct ProfileRecorder {
    enabled: bool,
    command: String,
    dataset_path: Option<PathBuf>,
    index_path: Option<PathBuf>,
    worker_count: Option<usize>,
    started_at_ns: u64,
    started_instant: Instant,
    memory_start: Option<MemorySnapshot>,
    stages: Vec<ProfileStage>,
}

impl ProfileRecorder {
    /// Creates an enabled recorder.
    ///
    /// # Errors
    ///
    /// Returns an error if the process cannot read its Linux memory snapshot.
    pub fn enabled(
        command: impl Into<String>,
        dataset_path: Option<PathBuf>,
        index_path: Option<PathBuf>,
        worker_count: Option<usize>,
    ) -> Result<Self> {
        Ok(Self {
            enabled: true,
            command: command.into(),
            dataset_path,
            index_path,
            worker_count,
            started_at_ns: unix_time_ns()?,
            started_instant: Instant::now(),
            memory_start: Some(current_memory_snapshot()?),
            stages: Vec::new(),
        })
    }

    pub fn disabled(
        command: impl Into<String>,
        dataset_path: Option<PathBuf>,
        index_path: Option<PathBuf>,
        worker_count: Option<usize>,
    ) -> Self {
        Self {
            enabled: false,
            command: command.into(),
            dataset_path,
            index_path,
            worker_count,
            started_at_ns: 0,
            started_instant: Instant::now(),
            memory_start: None,
            stages: Vec::new(),
        }
    }

    pub fn set_dataset_path(&mut self, dataset_path: Option<PathBuf>) {
        self.dataset_path = dataset_path;
    }

    pub fn set_index_path(&mut self, index_path: Option<PathBuf>) {
        self.index_path = index_path;
    }

    pub fn set_worker_count(&mut self, worker_count: Option<usize>) {
        self.worker_count = worker_count;
    }

    /// Measures a stage and records its duration and RSS snapshot.
    ///
    /// # Errors
    ///
    /// Returns an error if the wrapped operation fails or if RSS sampling
    /// fails before or after the operation.
    pub fn measure<T>(
        &mut self,
        name: impl Into<String>,
        f: impl FnOnce() -> Result<T>,
    ) -> Result<T> {
        if !self.enabled {
            return f();
        }

        let stage_name = name.into();
        let memory_start = current_memory_snapshot()?;
        let elapsed_start = Instant::now();
        let result = f();
        let memory_end = current_memory_snapshot()?;
        let elapsed_ns = u64::try_from(elapsed_start.elapsed().as_nanos()).map_err(|_| {
            Error::Import("profiling stage duration does not fit in u64".to_owned())
        })?;
        self.stages.push(ProfileStage {
            name: stage_name,
            elapsed_ns,
            memory_start,
            memory_end,
        });
        result
    }

    /// Finalizes the recorder and returns the complete profile payload.
    ///
    /// # Errors
    ///
    /// Returns an error if final RSS sampling or timestamp capture fails.
    pub fn finish(self, success: bool, error: Option<String>) -> Result<Option<CommandProfile>> {
        if !self.enabled {
            return Ok(None);
        }

        let Some(memory_start) = self.memory_start else {
            return Err(Error::Import(
                "enabled profile recorder did not capture start memory".to_owned(),
            ));
        };
        let memory_end = current_memory_snapshot()?;
        let ended_at_ns = unix_time_ns()?;
        let total_elapsed_ns =
            u64::try_from(self.started_instant.elapsed().as_nanos()).map_err(|_| {
                Error::Import("profiling total duration does not fit in u64".to_owned())
            })?;
        let total_stage = ProfileStage {
            name: "total command time".to_owned(),
            elapsed_ns: total_elapsed_ns,
            memory_start: memory_start.clone(),
            memory_end: memory_end.clone(),
        };
        let mut stages = self.stages;
        stages.push(total_stage);
        Ok(Some(CommandProfile {
            binary_version: env!("CARGO_PKG_VERSION").to_owned(),
            command: self.command,
            dataset_path: self.dataset_path,
            index_path: self.index_path,
            worker_count: self.worker_count,
            platform: std::env::consts::OS.to_owned(),
            cpu_count: std::thread::available_parallelism().map_or(1, std::num::NonZeroUsize::get),
            started_at_ns: self.started_at_ns,
            ended_at_ns,
            success,
            error,
            stages,
            memory_start,
            memory_end,
        }))
    }
}

/// Runs a command body with optional profile capture and writes the result.
///
/// # Errors
///
/// Returns an error if the command body fails, profile capture fails, or the
/// profile cannot be written to disk.
pub fn run_with_profile<F>(
    profile_path: Option<PathBuf>,
    command: impl Into<String>,
    dataset_path: Option<PathBuf>,
    index_path: Option<PathBuf>,
    worker_count: Option<usize>,
    body: F,
) -> Result<()>
where
    F: FnOnce(&mut ProfileRecorder) -> Result<()>,
{
    let mut recorder = if profile_path.is_some() {
        ProfileRecorder::enabled(command, dataset_path, index_path, worker_count)?
    } else {
        ProfileRecorder::disabled(command, dataset_path, index_path, worker_count)
    };

    let result = body(&mut recorder);
    if let Some(path) = profile_path {
        let profile = recorder.finish(
            result.is_ok(),
            result.as_ref().err().map(ToString::to_string),
        )?;
        let Some(profile) = profile.as_ref() else {
            return Err(Error::Import(
                "enabled profile recorder did not emit a profile".to_owned(),
            ));
        };
        write_profile_json(&path, profile)?;
    }
    result
}

/// Returns current RSS and process-lifetime peak RSS for the running process.
///
/// # Errors
///
/// Returns an error if the Linux memory status file cannot be read or parsed.
pub fn current_memory_snapshot() -> Result<MemorySnapshot> {
    #[cfg(target_os = "linux")]
    {
        parse_linux_memory_status()
    }

    #[cfg(not(target_os = "linux"))]
    {
        Err(Error::UnsupportedFeature(
            "profiling is only supported on Linux".to_owned(),
        ))
    }
}

/// Serializes a profile payload to the requested file path.
///
/// # Errors
///
/// Returns an error if the destination cannot be created or written.
pub fn write_profile_json(path: &Path, profile: &CommandProfile) -> Result<()> {
    if let Some(parent) = path
        .parent()
        .filter(|parent| !parent.as_os_str().is_empty())
    {
        std::fs::create_dir_all(parent)?;
    }
    let file = File::create(path)?;
    serde_json::to_writer_pretty(file, profile).map_err(|error| Error::Import(error.to_string()))
}

fn unix_time_ns() -> Result<u64> {
    let since_epoch = SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .map_err(|error| {
            Error::Import(format!("system clock is before the unix epoch: {error}"))
        })?;
    u64::try_from(since_epoch.as_nanos())
        .map_err(|_| Error::Import("timestamp does not fit in u64".to_owned()))
}

#[cfg(target_os = "linux")]
fn parse_linux_memory_status() -> Result<MemorySnapshot> {
    let status = File::open("/proc/self/status")?;
    let reader = BufReader::new(status);
    let mut current_rss_bytes = None;
    let mut peak_rss_bytes = None;

    for line in reader.lines() {
        let line = line?;
        if let Some(value) = line.strip_prefix("VmRSS:") {
            current_rss_bytes = Some(parse_linux_kib_to_bytes(value)?);
        } else if let Some(value) = line.strip_prefix("VmHWM:") {
            peak_rss_bytes = Some(parse_linux_kib_to_bytes(value)?);
        }
        if current_rss_bytes.is_some() && peak_rss_bytes.is_some() {
            break;
        }
    }

    let current_rss_bytes = current_rss_bytes
        .ok_or_else(|| Error::Import("VmRSS was not present in /proc/self/status".to_owned()))?;
    let process_peak_rss_bytes = peak_rss_bytes
        .ok_or_else(|| Error::Import("VmHWM was not present in /proc/self/status".to_owned()))?;

    Ok(MemorySnapshot {
        current_rss_bytes,
        process_peak_rss_bytes,
        peak_rss_bytes: process_peak_rss_bytes,
    })
}

#[cfg(target_os = "linux")]
fn parse_linux_kib_to_bytes(value: &str) -> Result<u64> {
    let kib = value
        .split_whitespace()
        .find_map(|part| part.parse::<u64>().ok())
        .ok_or_else(|| Error::Import("failed to parse Linux memory metric".to_owned()))?;
    kib.checked_mul(1024)
        .ok_or_else(|| Error::Import("Linux memory metric overflowed bytes".to_owned()))
}
