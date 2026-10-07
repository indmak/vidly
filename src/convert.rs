use std::path::{Path, PathBuf};
use std::process::Stdio;
use std::time::{Duration, Instant};
use thiserror::Error;
use tokio::io::AsyncBufReadExt;
use tokio_util::sync::CancellationToken;

#[derive(Debug, Clone, Error)]
pub enum ConvertError {
    #[error("已取消")]
    Canceled,
    #[error("源文件不存在")]
    InputMissing,
    #[error("读取时长失败（文件可能损坏）")]
    ProbeFailed,
    #[error("无法启动 ffmpeg：{0}")]
    Spawn(String),
    #[error("ffmpeg 异常退出（{code}）：{tail}")]
    Exit { code: i32, tail: String },
    #[error("IO 错误：{0}")]
    Io(String),
}

// `std::io::Error` is not `Clone`, but `Message` (which carries `ConvertError`)
// must be `Clone`; store the message text instead.
impl From<std::io::Error> for ConvertError {
    fn from(e: std::io::Error) -> Self {
        ConvertError::Io(e.to_string())
    }
}

#[derive(Debug, Clone)]
pub struct FfmpegPaths {
    pub ffmpeg: PathBuf,
    pub ffprobe: PathBuf,
}

#[derive(Debug, Clone)]
pub struct Request {
    pub input: PathBuf,
    pub output: PathBuf,
    pub faststart: bool,
    pub first_audio_only: bool,
    pub keep_subtitles: bool,
}

/// Locate ffmpeg/ffprobe: (1) next to the exe (portable/dist layout), (2) deb layout
/// (/usr/lib/vidly), (3) PATH, (4) well-known install dirs.
/// (4) matters on macOS: a GUI app launched from Finder does not inherit the user's
/// shell PATH, so a Homebrew ffmpeg in /opt/homebrew/bin would otherwise be invisible.
pub fn find_ffmpeg() -> Option<FfmpegPaths> {
    Some(FfmpegPaths {
        ffmpeg: find_binary("ffmpeg")?,
        ffprobe: find_binary("ffprobe")?,
    })
}

const COMMON_DIRS: &[&str] = &[
    "/opt/homebrew/bin",          // macOS Apple Silicon
    "/usr/local/bin",             // macOS Intel / Linux manual install
    "/opt/local/bin",             // MacPorts
    "/run/current-system/sw/bin", // NixOS
];

fn find_binary(name: &str) -> Option<PathBuf> {
    #[cfg(windows)]
    let name = format!("{name}.exe");

    if let Ok(exe) = std::env::current_exe() {
        if let Some(dir) = exe.parent() {
            let candidate = dir.join(&name);
            if candidate.is_file() {
                return Some(candidate);
            }
        }
        // deb install layout: the main binary is in /usr/bin, ffmpeg in /usr/lib/vidly.
        if let Some(usr) = exe.parent().and_then(|p| p.parent()) {
            let candidate = usr.join("lib/vidly").join(&name);
            if candidate.is_file() {
                return Some(candidate);
            }
        }
    }

    if let Some(path) = std::env::var_os("PATH") {
        if let Some(found) = std::env::split_paths(&path)
            .map(|dir| dir.join(&name))
            .find(|p| p.is_file())
        {
            return Some(found);
        }
    }

    COMMON_DIRS
        .iter()
        .map(|dir| Path::new(dir).join(&name))
        .find(|p| p.is_file())
}

/// Prevent a console window from flashing when spawning a console child process.
#[cfg(windows)]
fn hide_console(cmd: &mut tokio::process::Command) {
    const CREATE_NO_WINDOW: u32 = 0x0800_0000;
    cmd.creation_flags(CREATE_NO_WINDOW);
}

#[cfg(not(windows))]
fn hide_console(_cmd: &mut tokio::process::Command) {}

async fn probe_duration(ffprobe: &Path, input: &Path) -> Result<f64, ConvertError> {
    let mut command = tokio::process::Command::new(ffprobe);
    command
        .args([
            "-v",
            "error",
            "-show_entries",
            "format=duration",
            "-of",
            "default=noprint_wrappers=1:nokey=1",
        ])
        .arg(input);
    hide_console(&mut command);
    let out = command.output().await.map_err(|_| ConvertError::ProbeFailed)?;
    String::from_utf8_lossy(&out.stdout)
        .trim()
        .parse::<f64>()
        .map_err(|_| ConvertError::ProbeFailed)
}

/// Runs a single remux: only the container is swapped (MP4/MOV are both ISO BMFF),
/// audio/video streams are copied verbatim -> lossless, near file-copy speed.
pub async fn run<P>(
    paths: &FfmpegPaths,
    req: Request,
    cancel: CancellationToken,
    progress: P,
) -> Result<Duration, ConvertError>
where
    P: Fn(f32) + Send + 'static,
{
    let started = Instant::now();
    if !req.input.is_file() {
        return Err(ConvertError::InputMissing);
    }

    let total = probe_duration(&paths.ffprobe, &req.input).await?;
    tracing::debug!(duration = total, input = %req.input.display(), "已探测时长");
    if let Some(dir) = req.output.parent() {
        tokio::fs::create_dir_all(dir).await?;
    }

    let out_ext = req.output.extension().and_then(|e| e.to_str()).unwrap_or("");
    let is_mp4_like = ["mp4", "mov", "m4v"]
        .iter()
        .any(|e| out_ext.eq_ignore_ascii_case(e));

    // Map video + (all or first) audio; subtitles are dropped by default (poor cross-container support).
    let mut map = vec!["-map", "0:v?"];
    if req.first_audio_only {
        map.extend(["-map", "0:a:0?"]);
    } else {
        map.extend(["-map", "0:a?"]);
    }
    if req.keep_subtitles {
        map.extend(["-map", "0:s?"]);
    }

    let mut cmd = tokio::process::Command::new(&paths.ffmpeg);
    cmd.args(["-y", "-hide_banner", "-loglevel", "error", "-nostdin"]) // GUI apps must pass -nostdin
        .arg("-i")
        .arg(&req.input)
        .args(["-c", "copy"])
        .args(&map)
        .args(["-progress", "pipe:1", "-nostats"]);
    if is_mp4_like && req.faststart {
        cmd.args(["-movflags", "+faststart"]); // moov up front, better for streaming
    }
    cmd.arg(&req.output)
        .stdout(Stdio::piped())
        .stderr(Stdio::piped());
    hide_console(&mut cmd);

    let mut child = cmd.spawn().map_err(|e| {
        tracing::error!(error = %e, "无法启动 ffmpeg");
        ConvertError::Spawn(e.to_string())
    })?;

    // Drain stderr separately to avoid filling the pipe and deadlocking; keep the tail for errors.
    let stderr_task = {
        let mut stderr = child.stderr.take().expect("piped");
        tokio::spawn(async move {
            use tokio::io::AsyncReadExt;
            let mut buf = String::new();
            let _ = stderr.read_to_string(&mut buf).await;
            buf
        })
    };

    let mut lines = tokio::io::BufReader::new(child.stdout.take().expect("piped")).lines();
    loop {
        tokio::select! {
            _ = cancel.cancelled() => {
                let _ = child.kill().await;
                let _ = child.wait().await;
                let _ = tokio::fs::remove_file(&req.output).await; // clean up the half-written file
                return Err(ConvertError::Canceled);
            }
            line = lines.next_line() => {
                let Some(line) = line? else { break };
                // Gotcha: out_time_ms is a legacy name; the unit is actually microseconds.
                if let Some(v) = line.strip_prefix("out_time_ms=") {
                    if let Ok(us) = v.trim().parse::<f64>() {
                        if total > 0.0 {
                            let pct = (us / 1_000_000.0 / total * 100.0) as f32;
                            progress(pct.clamp(0.0, 100.0));
                        }
                    }
                }
            }
        }
    }

    let status = child.wait().await?;
    let stderr = stderr_task.await.unwrap_or_default();
    if !status.success() {
        let _ = tokio::fs::remove_file(&req.output).await;
        let tail = stderr.lines().rev().take(3).collect::<Vec<_>>().join(" | ");
        tracing::warn!(code = status.code().unwrap_or(-1), %tail, "ffmpeg 异常退出");
        return Err(ConvertError::Exit { code: status.code().unwrap_or(-1), tail });
    }

    tracing::debug!(elapsed = ?started.elapsed(), "remux 完成");
    Ok(started.elapsed())
}
