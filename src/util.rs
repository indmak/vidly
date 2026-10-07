use std::path::{Path, PathBuf};

/// Supported input containers.
pub const SUPPORTED: [&str; 5] = ["mp4", "mov", "m4v", "mkv", "webm"];

pub fn is_supported(p: &Path) -> bool {
    p.extension()
        .and_then(|e| e.to_str())
        .is_some_and(|e| SUPPORTED.iter().any(|s| e.eq_ignore_ascii_case(s)))
}

/// Default target extension for `Auto`: mp4/m4v -> mov; mov/mkv/webm -> mp4.
pub fn auto_target(path: &Path) -> &'static str {
    let ext = path.extension().and_then(|e| e.to_str()).unwrap_or("");
    if ext.eq_ignore_ascii_case("mov") {
        "mp4"
    } else if ext.eq_ignore_ascii_case("mkv") || ext.eq_ignore_ascii_case("webm") {
        "mp4"
    } else {
        "mov"
    }
}

/// Planned output path: output dir (defaults to the source dir) + original stem + target extension.
pub fn planned_output(path: &Path, out_dir: Option<&Path>, ext: &str) -> PathBuf {
    let dir = out_dir
        .map(|d| d.to_path_buf())
        .unwrap_or_else(|| path.parent().map(|p| p.to_path_buf()).unwrap_or_default());
    let stem = path.file_stem().and_then(|s| s.to_str()).unwrap_or("output");
    dir.join(format!("{stem}.{ext}"))
}

/// Auto-rename: name.mp4 -> name (1).mp4 -> name (2).mp4 ...
pub fn unique_path(mut p: PathBuf) -> PathBuf {
    if !p.exists() {
        return p;
    }
    let stem = p.file_stem().and_then(|s| s.to_str()).unwrap_or("output").to_owned();
    let ext = p.extension().and_then(|e| e.to_str()).map(|e| format!(".{e}")).unwrap_or_default();
    let parent = p.parent().map(|d| d.to_path_buf()).unwrap_or_default();
    for n in 1.. {
        p = parent.join(format!("{stem} ({n}){ext}"));
        if !p.exists() {
            return p;
        }
    }
    unreachable!()
}

/// Human-readable byte size, e.g. 1536 -> "1.5 KB".
pub fn human_size(bytes: u64) -> String {
    const UNITS: [&str; 5] = ["B", "KB", "MB", "GB", "TB"];
    let mut value = bytes as f64;
    let mut unit = 0;
    while value >= 1024.0 && unit < UNITS.len() - 1 {
        value /= 1024.0;
        unit += 1;
    }
    if unit == 0 {
        format!("{bytes} B")
    } else {
        format!("{value:.1} {}", UNITS[unit])
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn unique() {
        let p = unique_path(std::env::temp_dir().join("not_exists_qwerty987.mp4"));
        assert!(p.to_str().unwrap().ends_with("not_exists_qwerty987.mp4"));
    }

    #[test]
    fn supported() {
        assert!(is_supported(Path::new("a.MP4")));
        assert!(is_supported(Path::new("a.mov")));
        assert!(is_supported(Path::new("a.m4v")));
        assert!(is_supported(Path::new("a.mkv")));
        assert!(is_supported(Path::new("a.webm")));
        assert!(!is_supported(Path::new("a.avi")));
        assert!(!is_supported(Path::new("a")));
    }

    #[test]
    fn planned() {
        let out = planned_output(Path::new("/v/clip.mp4"), Some(Path::new("/out")), "mov");
        assert_eq!(out, PathBuf::from("/out/clip.mov"));
        let same = planned_output(Path::new("/v/clip.mov"), None, "mp4");
        assert_eq!(same, PathBuf::from("/v/clip.mp4"));
    }

    #[test]
    fn auto_targets() {
        assert_eq!(auto_target(Path::new("a.mp4")), "mov");
        assert_eq!(auto_target(Path::new("a.MOV")), "mp4");
        assert_eq!(auto_target(Path::new("a.mkv")), "mp4");
        assert_eq!(auto_target(Path::new("a.webm")), "mp4");
    }

    #[test]
    fn sizes() {
        assert_eq!(human_size(512), "512 B");
        assert_eq!(human_size(1536), "1.5 KB");
        assert_eq!(human_size(5 * 1024 * 1024), "5.0 MB");
    }
}
